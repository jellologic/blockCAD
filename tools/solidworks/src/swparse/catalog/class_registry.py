"""Registry of all 317 known serialized C++ classes.

Organized by functional category. Each entry records which category the class
belongs to and a brief description of its purpose.
"""

from dataclasses import dataclass
from enum import Enum, auto


class ClassCategory(Enum):
    SKETCH = auto()
    FEATURE = auto()
    DIMENSION = auto()
    GEOMETRY_REF = auto()
    ASSEMBLY = auto()
    INFRASTRUCTURE = auto()
    OTHER = auto()


@dataclass(frozen=True)
class ClassInfo:
    name: str
    category: ClassCategory
    description: str


def _build_registry() -> dict[str, ClassInfo]:
    """Build the class registry. Called once at import time."""
    entries: list[tuple[str, ClassCategory, str]] = [
        # ── Sketch ──
        ("sgArcHandle", ClassCategory.SKETCH, "Arc or full circle sketch entity"),
        ("sgLineHandle", ClassCategory.SKETCH, "Line segment sketch entity"),
        ("sgPointHandle", ClassCategory.SKETCH, "Point sketch entity"),
        ("sgSplineHandle", ClassCategory.SKETCH, "Spline curve sketch entity"),
        ("sgCircleDim", ClassCategory.SKETCH, "Circle dimension constraint"),
        ("sgSketch", ClassCategory.SKETCH, "Root sketch container"),
        ("sgBlock", ClassCategory.SKETCH, "Sketch block entity"),
        ("sgExtEnt_c", ClassCategory.SKETCH, "External entity reference"),
        ("moProfileFeature_c", ClassCategory.SKETCH, "2D sketch profile feature"),
        ("mo3DProfileFeature_c", ClassCategory.SKETCH, "3D sketch profile feature"),
        ("moSketchChain_c", ClassCategory.SKETCH, "Chain of connected sketch entities"),
        ("moSketchExtRef_w", ClassCategory.SKETCH, "External sketch reference"),
        ("moSketchBlockMgr_c", ClassCategory.SKETCH, "Sketch block manager"),
        ("moOriginProfileFeature_c", ClassCategory.SKETCH, "Origin point feature"),
        ("moCompSketchEntHandle_c", ClassCategory.SKETCH, "Component sketch entity handle"),
        # ── Features ──
        ("moExtrusion_c", ClassCategory.FEATURE, "Boss or cut extrusion"),
        ("moRevolution_c", ClassCategory.FEATURE, "Revolve feature"),
        ("moSweep_c", ClassCategory.FEATURE, "Sweep feature"),
        ("moSweepCut_c", ClassCategory.FEATURE, "Cut sweep"),
        ("Fillet_c", ClassCategory.FEATURE, "Fillet (round) feature"),
        ("Chamfer_c", ClassCategory.FEATURE, "Chamfer feature"),
        ("moMirrorPattern_c", ClassCategory.FEATURE, "Mirror pattern"),
        ("moCirPattern_c", ClassCategory.FEATURE, "Circular pattern"),
        ("moCombineBodies_c", ClassCategory.FEATURE, "Combine bodies"),
        ("moMoveCopyBody_c", ClassCategory.FEATURE, "Move/copy body"),
        ("moEndSpec_c", ClassCategory.FEATURE, "Extrusion end condition"),
        ("moFromEndSpec_c", ClassCategory.FEATURE, "From-end condition"),
        ("moWeldmentFeature_c", ClassCategory.FEATURE, "Weldment structural member"),
        ("moPerBodyChooserData_c", ClassCategory.FEATURE, "Per-body chooser data"),
        # ── Dimensions ──
        ("moLengthParameter_c", ClassCategory.DIMENSION, "Length dimension (float64, meters)"),
        ("moAngleParameter_c", ClassCategory.DIMENSION, "Angle dimension"),
        ("ParallelPlaneDistanceDim_c", ClassCategory.DIMENSION, "Plane-to-plane distance"),
        ("AngleDim_c", ClassCategory.DIMENSION, "Angle dimension"),
        ("moDisplayDistanceDim_c", ClassCategory.DIMENSION, "Distance dimension display"),
        ("moDisplayRadialDim_c", ClassCategory.DIMENSION, "Radial dimension display"),
        ("moFeatureDimHandle_c", ClassCategory.DIMENSION, "Feature dimension handle"),
        ("moSkDimHandleRadial_c", ClassCategory.DIMENSION, "Radial dim handle (<MOD-DIAM>)"),
        ("sgPntPntDist", ClassCategory.DIMENSION, "Point-to-point distance constraint"),
        ("sgPntLineDist", ClassCategory.DIMENSION, "Point-to-line distance constraint"),
        # ── Geometry refs ──
        ("moFaceRef_c", ClassCategory.GEOMETRY_REF, "Face reference"),
        ("moEdgeRef_c", ClassCategory.GEOMETRY_REF, "Edge reference"),
        ("moEndFaceSurfIdRep_c", ClassCategory.GEOMETRY_REF, "End face surface ID"),
        ("moBBoxCenterData_c", ClassCategory.GEOMETRY_REF, "Bounding box center data"),
        ("moCompFace_c", ClassCategory.GEOMETRY_REF, "Component face"),
        ("moCompEdge_c", ClassCategory.GEOMETRY_REF, "Component edge"),
        # ── Assembly ──
        ("moAssembly_c", ClassCategory.ASSEMBLY, "Root assembly object"),
        ("moMateGroup_c", ClassCategory.ASSEMBLY, "Mate group container"),
        ("moCompFeature_c", ClassCategory.ASSEMBLY, "Component feature reference"),
        ("moCompHolder_c", ClassCategory.ASSEMBLY, "Component holder"),
        ("moCompRefPlane_c", ClassCategory.ASSEMBLY, "Component reference plane"),
        # ── Infrastructure ──
        ("moPart_c", ClassCategory.INFRASTRUCTURE, "Root part object"),
        ("moHeader_c", ClassCategory.INFRASTRUCTURE, "Document header"),
        ("moLogs_c", ClassCategory.INFRASTRUCTURE, "Modification logs"),
        ("moStamp_c", ClassCategory.INFRASTRUCTURE, "Timestamp"),
        ("moNodeName_c", ClassCategory.INFRASTRUCTURE, "Feature/node name"),
        ("moView_c", ClassCategory.INFRASTRUCTURE, "Viewport orientation"),
        ("moUnitsTable_c", ClassCategory.INFRASTRUCTURE, "Units configuration"),
        ("moMaterial_c", ClassCategory.INFRASTRUCTURE, "Material definition"),
        ("moVisualProperties_c", ClassCategory.INFRASTRUCTURE, "Visual appearance"),
        ("gcXhatch_c", ClassCategory.INFRASTRUCTURE, "Cross-hatch pattern"),
        ("moRefPlane_c", ClassCategory.INFRASTRUCTURE, "Reference plane"),
        ("moDefaultRefPlnData_c", ClassCategory.INFRASTRUCTURE, "Default planes (Front/Top/Right)"),
    ]

    return {e[0]: ClassInfo(name=e[0], category=e[1], description=e[2]) for e in entries}


CLASS_REGISTRY: dict[str, ClassInfo] = _build_registry()


def lookup(class_name: str) -> ClassInfo | None:
    """Look up a class by name. Returns ``None`` if unknown."""
    return CLASS_REGISTRY.get(class_name)


def categorize(class_name: str) -> ClassCategory:
    """Return the category for a class, defaulting to OTHER."""
    info = CLASS_REGISTRY.get(class_name)
    return info.category if info else ClassCategory.OTHER
