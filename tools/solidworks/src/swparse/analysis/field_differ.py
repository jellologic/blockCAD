"""Cross-file binary comparison and entropy analysis for class instances.

Compares the same class's binary payload across multiple files to identify:
  - Constant bytes (flags, version markers)
  - Variable bytes (dimensions, IDs, pointers)
  - Float64 fields (coordinates in meters)
  - String fields (FF FE FF markers)
  - Embedded sub-class fields (FF FF 01 00 markers)
"""

from __future__ import annotations

import math
import struct
from dataclasses import dataclass

from .class_extractor import ExtractedInstance


@dataclass
class FieldCandidate:
    """A candidate field identified by entropy analysis."""

    offset: int
    length: int
    field_type: str  # "const", "float64", "uint32", "string", "class_ref", "unknown"
    entropy: float  # 0.0 = constant, higher = more variable
    description: str
    sample_values: list  # sample values from different files


@dataclass
class FieldMap:
    """Entropy-based field map for a class."""

    class_name: str
    instance_count: int
    min_payload_size: int
    max_payload_size: int
    candidates: list[FieldCandidate]


def _byte_entropy(values: list[int]) -> float:
    """Shannon entropy of a list of byte values (0-8 bits)."""
    if not values:
        return 0.0
    freq: dict[int, int] = {}
    for v in values:
        freq[v] = freq.get(v, 0) + 1
    n = len(values)
    ent = 0.0
    for count in freq.values():
        p = count / n
        if p > 0:
            ent -= p * math.log2(p)
    return ent


def analyze(
    instances: list[ExtractedInstance],
    *,
    max_scan_bytes: int = 200,
) -> FieldMap:
    """Analyze field patterns across multiple instances of the same class."""
    if not instances:
        raise ValueError("No instances to analyze")

    class_name = instances[0].class_name
    min_size = min(len(inst.data) for inst in instances)
    max_size = max(len(inst.data) for inst in instances)
    scan_end = min(min_size, max_scan_bytes)

    candidates: list[FieldCandidate] = []
    offset = 0

    while offset < scan_end:
        # Check for string marker at this offset
        string_count = sum(
            1 for inst in instances
            if offset + 3 <= len(inst.data) and inst.data[offset : offset + 3] == b"\xff\xfe\xff"
        )
        if string_count > len(instances) * 0.8:
            candidates.append(
                FieldCandidate(offset, -1, "string", 0.0, "UTF-16LE string (FFFEFF)", [])
            )
            offset += 4  # skip marker + length byte; actual string is variable
            continue

        # Check for sub-class marker
        class_count = sum(
            1 for inst in instances
            if offset + 4 <= len(inst.data) and inst.data[offset : offset + 4] == b"\xff\xff\x01\x00"
        )
        if class_count > len(instances) * 0.8:
            candidates.append(
                FieldCandidate(offset, -1, "class_ref", 0.0, "Embedded class (FFFF0100)", [])
            )
            break  # can't continue fixed-offset scanning past variable-length child

        # Check for sub-record marker
        subrec_count = sum(
            1 for inst in instances
            if offset + 5 <= len(inst.data) and inst.data[offset : offset + 5] == b"\xff\xff\x1f\x00\x03"
        )
        if subrec_count > len(instances) * 0.8:
            candidates.append(
                FieldCandidate(offset, 5, "sub_record", 0.0, "Sub-record marker (FFFF1F0003)", [])
            )
            break

        # Compute per-byte entropy
        byte_values = [inst.data[offset] for inst in instances if offset < len(inst.data)]
        ent = _byte_entropy(byte_values)

        # Try float64 at this offset
        if offset + 8 <= scan_end:
            f64_values = []
            for inst in instances:
                if offset + 8 <= len(inst.data):
                    val = struct.unpack_from("<d", inst.data, offset)[0]
                    f64_values.append(val)

            if f64_values:
                all_sensible = all(
                    abs(v) < 1e6 and (abs(v) > 1e-10 or v == 0.0) and not math.isnan(v)
                    for v in f64_values
                )
                if all_sensible and len(set(f64_values)) > 1:
                    mm_vals = [v * 1000 for v in f64_values[:5]]
                    candidates.append(
                        FieldCandidate(
                            offset, 8, "float64",
                            ent, f"Likely dimension (mm: {mm_vals})",
                            f64_values[:5],
                        )
                    )
                    offset += 8
                    continue

        # Try uint32 at this offset
        if offset + 4 <= scan_end:
            u32_values = []
            for inst in instances:
                if offset + 4 <= len(inst.data):
                    val = struct.unpack_from("<I", inst.data, offset)[0]
                    u32_values.append(val)

            if u32_values and ent < 0.5:
                # Constant uint32
                candidates.append(
                    FieldCandidate(
                        offset, 4, "const",
                        ent, f"Constant: 0x{u32_values[0]:08x}",
                        u32_values[:5],
                    )
                )
                offset += 4
                continue

        # Default: single byte
        if ent == 0:
            candidates.append(
                FieldCandidate(offset, 1, "const", 0.0, f"Constant: 0x{byte_values[0]:02x}", [])
            )
        else:
            candidates.append(
                FieldCandidate(offset, 1, "unknown", ent, f"Variable byte (entropy={ent:.2f})", [])
            )
        offset += 1

    return FieldMap(
        class_name=class_name,
        instance_count=len(instances),
        min_payload_size=min_size,
        max_payload_size=max_size,
        candidates=candidates,
    )


def format_field_map(fm: FieldMap) -> str:
    """Format a FieldMap as a readable table."""
    lines = [
        f"Class: {fm.class_name}",
        f"Instances: {fm.instance_count}",
        f"Payload size: {fm.min_payload_size}-{fm.max_payload_size} bytes",
        "",
        f"{'Offset':<8s} {'Len':>4s} {'Type':<12s} {'Entropy':>8s}  {'Description'}",
        "-" * 72,
    ]
    for c in fm.candidates:
        lines.append(
            f"  0x{c.offset:04x}  {c.length:>4d}  {c.field_type:<12s}  {c.entropy:>7.2f}   {c.description}"
        )
    return "\n".join(lines)
