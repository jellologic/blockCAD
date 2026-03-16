"""Top-level API for reading and writing SolidWorks 3DEXPERIENCE files.

Usage::

    from swparse import SldFile

    sld = SldFile.open("part.SLDPRT")
    print(sld.header)
    for name, stream in sld.streams.items():
        print(name, stream.size, stream.content_type)

    # Extract a PNG preview
    png = sld.streams.get("PreviewPNG")
    if png:
        Path("preview.png").write_bytes(png.data)
"""

from __future__ import annotations

import struct
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, Union

from . import header as hdr
from . import nibble
from . import records as rec
from . import streams as stm


@dataclass
class SldFile:
    """Parsed SolidWorks file."""

    path: Optional[Path]
    header: hdr.FileHeader
    records: list[rec.Record]
    streams: dict[str, stm.Stream]  # best version per stream name
    raw: bytes  # original file bytes

    # ------------------------------------------------------------------ #
    #  Construction
    # ------------------------------------------------------------------ #

    @classmethod
    def open(cls, path: Union[str, Path]) -> "SldFile":
        """Parse *path* and return a fully-loaded ``SldFile``."""
        p = Path(path)
        data = p.read_bytes()
        return cls.from_bytes(data, path=p)

    @classmethod
    def from_bytes(cls, data: bytes, *, path: Optional[Path] = None) -> "SldFile":
        file_header = hdr.parse(data)
        all_records = list(rec.iterate(data))

        # Decompress streams, keeping the largest version per name
        best: dict[str, stm.Stream] = {}
        for r in all_records:
            decompressed = stm.decompress(r)
            if decompressed is None:
                continue
            ct = stm.classify(decompressed)
            s = stm.Stream(name=r.name, data=decompressed, content_type=ct, record=r)
            prev = best.get(r.name)
            if prev is None or s.size > prev.size:
                best[r.name] = s

        return cls(
            path=path,
            header=file_header,
            records=all_records,
            streams=best,
            raw=data,
        )

    # ------------------------------------------------------------------ #
    #  Convenience
    # ------------------------------------------------------------------ #

    def stream_names(self) -> list[str]:
        return sorted(self.streams.keys())

    def get_xml(self, name: str) -> Optional[str]:
        """Return decoded XML text for *name*, or ``None``."""
        s = self.streams.get(name)
        return s.text if s else None

    def get_bytes(self, name: str) -> Optional[bytes]:
        s = self.streams.get(name)
        return s.data if s else None

    def get_preview_png(self) -> Optional[bytes]:
        """Return the embedded PNG thumbnail, if present."""
        for name, s in self.streams.items():
            if "PreviewPNG" in name and s.content_type == stm.ContentType.PNG:
                return s.data
        return None

    # ------------------------------------------------------------------ #
    #  Write-back
    # ------------------------------------------------------------------ #

    def rebuild(
        self,
        modified_streams: Optional[dict[str, bytes]] = None,
    ) -> bytes:
        """Reconstruct the file from records, optionally replacing stream data.

        *modified_streams* maps stream names to new **uncompressed** bytes.
        Records whose stream name matches will be recompressed; all others
        are kept as-is from the original file.
        """
        modified_streams = modified_streams or {}
        parts: list[bytes] = [self.header.raw]

        for r in self.records:
            if r.name in modified_streams:
                new_data = modified_streams[r.name]
                new_payload = stm.compress(new_data)
                new_field1 = rec.compute_field1(new_data, len(new_data))
                new_header = (
                    rec.MARKER
                    + r.tag
                    + struct.pack(
                        "<4I",
                        new_field1,
                        len(new_payload),
                        len(new_data),
                        r.name_len,
                    )
                )
                parts.append(new_header + r.name_raw + new_payload)
            else:
                # Use the original raw bytes
                end = r.offset + r.total_size
                parts.append(self.raw[r.offset : end])

        return b"".join(parts)
