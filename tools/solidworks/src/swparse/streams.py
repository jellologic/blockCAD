"""Stream decompression and content classification.

Every record's payload is raw-deflate compressed (zlib wbits=-15, no header).
Some stream names appear multiple times with different tags – these represent
different versions/snapshots of the same logical stream.  By convention we
keep the version with the largest decompressed size.
"""

from __future__ import annotations

import zlib
from dataclasses import dataclass
from enum import Enum, auto
from typing import Optional

from .records import Record


class ContentType(Enum):
    XML = auto()
    BINARY = auto()
    PNG = auto()
    BMP = auto()
    EMPTY = auto()
    UNKNOWN = auto()


@dataclass
class Stream:
    """A decompressed stream with metadata."""

    name: str
    data: bytes
    content_type: ContentType
    record: Record  # source record

    @property
    def size(self) -> int:
        return len(self.data)

    @property
    def text(self) -> Optional[str]:
        """Return decoded text for XML streams, else ``None``."""
        if self.content_type == ContentType.XML:
            return self.data.decode("utf-8", errors="replace")
        return None


def decompress(record: Record) -> Optional[bytes]:
    """Decompress a record's payload.  Returns ``None`` on failure."""
    if len(record.payload) < 2:
        return None
    try:
        return zlib.decompress(record.payload, -15)
    except zlib.error:
        pass
    # Fallback: try standard zlib (with header)
    try:
        return zlib.decompress(record.payload)
    except zlib.error:
        return None


def classify(data: bytes) -> ContentType:
    """Guess the content type of decompressed data."""
    if not data:
        return ContentType.EMPTY
    # Some XML streams have a leading garbage byte before <?xml
    if data[:5] == b"<?xml" or data[:1] == b"<" or b"<?xml" in data[:10]:
        return ContentType.XML
    if data[:4] == b"\x89PNG":
        return ContentType.PNG
    if data[:2] == b"BM":
        return ContentType.BMP
    if len(data) < 4:
        return ContentType.UNKNOWN
    return ContentType.BINARY


def compress(data: bytes, level: int = 6) -> bytes:
    """Compress *data* with raw deflate (no header), matching the container format."""
    c = zlib.compressobj(level, zlib.DEFLATED, -15)
    return c.compress(data) + c.flush()
