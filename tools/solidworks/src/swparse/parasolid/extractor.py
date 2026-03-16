"""Extract Parasolid XT binary data from SolidWorks partition streams.

The ``Contents/Config-N-Partition`` stream contains B-rep geometry stored as
Parasolid XT (Transmit) binary format.  The data is **double-compressed**:

    Outer layer: raw deflate (container-level, already handled by streams.py)
    Inner layer: standard zlib (with 78 xx header)

The inner decompressed data starts with::

    PS?: TRANSMIT FILE (partition) created by modeller version NNNNNNN

A single partition stream may contain multiple chunks:
    1. Main partition (topology + geometry)
    2. Deltas (incremental changes)

Each chunk has its own inner zlib compression.

Coordinate system:
    - All coordinates in METERS (SI units)
    - Float64 values are BIG-ENDIAN (unlike the rest of the file)
"""

from __future__ import annotations

import zlib
from dataclasses import dataclass
from typing import Optional


@dataclass
class ParasolidChunk:
    """One decompressed Parasolid XT chunk."""

    chunk_type: str  # "partition" or "deltas"
    version: str  # e.g. "2800174"
    schema: str  # e.g. "SCH_2800174_28002_13006"
    data: bytes  # full decompressed chunk including header
    inner_offset: int  # offset of the zlib header in the outer stream


def _find_zlib_headers(data: bytes) -> list[int]:
    """Find all standard zlib headers (78 xx) in *data*."""
    offsets = []
    for i in range(len(data) - 1):
        if data[i] == 0x78 and data[i + 1] in (0x01, 0x5E, 0x9C, 0xDA):
            offsets.append(i)
    return offsets


def _parse_header(data: bytes) -> tuple[str, str, str]:
    """Parse the PS header to extract chunk type, version, and schema."""
    # Header format: "PS" + padding + "?: TRANSMIT FILE (TYPE) created by modeller version VERSION"
    # followed by NUL + padding + "SCH_VERSION_VERSION_VERSION"
    chunk_type = "unknown"
    version = "unknown"
    schema = "unknown"

    text = data[:200].decode("ascii", errors="replace")

    if "(partition)" in text:
        chunk_type = "partition"
    elif "(deltas)" in text:
        chunk_type = "deltas"

    # Extract version number after "version "
    ver_marker = "version "
    ver_pos = text.find(ver_marker)
    if ver_pos >= 0:
        ver_start = ver_pos + len(ver_marker)
        ver_end = ver_start
        while ver_end < len(text) and text[ver_end].isdigit():
            ver_end += 1
        version = text[ver_start:ver_end]

    # Extract schema (starts with "SCH_")
    sch_marker = "SCH_"
    sch_pos = text.find(sch_marker)
    if sch_pos >= 0:
        sch_end = sch_pos
        while sch_end < len(text) and text[sch_end] not in ("\x00", "\xff"):
            sch_end += 1
        schema = text[sch_pos:sch_end]

    return chunk_type, version, schema


def extract(outer_data: bytes) -> list[ParasolidChunk]:
    """Extract all Parasolid XT chunks from a decompressed partition stream.

    *outer_data* should be the already-decompressed ``Contents/Config-N-Partition``
    stream (i.e., after the container-level deflate).
    """
    zlib_offsets = _find_zlib_headers(outer_data)
    chunks: list[ParasolidChunk] = []

    for off in zlib_offsets:
        try:
            inner = zlib.decompress(outer_data[off:])
        except zlib.error:
            continue

        if len(inner) < 10:
            continue

        # Verify it's Parasolid data
        if inner[:2] != b"PS":
            continue

        chunk_type, version, schema = _parse_header(inner)
        chunks.append(
            ParasolidChunk(
                chunk_type=chunk_type,
                version=version,
                schema=schema,
                data=inner,
                inner_offset=off,
            )
        )

    return chunks


def extract_from_sld(sld_file) -> list[ParasolidChunk]:
    """Convenience: extract Parasolid chunks from a parsed ``SldFile``."""
    all_chunks: list[ParasolidChunk] = []
    for name, stream in sld_file.streams.items():
        if "Partition" in name and stream.data and len(stream.data) > 100:
            all_chunks.extend(extract(stream.data))
    return all_chunks
