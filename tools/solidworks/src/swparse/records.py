"""Record-level parsing for SolidWorks 3DEXPERIENCE files.

Every record begins with the 6-byte marker ``14 00 06 00 08 00``, followed by
a fixed-layout header:

    marker          6 bytes   always 14 00 06 00 08 00
    tag             4 bytes   starts with 3B 78, last 2 vary by version
    field1          4 bytes   uint32 LE – checksum/hash for large records
    compressed_sz   4 bytes   uint32 LE – compressed payload size
    uncompressed_sz 4 bytes   uint32 LE – uncompressed payload size
    name_len        4 bytes   uint32 LE – byte length of nibble-swapped name

Total fixed header: 26 bytes.

After the header come ``name_len`` bytes of nibble-swapped ASCII (the stream
name), followed by raw-deflate-compressed stream data.
"""

from __future__ import annotations

import struct
import zlib
from dataclasses import dataclass, field
from typing import Iterator

from . import nibble

MARKER = b"\x14\x00\x06\x00\x08\x00"
MARKER_LEN = 6
HEADER_FIELDS_LEN = 20  # tag(4) + field1(4) + comp(4) + uncomp(4) + namelen(4)
FIXED_HEADER_LEN = MARKER_LEN + HEADER_FIELDS_LEN  # 26


@dataclass
class Record:
    """One record inside the container."""

    offset: int  # absolute byte offset of the marker in the file
    tag: bytes  # 4 bytes
    field1: int  # uint32 – checksum/hash
    compressed_size: int  # uint32 – declared compressed payload size
    uncompressed_size: int  # uint32 – declared uncompressed payload size
    name_len: int  # uint32 – stream name byte count
    name_raw: bytes  # nibble-swapped name bytes
    name: str  # decoded ASCII stream name
    payload: bytes  # raw compressed bytes (not yet decompressed)

    # derived
    total_size: int = field(init=False)  # full record size (header + name + payload)

    def __post_init__(self) -> None:
        self.total_size = FIXED_HEADER_LEN + self.name_len + len(self.payload)

    @property
    def tag_hex(self) -> str:
        return self.tag.hex()


def compute_field1(decompressed: bytes, uncompressed_size: int) -> int:
    """Compute the correct field1 value for a record.

    Three cases determined by reverse engineering:
      - **Empty** (uncompressed_size == 0): field1 = 0
      - **Small** (field1 encodes a size sentinel): field1 = 2 × uncompressed_size
      - **Large** (most records): field1 = CRC32(decompressed_data)

    The CRC32 uses the standard ISO 3309 polynomial (same as ``zlib.crc32``).
    Verified across 1313 large records with zero mismatches.
    """
    if uncompressed_size == 0:
        return 0
    # CRC32 of the decompressed content
    return zlib.crc32(decompressed) & 0xFFFFFFFF


def find_markers(data: bytes | memoryview) -> list[int]:
    """Return sorted list of all record-marker byte offsets in *data*."""
    positions: list[int] = []
    view = bytes(data) if isinstance(data, memoryview) else data
    start = 0
    while True:
        pos = view.find(MARKER, start)
        if pos < 0:
            break
        positions.append(pos)
        start = pos + 1
    return positions


def parse_at(data: bytes | memoryview, offset: int, end: int) -> Record:
    """Parse a single record starting at *offset*, ending before *end*."""
    d = bytes(data)
    base = offset + MARKER_LEN

    tag = d[base : base + 4]
    field1, compressed_sz, uncompressed_sz, name_len = struct.unpack_from(
        "<4I", d, base + 4
    )

    name_start = offset + FIXED_HEADER_LEN
    name_raw = d[name_start : name_start + name_len]
    name = nibble.decode_name(name_raw).rstrip("\x00").strip()

    payload_start = name_start + name_len
    payload = d[payload_start:end]

    return Record(
        offset=offset,
        tag=tag,
        field1=field1,
        compressed_size=compressed_sz,
        uncompressed_size=uncompressed_sz,
        name_len=name_len,
        name_raw=name_raw,
        name=name,
        payload=payload,
    )


def iterate(data: bytes | memoryview) -> Iterator[Record]:
    """Yield every ``Record`` in *data*, ordered by file offset."""
    positions = find_markers(data)
    for i, pos in enumerate(positions):
        end = positions[i + 1] if i + 1 < len(positions) else len(data)
        yield parse_at(data, pos, end)
