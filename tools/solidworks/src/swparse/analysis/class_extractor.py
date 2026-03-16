"""Extract raw binary payloads of class instances from ResolvedFeatures streams.

Given a target class name, this module locates every instance of that class
across one or more files, extracts the raw bytes between the class marker and
the next class marker, and returns them for cross-file comparison.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from ..container import SldFile
from ..serialization.classes import ClassInstance, find_all as find_classes


@dataclass
class ExtractedInstance:
    """One extracted class instance with its raw payload."""

    source_file: str
    class_name: str
    offset: int  # absolute offset in the stream
    data: bytes  # raw bytes from data_offset to next class marker
    prev_class: Optional[str]  # class name of preceding instance
    next_class: Optional[str]  # class name of following instance
    stream_name: str


def extract_class_instances(
    sld: SldFile,
    target_class: str,
    *,
    stream_filter: str = "ResolvedFeatures",
) -> list[ExtractedInstance]:
    """Extract all instances of *target_class* from *sld*.

    Searches streams whose name contains *stream_filter*.
    """
    results: list[ExtractedInstance] = []
    source = str(sld.path) if sld.path else "<bytes>"

    for sname, stream in sld.streams.items():
        if stream_filter and stream_filter not in sname:
            continue
        if not stream.data or len(stream.data) < 10:
            continue

        all_instances = find_classes(stream.data)
        for i, inst in enumerate(all_instances):
            if inst.name != target_class:
                continue

            # Payload extends from data_offset to start of next class marker
            if i + 1 < len(all_instances):
                end = all_instances[i + 1].offset
            else:
                end = len(stream.data)

            prev_cls = all_instances[i - 1].name if i > 0 else None
            next_cls = all_instances[i + 1].name if i + 1 < len(all_instances) else None

            results.append(
                ExtractedInstance(
                    source_file=source,
                    class_name=target_class,
                    offset=inst.offset,
                    data=stream.data[inst.data_offset : end],
                    prev_class=prev_cls,
                    next_class=next_cls,
                    stream_name=sname,
                )
            )

    return results


def extract_from_directory(
    directory: str | Path,
    target_class: str,
    *,
    stream_filter: str = "ResolvedFeatures",
    suffix: str = ".SLDPRT",
) -> list[ExtractedInstance]:
    """Extract instances of *target_class* from all files in *directory*."""
    d = Path(directory)
    results: list[ExtractedInstance] = []

    for fpath in sorted(d.iterdir()):
        if not fpath.name.endswith(suffix):
            continue
        try:
            sld = SldFile.open(fpath)
            results.extend(
                extract_class_instances(sld, target_class, stream_filter=stream_filter)
            )
        except Exception:
            pass  # skip files that fail to parse

    return results
