"""Find and modify dimension values across all streams in a SolidWorks file.

Dimension values (lengths, radii, angles) are stored as float64 in **meters**
(SI units) throughout multiple streams:

    - Contents/Config-0           (main configuration, bounding box)
    - Contents/Config-0-ResolvedFeatures  (parametric feature tree)
    - Contents/Config-0-LWDATA    (lightweight display data)
    - Contents/DisplayLists       (tessellated display mesh)
    - Header2                     (document header)
    - Contents/Config-0-Partition (Parasolid XT, big-endian!)

When modifying a dimension, ALL occurrences in ALL streams must be updated
for consistency.
"""

from __future__ import annotations

import struct
import zlib
from dataclasses import dataclass
from typing import Optional

from ..serialization.primitives import pack_f64, pack_f64_be


@dataclass
class DimensionHit:
    """A located dimension value."""

    stream_name: str
    offset: int  # byte offset within the decompressed stream
    value: float  # in meters
    value_mm: float  # convenience: value * 1000
    is_big_endian: bool  # True for Parasolid XT data


def find_value(
    sld_file,
    value_meters: float,
    tolerance: float = 1e-12,
) -> list[DimensionHit]:
    """Find all occurrences of *value_meters* across all streams."""
    target_le = pack_f64(value_meters)
    target_be = pack_f64_be(value_meters)
    hits: list[DimensionHit] = []

    for name, stream in sld_file.streams.items():
        if not stream.data:
            continue
        # Search for LE float64
        _search_bytes(stream.data, target_le, name, value_meters, False, hits)
        # Search for BE float64 (Parasolid)
        _search_bytes(stream.data, target_be, name, value_meters, True, hits)

    return hits


def _search_bytes(
    data: bytes,
    pattern: bytes,
    stream_name: str,
    value: float,
    is_be: bool,
    hits: list[DimensionHit],
) -> None:
    start = 0
    while True:
        pos = data.find(pattern, start)
        if pos < 0:
            break
        hits.append(
            DimensionHit(
                stream_name=stream_name,
                offset=pos,
                value=value,
                value_mm=value * 1000,
                is_big_endian=is_be,
            )
        )
        start = pos + 1


def replace_value(
    data: bytes,
    old_meters: float,
    new_meters: float,
    big_endian: bool = False,
) -> tuple[bytes, int]:
    """Replace all occurrences of *old_meters* with *new_meters* in *data*.

    Returns ``(modified_data, replacement_count)``.
    """
    if big_endian:
        old_bytes = pack_f64_be(old_meters)
        new_bytes = pack_f64_be(new_meters)
    else:
        old_bytes = pack_f64(old_meters)
        new_bytes = pack_f64(new_meters)

    count = data.count(old_bytes)
    if count > 0:
        data = data.replace(old_bytes, new_bytes)
    return data, count


def modify_dimension_in_sld(
    sld_file,
    old_meters: float,
    new_meters: float,
    *,
    prior_modifications: dict[str, bytes] | None = None,
) -> dict[str, bytes]:
    """Replace a dimension value in all relevant streams.

    Returns a dict of ``{stream_name: modified_bytes}`` suitable for
    passing to ``SldFile.rebuild()``.

    Pass *prior_modifications* when chaining multiple dimension changes so
    that earlier replacements are preserved::

        mods = modify_dimension_in_sld(sld, 0.004, 0.005)
        mods = modify_dimension_in_sld(sld, 0.145, 0.200, prior_modifications=mods)
        new_file = sld.rebuild(modified_streams=mods)

    Also handles derived values:
        - negative of the value (bounding box min coords)
        - half of the value (bounding box center)
    """
    replacements = [
        (old_meters, new_meters),
        (-old_meters, -new_meters),
        (old_meters / 2, new_meters / 2),
    ]

    modified: dict[str, bytes] = dict(prior_modifications) if prior_modifications else {}

    for name, stream in sld_file.streams.items():
        if not stream.data:
            continue

        # Start from prior modification if one exists, else use original
        data = modified.get(name, stream.data)
        original = data
        total_changes = 0

        for old_val, new_val in replacements:
            # LE replacements (most streams)
            data, count = replace_value(data, old_val, new_val, big_endian=False)
            total_changes += count

        # Handle Parasolid partition streams (inner double-compression, BE floats)
        if "Partition" in name:
            data = _modify_parasolid_partition(data, replacements)

        if data != original:
            modified[name] = data

    return modified


def _modify_parasolid_partition(
    outer_data: bytes,
    replacements: list[tuple[float, float]],
) -> bytes:
    """Handle the double-compressed Parasolid partition data."""
    # Find inner zlib headers
    for k in range(len(outer_data) - 1):
        if outer_data[k] == 0x78 and outer_data[k + 1] in (0x01, 0x5E, 0x9C, 0xDA):
            try:
                inner = zlib.decompress(outer_data[k:])
                if inner[:2] != b"PS":
                    continue

                # Replace BE float64 values
                for old_val, new_val in replacements:
                    inner, _ = replace_value(inner, old_val, new_val, big_endian=True)

                # Recompress inner
                inner_compressed = zlib.compress(inner, 6)

                # Reconstruct: pre-zlib header bytes + recompressed
                return outer_data[:k] + inner_compressed
            except zlib.error:
                continue

    return outer_data
