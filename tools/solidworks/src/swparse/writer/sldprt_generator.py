"""Generate SLDPRT files from scratch using a template approach.

Takes a template SLDPRT file (e.g., a simple cylinder), replaces the
dimension values and XML metadata, and produces a new valid SLDPRT.

The Parasolid XT geometry is kept from the template (topology unchanged)
but all coordinate values are replaced. For a true geometry change
(e.g., cylinder → box), use the Parasolid text writer and import
the .x_t file into SolidWorks directly.

Usage::

    from swparse.writer.sldprt_generator import generate_sldprt

    data = generate_sldprt(
        template_path="template.SLDPRT",
        dimensions={"radius": 0.0127, "height": 0.0254},
        part_name="1inch_cube",
    )
    Path("output.SLDPRT").write_bytes(data)
"""

from __future__ import annotations

import struct
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from ..container import SldFile
from ..geometry.dimensions import replace_value
from ..records import compute_field1


@dataclass
class DimensionSpec:
    """A dimension replacement specification."""

    old_meters: float
    new_meters: float
    label: str = ""


def generate_sldprt(
    template_path: str | Path,
    dimensions: list[DimensionSpec],
    part_name: Optional[str] = None,
) -> bytes:
    """Generate a new SLDPRT by modifying a template.

    Args:
        template_path: Path to a valid SLDPRT template file.
        dimensions: List of dimension replacements (old → new, in meters).
        part_name: Optional new part name for XML metadata.

    Returns:
        The complete SLDPRT file as bytes.
    """
    sld = SldFile.open(template_path)

    # Build the full replacement list including derived values
    all_replacements: list[tuple[float, float, bool]] = []
    for spec in dimensions:
        # LE replacements
        all_replacements.append((spec.old_meters, spec.new_meters, False))
        all_replacements.append((-spec.old_meters, -spec.new_meters, False))
        all_replacements.append((spec.old_meters / 2, spec.new_meters / 2, False))
        # BE replacements (for Parasolid)
        all_replacements.append((spec.old_meters, spec.new_meters, True))
        all_replacements.append((-spec.old_meters, -spec.new_meters, True))

    # Modify each stream
    modified: dict[str, bytes] = {}

    for sname, stream in sld.streams.items():
        if not stream.data or len(stream.data) < 8:
            continue

        data = stream.data
        changed = False

        # Handle Parasolid partition specially (inner compression)
        if "Partition" in sname:
            data = _modify_partition(data, all_replacements)
            if data != stream.data:
                changed = True
        else:
            # Apply LE replacements to normal streams
            for old_val, new_val, is_be in all_replacements:
                if is_be:
                    continue  # skip BE for non-Parasolid streams
                data, count = replace_value(data, old_val, new_val, big_endian=False)
                if count > 0:
                    changed = True

        if changed:
            modified[sname] = data

    # Update XML metadata
    if part_name:
        for sname, stream in sld.streams.items():
            if "Features" in sname and stream.data:
                text = stream.data.decode("utf-8", errors="replace")
                # Replace old part name (heuristic: find Name= attribute)
                import re
                text = re.sub(
                    r'swPath="[^"]*"',
                    f'swPath="{part_name}.SLDPRT"',
                    text,
                )
                modified[sname] = text.encode("utf-8")

        # Update KeyWords XML dimensions
        for sname, stream in sld.streams.items():
            if "KeyWords" not in sname or not stream.data:
                continue
            text = modified.get(sname, stream.data).decode("utf-8", errors="replace")
            for spec in dimensions:
                old_mm = f"{spec.old_meters * 1000:g}"
                new_mm = f"{spec.new_meters * 1000:g}"
                text = text.replace(f">{old_mm}<", f">{new_mm}<")
                text = text.replace(f"&gt;{old_mm}", f"&gt;{new_mm}")
            modified[sname] = text.encode("utf-8")

    return sld.rebuild(modified_streams=modified)


def _modify_partition(
    outer_data: bytes,
    replacements: list[tuple[float, float, bool]],
) -> bytes:
    """Modify Parasolid partition with inner decompression."""
    for k in range(len(outer_data) - 1):
        if outer_data[k] == 0x78 and outer_data[k + 1] in (0x01, 0x5E, 0x9C, 0xDA):
            try:
                inner = zlib.decompress(outer_data[k:])
                if inner[:2] != b"PS":
                    continue

                for old_val, new_val, is_be in replacements:
                    inner, _ = replace_value(inner, old_val, new_val, big_endian=is_be)

                inner_recompressed = zlib.compress(inner, 6)
                return outer_data[:k] + inner_recompressed
            except zlib.error:
                continue

    return outer_data
