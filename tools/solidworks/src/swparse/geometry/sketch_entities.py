"""Parse sketch topology and dimensions from SolidWorks ResolvedFeatures.

## Architecture

Sketch data in SolidWorks is split across multiple representations:

1. **Sub-records** (``FF FF 1F 00 03`` markers in the sgArcHandle region):
   Entity topology — IDs, types, and annotation display positions.
   These do NOT contain the actual geometric coordinates.

2. **Dimension classes** (``moLengthParameter_c``, ``sgEntHandle``,
   ``moSkDimHandleRadial_c``, ``ParallelPlaneDistanceDim_c``):
   Named parametric values like ``D1=0.008m`` (diameter 8mm).
   This is where dimension VALUES live.

3. **Parasolid XT** (``Contents/Config-N-Partition``):
   Evaluated B-rep geometry with exact coordinates.
   This is the geometric "truth" — surfaces, curves, points.

Sub-record layout::

    [-4]  entity_id : u32 LE
    [+0]  FF FF 1F 00 03           sub-record marker
    [+5]  FF FF FF FF FF FF FF FF  sentinel
    [+13] 00 00 80 BF              float32 = -1.0
    [+17] type_field : u32 LE      entity type discriminator
    [+21] flags/padding
    [+58] annotation_x : float64 LE  (display position, NOT geometry)
    [+66] annotation_y : float64 LE

Type field values:
    0 = Point entity
    1 = Line/circle entity
    2 = Arc entity
    3 = Constrained point
"""

from __future__ import annotations

import struct
from dataclasses import dataclass, field
from typing import Optional, Union

from ..container import SldFile
from ..serialization.classes import ClassInstance, find_all as find_classes

SUB_MARKER = b"\xff\xff\x1f\x00\x03"
SUB_MARKER_LEN = 5


# ── Entity topology (from sub-records) ──────────────────────────────────


@dataclass
class SketchEntity:
    """A sketch entity parsed from sub-record topology data."""

    entity_id: int
    entity_type: int  # 0=point, 1=line/circle, 2=arc, 3=constrained
    annotation_x: float  # display position (meters), NOT geometric coordinate
    annotation_y: float

    @property
    def type_name(self) -> str:
        return {0: "point", 1: "line/circle", 2: "arc", 3: "constrained"}.get(
            self.entity_type, f"unknown({self.entity_type})"
        )


# ── Dimensions (from dimension classes) ────────────────────────────────


@dataclass
class SketchDimension:
    """A named dimension value extracted from the feature tree."""

    name: str  # e.g., "D1"
    value_m: float  # value in meters
    class_name: str  # source class (moLengthParameter_c, etc.)
    is_diameter: bool = False  # True if <MOD-DIAM> prefix

    @property
    def value_mm(self) -> float:
        return self.value_m * 1000


# ── Sketch chain ───────────────────────────────────────────────────────


@dataclass
class SketchChain:
    """A chain of connected sketch entities forming a loop or path."""

    entity_ids: list[int]
    is_closed: bool = False


# ── Complete sketch summary ────────────────────────────────────────────


@dataclass
class SketchSummary:
    """Complete sketch data extracted from a SolidWorks file."""

    entities: list[SketchEntity]
    dimensions: list[SketchDimension]
    chains: list[SketchChain]

    @property
    def point_count(self) -> int:
        return sum(1 for e in self.entities if e.entity_type in (0, 3))

    @property
    def line_count(self) -> int:
        return sum(1 for e in self.entities if e.entity_type == 1)

    @property
    def arc_count(self) -> int:
        return sum(1 for e in self.entities if e.entity_type == 2)


# ── Sub-record parsing ─────────────────────────────────────────────────


def _find_sub_records(data: bytes) -> list[int]:
    """Find all sub-record marker positions in *data*."""
    positions: list[int] = []
    start = 0
    while True:
        pos = data.find(SUB_MARKER, start)
        if pos < 0:
            break
        positions.append(pos)
        start = pos + 1
    return positions


def _parse_sub_record(data: bytes, marker_pos: int) -> Optional[SketchEntity]:
    """Parse entity topology from a sub-record at *marker_pos*."""
    if marker_pos < 4:
        return None
    entity_id = struct.unpack_from("<I", data, marker_pos - 4)[0]

    # Type field at marker + 17
    type_offset = marker_pos + 17
    if type_offset + 4 > len(data):
        return None
    type_field = struct.unpack_from("<I", data, type_offset)[0]

    # Annotation position at marker + 58/66 (display position, NOT geometry)
    ann_x = 0.0
    ann_y = 0.0
    coord_offset = marker_pos + 58
    if coord_offset + 16 <= len(data):
        ann_x = struct.unpack_from("<d", data, coord_offset)[0]
        ann_y = struct.unpack_from("<d", data, coord_offset + 8)[0]
        # Sanity check: annotation positions should be reasonable screen coordinates
        if abs(ann_x) > 100 or abs(ann_y) > 100:
            ann_x = 0.0
            ann_y = 0.0

    return SketchEntity(
        entity_id=entity_id,
        entity_type=type_field,
        annotation_x=ann_x,
        annotation_y=ann_y,
    )


# ── Dimension extraction ──────────────────────────────────────────────


def _extract_dimensions_from_xml(sld: SldFile) -> list[SketchDimension]:
    """Extract dimension values from the KeyWords XML."""
    import xml.etree.ElementTree as ET

    dims: list[SketchDimension] = []

    for sname, stream in sld.streams.items():
        if "KeyWords" not in sname or not stream.data:
            continue

        text = stream.data.decode("utf-8", errors="replace")
        xml_start = text.find("<?xml")
        if xml_start < 0:
            continue

        try:
            root = ET.fromstring(text[xml_start:])
        except ET.ParseError:
            continue

        for child in root:
            tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag
            class_name = child.get("Type", tag)

            for dim_elem in child:
                dim_tag = dim_elem.tag.split("}")[-1] if "}" in dim_elem.tag else dim_elem.tag
                if dim_tag != "Dimension":
                    continue

                dim_name = dim_elem.get("Name", "?")
                dim_text = dim_elem.text or ""

                is_diameter = "<MOD-DIAM>" in dim_text
                clean = dim_text.replace("<MOD-DIAM>", "").replace("°", "").strip()
                if clean.startswith("R"):
                    clean = clean[1:]

                try:
                    value_mm = float(clean)
                    dims.append(
                        SketchDimension(
                            name=dim_name,
                            value_m=value_mm / 1000.0,
                            class_name=class_name,
                            is_diameter=is_diameter,
                        )
                    )
                except ValueError:
                    pass

    return dims


# ── Chain extraction ──────────────────────────────────────────────────


def _extract_chains(sld: SldFile) -> list[SketchChain]:
    """Extract sketch chains from moSketchChain_c instances."""
    chains: list[SketchChain] = []

    for sname, stream in sld.streams.items():
        if "ResolvedFeatures" not in sname or not stream.data:
            continue

        all_classes = find_classes(stream.data)
        for i, inst in enumerate(all_classes):
            if inst.name != "moSketchChain_c":
                continue

            end = all_classes[i + 1].offset if i + 1 < len(all_classes) else len(stream.data)
            payload = stream.data[inst.data_offset : end]

            if len(payload) < 2:
                continue

            count = struct.unpack_from("<H", payload, 0)[0]
            if count == 0 or count > 500:
                continue

            ids: list[int] = []
            offset = 2
            for _ in range(count):
                if offset + 4 > len(payload):
                    break
                eid = struct.unpack_from("<I", payload, offset)[0]
                ids.append(eid)
                offset += 4

            if ids:
                chains.append(SketchChain(entity_ids=ids))

    return chains


# ── Public API ────────────────────────────────────────────────────────


def extract_sketch(sld: SldFile) -> SketchSummary:
    """Extract complete sketch data from a parsed SolidWorks file.

    Returns a ``SketchSummary`` containing:
      - Entity topology (IDs, types) from sub-records
      - Named dimension values from XML
      - Sketch chains (entity connectivity)

    Note: Actual geometric coordinates (point positions, curve definitions)
    are in the Parasolid XT partition, not in the sketch sub-records.
    Use ``parasolid.extractor.extract_from_sld()`` for geometry.
    """
    # Extract sub-record entities
    entities: list[SketchEntity] = []
    for sname, stream in sld.streams.items():
        if "ResolvedFeatures" not in sname or not stream.data:
            continue
        positions = _find_sub_records(stream.data)
        for pos in positions:
            entity = _parse_sub_record(stream.data, pos)
            if entity is not None:
                entities.append(entity)

    # Extract dimensions from XML
    dimensions = _extract_dimensions_from_xml(sld)

    # Extract chains
    chains = _extract_chains(sld)

    return SketchSummary(entities=entities, dimensions=dimensions, chains=chains)
