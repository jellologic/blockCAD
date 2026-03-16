"""Structured field layout definitions for reverse-engineered class instances.

As class layouts are decoded through cross-file analysis, they are recorded
here so that the parser can read individual fields rather than treating
class payloads as opaque blobs.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class FieldDef:
    """Definition of one field within a serialized class."""

    offset: int | str  # int for fixed offset, "variable" after variable-length field
    name: str
    field_type: str  # "uint8", "uint16", "uint32", "int32", "float64", "string", "class_ref", "bytes(N)"
    description: str = ""
    confidence: float = 0.0  # 0.0-1.0
    evidence: list[str] = field(default_factory=list)


@dataclass
class ClassLayout:
    """Complete layout of a serialized class."""

    class_name: str
    version: int | None = None  # if layout varies by version
    fixed_prefix_size: int = 0  # bytes before first variable-length field
    fields: list[FieldDef] = field(default_factory=list)
    children: list[str] = field(default_factory=list)  # embedded child class names
    notes: str = ""


# ── Known layouts (populated as reverse engineering progresses) ──────────

KNOWN_LAYOUTS: dict[str, ClassLayout] = {
    "moLengthParameter_c": ClassLayout(
        class_name="moLengthParameter_c",
        notes="Length dimension parameter. Contains the dimension value as float64 in meters. "
              "The dimension name (e.g., 'D1') appears as a UTF-16LE string after the value. "
              "Field offsets are approximate and need cross-file validation.",
        fields=[
            FieldDef(0, "flags", "bytes(2)", "Unknown flags/version", confidence=0.3),
            FieldDef(2, "marker1", "bytes(2)", "Possible sub-marker (0x4680 observed)", confidence=0.3),
            FieldDef(4, "name_string", "string", "Dimension name as FFFEFF + UTF-16LE (e.g., 'D1')", confidence=0.8),
            # After the variable-length name, there are more fields including the actual value.
            # The float64 value appears at a variable offset depending on name length.
        ],
    ),
    "moEndSpec_c": ClassLayout(
        class_name="moEndSpec_c",
        notes="Extrusion end condition. Expected to contain an end-condition enum "
              "(blind, through-all, up-to-surface) and a reference to a depth parameter.",
        fields=[],
    ),
    "moExtrusion_c": ClassLayout(
        class_name="moExtrusion_c",
        notes="Boss or cut extrusion feature. Contains references to profile sketch, "
              "end conditions, and direction. Feature name appears as UTF-16LE string.",
        fields=[
            FieldDef(0, "flags", "bytes(2)", "Unknown flags/version", confidence=0.3),
            FieldDef(2, "marker1", "bytes(2)", "Sub-marker (0x4680 observed)", confidence=0.3),
            FieldDef(4, "name_string", "string", "Feature name (e.g., 'Boss-Extrude1')", confidence=0.8),
        ],
        children=["moPerBodyChooserData_c", "moEndSpec_c", "moFromEndSpec_c"],
    ),
    "moRefPlane_c": ClassLayout(
        class_name="moRefPlane_c",
        notes="Reference plane. The three default planes (Front/Top/Right) have well-known "
              "orientations. Custom planes may store origin + normal vectors.",
        fields=[],
    ),
}


def get_layout(class_name: str) -> ClassLayout | None:
    """Return the known layout for *class_name*, or ``None``."""
    return KNOWN_LAYOUTS.get(class_name)
