"""C++ class instance marker parsing for SolidWorks serialized objects.

Class instances are introduced by the 4-byte marker ``FF FF 01 00``,
followed by the class name as a length-prefixed ASCII string::

    FF FF 01 00  [name_len : uint16 LE]  [class_name : ASCII bytes]

Examples::

    ff ff 01 00 0d 00 6d 6f 45 78 74 72 75 73 69 6f 6e 5f 63
    → moExtrusion_c (13 bytes)

    ff ff 01 00 12 00 6d 6f 50 72 6f 66 69 6c 65 46 65 61 74 75 72 65 5f 63
    → moProfileFeature_c (18 bytes)

Class names typically follow patterns:
    - ``mo*_c``  – model objects (most common)
    - ``mo*_w``  – wrapper/reference objects
    - ``sg*``    – sketch geometry entities (sgArcHandle, sgLineHandle, etc.)
    - ``gc*_c``  – graphics/geometry cache objects
    - ``dm*_c``  – document management objects
    - ``ui*_c``  – UI-related objects
    - ``vis*_c`` – visual state objects
    - ``uo*_c``  – user object / tessellation data
    - Fillet_c, Chamfer_c, etc. – feature classes without prefix
"""

from __future__ import annotations

import struct
from dataclasses import dataclass
from typing import Optional

CLASS_MARKER = b"\xff\xff\x01\x00"


@dataclass(frozen=True)
class ClassInstance:
    """A serialized C++ class instance found in binary data."""

    offset: int  # byte offset of the FF FF 01 00 marker
    name: str  # class name (ASCII)
    name_len: int
    data_offset: int  # byte offset where instance data starts (after name)

    @property
    def byte_length(self) -> int:
        """Total bytes consumed by the marker + length + name."""
        return 4 + 2 + self.name_len  # marker(4) + len(2) + name


def try_read(data: bytes, offset: int) -> Optional[ClassInstance]:
    """Try to read a class marker at *offset*.  Returns ``None`` if invalid."""
    if offset + 6 > len(data):
        return None
    if data[offset : offset + 4] != CLASS_MARKER:
        return None

    name_len = struct.unpack_from("<H", data, offset + 4)[0]
    if name_len < 4 or name_len > 80:
        return None

    name_end = offset + 6 + name_len
    if name_end > len(data):
        return None

    try:
        name = data[offset + 6 : name_end].decode("ascii")
    except UnicodeDecodeError:
        return None

    # Sanity check: class names end with _c, _w, or are known prefixes
    if not (
        name.endswith("_c")
        or name.endswith("_w")
        or name.startswith("sg")
        or name.startswith("Pa")
        or name.startswith("An")
        or name.startswith("Th")
        or name.startswith("Fi")
        or name.startswith("Ch")
        or name.startswith("ed")
    ):
        return None

    return ClassInstance(
        offset=offset,
        name=name,
        name_len=name_len,
        data_offset=name_end,
    )


def find_all(data: bytes) -> list[ClassInstance]:
    """Find every class instance marker in *data*."""
    results: list[ClassInstance] = []
    i = 0
    while i < len(data) - 6:
        c = try_read(data, i)
        if c is not None:
            results.append(c)
            i = c.data_offset
        else:
            i += 1
    return results


def count_by_name(instances: list[ClassInstance]) -> dict[str, int]:
    """Return {class_name: count} from a list of instances."""
    counts: dict[str, int] = {}
    for c in instances:
        counts[c.name] = counts.get(c.name, 0) + 1
    return counts
