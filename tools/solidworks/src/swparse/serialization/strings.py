"""UTF-16LE string decoding for SolidWorks serialized objects.

Strings are encoded as::

    FF FE FF  [char_count : uint8]  [chars : char_count * 2 bytes UTF-16LE]

The ``FF FE FF`` prefix acts as a string marker (not a true BOM, despite
containing the UTF-16LE BOM sequence ``FF FE``).

Examples::

    ff fe ff 07 53 00 6b 00 65 00 74 00 63 00 68 00 31 00
    → "Sketch1" (7 chars)

    ff fe ff 00
    → "" (empty string)
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

STRING_MARKER = b"\xff\xfe\xff"


@dataclass(frozen=True)
class DecodedString:
    offset: int  # byte offset of the FF FE FF marker
    value: str
    byte_length: int  # total consumed bytes (marker + length + data)


def try_read(data: bytes, offset: int) -> Optional[DecodedString]:
    """Try to read a string at *offset*.  Returns ``None`` if the marker
    is not present or the data is truncated."""
    if offset + 4 > len(data):
        return None
    if data[offset : offset + 3] != STRING_MARKER:
        return None

    char_count = data[offset + 3]
    str_bytes = char_count * 2
    start = offset + 4
    end = start + str_bytes

    if end > len(data):
        return None

    try:
        value = data[start:end].decode("utf-16-le")
    except UnicodeDecodeError:
        value = data[start:end].decode("utf-16-le", errors="replace")

    return DecodedString(
        offset=offset,
        value=value,
        byte_length=4 + str_bytes,  # 3 (marker) + 1 (length) + data
    )


def find_all(data: bytes) -> list[DecodedString]:
    """Find every string in *data*."""
    results: list[DecodedString] = []
    i = 0
    while i < len(data) - 3:
        s = try_read(data, i)
        if s is not None:
            results.append(s)
            i += s.byte_length
        else:
            i += 1
    return results
