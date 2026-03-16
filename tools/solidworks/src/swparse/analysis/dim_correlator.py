"""Correlate known XML dimension values with binary class instance offsets.

Uses the ``swXmlContents/KeyWords`` XML as ground truth — it contains
feature names, types, and dimension values in human-readable form. This
module finds where those exact float64 values appear within specific
class instances in the ``ResolvedFeatures`` stream, producing a mapping
of ``class_name + relative_offset → dimension_name``.
"""

from __future__ import annotations

import struct
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from typing import Optional

from ..container import SldFile
from ..serialization.classes import ClassInstance, find_all as find_classes


@dataclass
class DimensionCorrelation:
    """A confirmed link between an XML dimension and a binary class field."""

    dim_name: str  # e.g., "D1"
    dim_value_mm: float  # value in millimeters (from XML)
    dim_value_m: float  # value in meters (for binary search)
    class_name: str  # class instance containing this value
    relative_offset: int  # byte offset within the class payload
    stream_name: str
    source_file: str


def _parse_keywords_xml(sld: SldFile) -> list[tuple[str, str, float]]:
    """Extract (feature_name, dim_name, value_mm) from KeyWords XML."""
    dims: list[tuple[str, str, float]] = []

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
            feat_name = child.get("Name", "")

            for dim_elem in child:
                dim_tag = dim_elem.tag.split("}")[-1] if "}" in dim_elem.tag else dim_elem.tag
                if dim_tag != "Dimension":
                    continue
                dim_name = dim_elem.get("Name", "?")
                dim_text = dim_elem.text or ""

                # Parse dimension value (may have <MOD-DIAM> prefix, R prefix, ° suffix)
                clean = dim_text.replace("<MOD-DIAM>", "").replace("°", "").strip()
                if clean.startswith("R"):
                    clean = clean[1:]

                try:
                    value_mm = float(clean)
                    dims.append((feat_name, dim_name, value_mm))
                except ValueError:
                    pass

    return dims


def correlate(sld: SldFile) -> list[DimensionCorrelation]:
    """Find where XML dimension values appear in binary class instances."""
    source = str(sld.path) if sld.path else "<bytes>"
    xml_dims = _parse_keywords_xml(sld)
    if not xml_dims:
        return []

    results: list[DimensionCorrelation] = []

    for sname, stream in sld.streams.items():
        if "ResolvedFeatures" not in sname or not stream.data:
            continue

        all_instances = find_classes(stream.data)
        if not all_instances:
            continue

        for feat_name, dim_name, value_mm in xml_dims:
            value_m = value_mm / 1000.0
            if abs(value_m) < 1e-15:
                continue

            target = struct.pack("<d", value_m)

            # Search within each class instance's payload
            for i, inst in enumerate(all_instances):
                end = all_instances[i + 1].offset if i + 1 < len(all_instances) else len(stream.data)
                payload = stream.data[inst.data_offset : end]

                pos = 0
                while pos < len(payload) - 8:
                    hit = payload.find(target, pos)
                    if hit < 0:
                        break

                    results.append(
                        DimensionCorrelation(
                            dim_name=dim_name,
                            dim_value_mm=value_mm,
                            dim_value_m=value_m,
                            class_name=inst.name,
                            relative_offset=hit,
                            stream_name=sname,
                            source_file=source,
                        )
                    )
                    pos = hit + 1

    return results


def format_correlations(corrs: list[DimensionCorrelation]) -> str:
    """Format correlation results as a readable table."""
    lines = [
        f"{'Dim Name':<10s} {'Value (mm)':>12s} {'Class':<45s} {'Offset':>8s}",
        "-" * 80,
    ]
    for c in corrs:
        lines.append(
            f"  {c.dim_name:<8s} {c.dim_value_mm:>10.4f}   "
            f"{c.class_name:<43s} +0x{c.relative_offset:04x}"
        )
    return "\n".join(lines)
