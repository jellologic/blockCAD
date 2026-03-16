"""Registry of known stream names and their purposes.

Compiled from reverse-engineering 38 SLDPRT part files and 1 SLDASM assembly
from a 3-wheel bicycle model (SolidWorks 2015 / v23.0 / Parasolid 28.0).
"""

from dataclasses import dataclass
from enum import Enum, auto


class StreamCategory(Enum):
    GEOMETRY = auto()
    FEATURE = auto()
    DISPLAY = auto()
    PREVIEW = auto()
    CONFIG = auto()
    XML_METADATA = auto()
    THIRD_PARTY = auto()
    VERSION = auto()
    ASSEMBLY = auto()
    OTHER = auto()


@dataclass(frozen=True)
class StreamInfo:
    name: str
    category: StreamCategory
    description: str
    typical_content: str  # "xml", "binary", "png", "bmp"


KNOWN_STREAMS: dict[str, StreamInfo] = {
    # ── Geometry / Features ──────────────────────────────────────────
    "Contents/Config-0-Partition": StreamInfo(
        "Contents/Config-0-Partition",
        StreamCategory.GEOMETRY,
        "B-rep solid geometry in Parasolid XT binary format. Double-compressed: "
        "outer raw deflate + inner standard zlib. Contains topology (body → region "
        "→ shell → face → loop → edge → vertex) and geometry (surfaces, curves, points). "
        "Coordinates in meters, float64 big-endian.",
        "binary",
    ),
    "Contents/Config-0-ResolvedFeatures": StreamInfo(
        "Contents/Config-0-ResolvedFeatures",
        StreamCategory.FEATURE,
        "Full parametric feature tree as serialized C++ objects. Contains sketch "
        "profiles (moProfileFeature_c), extrusions (moExtrusion_c), fillets, "
        "chamfers, mirrors, patterns, and all dimension parameters. Class markers "
        "use FFFF0100 prefix, strings use FFFEFF prefix.",
        "binary",
    ),
    "Contents/Config-0": StreamInfo(
        "Contents/Config-0",
        StreamCategory.CONFIG,
        "Main configuration data. Contains the root moPart_c object with "
        "document header, units table, material properties, view orientation, "
        "lighting, and bounding box. Bounding box stored as 6 float64 LE values "
        "in meters: half_height, +radius, +radius, height, -radius, -radius.",
        "binary",
    ),
    "Contents/Config-0-GhostPartition": StreamInfo(
        "Contents/Config-0-GhostPartition",
        StreamCategory.GEOMETRY,
        "Hidden/suppressed geometry partition.",
        "binary",
    ),
    "Contents/Config-0-LWDATA": StreamInfo(
        "Contents/Config-0-LWDATA",
        StreamCategory.DISPLAY,
        "Lightweight display data including cross-hatch patterns and simplified "
        "geometry for fast rendering. Contains dimension values.",
        "binary",
    ),
    "Contents/DisplayLists": StreamInfo(
        "Contents/DisplayLists",
        StreamCategory.DISPLAY,
        "Tessellated triangle mesh data for OpenGL display. Contains dimension "
        "values for vertex positions.",
        "binary",
    ),
    "Contents/CMgr": StreamInfo(
        "Contents/CMgr",
        StreamCategory.CONFIG,
        "Configuration Manager data (moConfigurationMgr_c).",
        "binary",
    ),
    "Contents/CMgrHdr2": StreamInfo(
        "Contents/CMgrHdr2",
        StreamCategory.CONFIG,
        "Configuration Manager header. Contains configuration names in UTF-16 "
        "(e.g., 'Default<As Machined>').",
        "binary",
    ),
    "Contents/CnfgObjs": StreamInfo(
        "Contents/CnfgObjs",
        StreamCategory.CONFIG,
        "Configuration objects (moPartConfigObject_c). Per-body chooser data.",
        "binary",
    ),
    "Contents/Definition": StreamInfo(
        "Contents/Definition",
        StreamCategory.CONFIG,
        "Part definition including coordinate transforms and reference data.",
        "binary",
    ),
    "Contents/MBLinkData": StreamInfo(
        "Contents/MBLinkData",
        StreamCategory.OTHER,
        "Model-based link data.",
        "binary",
    ),
    "Contents/OleItems": StreamInfo(
        "Contents/OleItems",
        StreamCategory.OTHER,
        "Embedded OLE items (typically empty/minimal).",
        "binary",
    ),
    "Contents/User Units Table": StreamInfo(
        "Contents/User Units Table",
        StreamCategory.CONFIG,
        "User-defined unit system configuration.",
        "binary",
    ),
    "Contents/View Orientation Data": StreamInfo(
        "Contents/View Orientation Data",
        StreamCategory.DISPLAY,
        "Saved viewport orientations.",
        "binary",
    ),
    "Contents/eModelLic": StreamInfo(
        "Contents/eModelLic",
        StreamCategory.OTHER,
        "Model licensing information.",
        "binary",
    ),
    # ── Preview ──────────────────────────────────────────────────────
    "PreviewPNG": StreamInfo(
        "PreviewPNG",
        StreamCategory.PREVIEW,
        "PNG thumbnail preview image. Directly extractable as a .png file.",
        "png",
    ),
    "Preview": StreamInfo(
        "Preview",
        StreamCategory.PREVIEW,
        "BMP preview image (Windows bitmap format).",
        "bmp",
    ),
    # ── Headers ──────────────────────────────────────────────────────
    "Header2": StreamInfo(
        "Header2",
        StreamCategory.CONFIG,
        "Document header with moHeader_c root object. Contains bounding box, "
        "creation/modification stamps, and part name.",
        "binary",
    ),
    "ModelStamps": StreamInfo(
        "ModelStamps",
        StreamCategory.VERSION,
        "Model timestamps (12 bytes).",
        "binary",
    ),
    # ── XML Metadata (OPC format, same as .docx) ────────────────────
    "[Content_Types].xml": StreamInfo(
        "[Content_Types].xml",
        StreamCategory.XML_METADATA,
        "OPC (Open Packaging Convention) content types. Maps file extensions "
        "to MIME types. Same format as .docx/.xlsx files.",
        "xml",
    ),
    "_rels/.rels": StreamInfo(
        "_rels/.rels",
        StreamCategory.XML_METADATA,
        "OPC relationships. Links to app.xml, core.xml, custom.xml.",
        "xml",
    ),
    "docProps/app.xml": StreamInfo(
        "docProps/app.xml",
        StreamCategory.XML_METADATA,
        "Application properties: SolidWorks version (e.g., 23.0000), "
        "company name, total editing time.",
        "xml",
    ),
    "docProps/core.xml": StreamInfo(
        "docProps/core.xml",
        StreamCategory.XML_METADATA,
        "Core document properties: author, creation date, modification date. "
        "Uses Dublin Core metadata terms.",
        "xml",
    ),
    "docProps/custom.xml": StreamInfo(
        "docProps/custom.xml",
        StreamCategory.XML_METADATA,
        "Custom properties including unit system settings (linear units, "
        "angular units, decimal places, etc.).",
        "xml",
    ),
    "docProps/ISolidWorksInformation.xml": StreamInfo(
        "docProps/ISolidWorksInformation.xml",
        StreamCategory.XML_METADATA,
        "SolidWorks-specific metadata: file name, folder, creation date, "
        "configuration name, author, keywords.",
        "xml",
    ),
    "swXmlContents/Features": StreamInfo(
        "swXmlContents/Features",
        StreamCategory.XML_METADATA,
        "Minimal feature tree in XML: file references, configurations, and "
        "model list. Does NOT contain actual feature geometry.",
        "xml",
    ),
    "swXmlContents/KeyWords": StreamInfo(
        "swXmlContents/KeyWords",
        StreamCategory.XML_METADATA,
        "Feature names, types, and dimension values in XML. The richest "
        "human-readable source for parametric data. Contains Extrusion, "
        "Sketch, Fillet, Revolve elements with Dimension children.",
        "xml",
    ),
    # ── Assembly-specific ────────────────────────────────────────────
    "swXmlContents/COMPINSTANCETREE": StreamInfo(
        "swXmlContents/COMPINSTANCETREE",
        StreamCategory.ASSEMBLY,
        "Component instance tree for assemblies. Lists all part files with "
        "their paths, IDs, and 4x4 transform matrices. Essential for "
        "understanding assembly structure.",
        "xml",
    ),
    "Contents/Config-0-MatesList": StreamInfo(
        "Contents/Config-0-MatesList",
        StreamCategory.ASSEMBLY,
        "Assembly constraints (mates): Coincident, Concentric, Perpendicular, "
        "etc. Binary serialized with moMateCoincident, moMateConcentric classes.",
        "binary",
    ),
    "Contents/Config-0-Attachment": StreamInfo(
        "Contents/Config-0-Attachment",
        StreamCategory.ASSEMBLY,
        "Assembly attachment data.",
        "binary",
    ),
    # ── Third-party ──────────────────────────────────────────────────
    "ThirdPty/Animator": StreamInfo(
        "ThirdPty/Animator",
        StreamCategory.THIRD_PARTY,
        "Motion study / animation data (moAnimationManager_c).",
        "binary",
    ),
    "ThirdPtyStore/VisualStates": StreamInfo(
        "ThirdPtyStore/VisualStates",
        StreamCategory.THIRD_PARTY,
        "Display state visual properties (visStateKeyData_c).",
        "binary",
    ),
    "SWIFT/Config0-Schema1": StreamInfo(
        "SWIFT/Config0-Schema1",
        StreamCategory.THIRD_PARTY,
        "SWIFT schema data (moSwiftSchema_c).",
        "binary",
    ),
    "SwDocContentMgr/SwDocContentMgrInfo": StreamInfo(
        "SwDocContentMgr/SwDocContentMgrInfo",
        StreamCategory.CONFIG,
        "Document content manager info (moBCModeInfo_c). Configuration names "
        "and material references.",
        "binary",
    ),
    # ── Version tracking ─────────────────────────────────────────────
    "_MO_VERSION_9000/Biography": StreamInfo(
        "_MO_VERSION_9000/Biography",
        StreamCategory.VERSION,
        "File biography (moBiography_c) with modification history.",
        "binary",
    ),
    "_MO_VERSION_9000/History": StreamInfo(
        "_MO_VERSION_9000/History",
        StreamCategory.VERSION,
        "Version history (moVersionHistory_c).",
        "binary",
    ),
    "_DL_VERSION_9000/DLUpdateStamp": StreamInfo(
        "_DL_VERSION_9000/DLUpdateStamp",
        StreamCategory.VERSION,
        "Display list update timestamp.",
        "binary",
    ),
}
