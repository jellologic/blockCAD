"""Primitive value readers for the SolidWorks binary serialization format.

All multi-byte integers and floats in the serialized object streams use
**little-endian** byte order.  Coordinates are stored in **meters** (SI).

Parasolid XT data is the exception – it uses **big-endian** float64.
"""

from __future__ import annotations

import struct
from typing import Tuple

# Type alias for (value, new_offset) return pairs
ReadResult = Tuple


def read_u8(data: bytes, offset: int) -> tuple[int, int]:
    return data[offset], offset + 1


def read_u16(data: bytes, offset: int) -> tuple[int, int]:
    return struct.unpack_from("<H", data, offset)[0], offset + 2


def read_u32(data: bytes, offset: int) -> tuple[int, int]:
    return struct.unpack_from("<I", data, offset)[0], offset + 4


def read_i32(data: bytes, offset: int) -> tuple[int, int]:
    return struct.unpack_from("<i", data, offset)[0], offset + 4


def read_f64(data: bytes, offset: int) -> tuple[float, int]:
    """Read a little-endian float64 (used in Config-0 / ResolvedFeatures)."""
    return struct.unpack_from("<d", data, offset)[0], offset + 8


def read_f64_be(data: bytes, offset: int) -> tuple[float, int]:
    """Read a big-endian float64 (used in Parasolid XT data)."""
    return struct.unpack_from(">d", data, offset)[0], offset + 8


def read_timestamp(data: bytes, offset: int) -> tuple[int, int]:
    """Read a Unix timestamp (uint32 LE)."""
    return read_u32(data, offset)


def pack_f64(value: float) -> bytes:
    """Pack a float64 as little-endian bytes."""
    return struct.pack("<d", value)


def pack_f64_be(value: float) -> bytes:
    """Pack a float64 as big-endian bytes (for Parasolid XT)."""
    return struct.pack(">d", value)
