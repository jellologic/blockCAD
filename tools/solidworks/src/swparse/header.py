"""Parse the variable-length file header of SolidWorks 3DEXPERIENCE files.

Header layout (14-22 bytes, variable):
    [checksum : 4 bytes]
    [version  : 4 bytes, big-endian, always 0x00000004]
    [type_id  : remaining bytes until first record marker]

The header length varies per file because the type_id field has no fixed size.
We locate the end of the header by finding the first record marker.
"""

from __future__ import annotations

import struct
from dataclasses import dataclass

RECORD_MARKER = b"\x14\x00\x06\x00\x08\x00"
EXPECTED_VERSION = 4


@dataclass(frozen=True)
class FileHeader:
    """Parsed file header."""

    checksum: bytes  # 4 bytes
    version: int  # always 4
    type_id: bytes  # variable length
    raw: bytes  # full raw header bytes
    size: int  # total header byte length

    @property
    def checksum_hex(self) -> str:
        return self.checksum.hex()

    @property
    def type_id_hex(self) -> str:
        return self.type_id.hex()


def parse(data: bytes | memoryview) -> FileHeader:
    """Parse the file header from the start of *data*.

    Raises ``ValueError`` if the version sentinel is not found or the
    record marker is missing.
    """
    if len(data) < 14:
        raise ValueError("File too short for a valid header")

    # Bytes 0-3: checksum
    checksum = bytes(data[0:4])

    # Bytes 4-7: version (big-endian uint32, should be 4)
    version = struct.unpack(">I", data[4:8])[0]
    if version != EXPECTED_VERSION:
        raise ValueError(
            f"Unexpected version {version:#010x} (expected {EXPECTED_VERSION:#010x})"
        )

    # Find the first record marker to determine where the header ends
    marker_pos = bytes(data).find(RECORD_MARKER)
    if marker_pos < 0:
        raise ValueError("No record marker found in file")
    if marker_pos < 8:
        raise ValueError(f"Record marker at offset {marker_pos} overlaps header fields")

    # Everything between the version field and the marker is the type_id
    type_id = bytes(data[8:marker_pos])

    return FileHeader(
        checksum=checksum,
        version=version,
        type_id=type_id,
        raw=bytes(data[:marker_pos]),
        size=marker_pos,
    )
