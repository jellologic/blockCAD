// Barrel export for sketch tools

// Drawing tools
export { handleLineClick, applySnap, getSnapPreview } from "./line-tool";
export { handleRectangleClick } from "./rectangle-tool";
export { handleCircleClick } from "./circle-tool";
export { handleArcClick, circumcenter } from "./arc-tool";
export { handleEllipseClick } from "./ellipse-tool";
export { handlePolygonClick } from "./polygon-tool";
export { handleSlotClick } from "./slot-tool";

// Constraint / measure tools
export { handleDimensionClick } from "./dimension-tool";
export { handleMeasureClick } from "./measure-tool";

// Modify tools
export { handleTrimClick } from "./trim-tool";
export { handleExtendClick } from "./extend-tool";
export { handleOffsetClick } from "./offset-tool";
export { handleMirrorClick } from "./mirror-tool";

// Sketch fillet/chamfer
export {
  handleSketchFilletClick,
  getFilletRadius,
  setFilletRadius,
} from "./sketch-fillet-tool";
export {
  handleSketchChamferClick,
  getChamferDistance,
  setChamferDistance,
} from "./sketch-chamfer-tool";

// Pattern tools
export {
  handleSketchLinearPatternClick,
  getLinearPatternCount,
  setLinearPatternCount,
} from "./sketch-linear-pattern-tool";
export {
  handleSketchCircularPatternClick,
  getCircularPatternCount,
  setCircularPatternCount,
} from "./sketch-circular-pattern-tool";

// Other
export { handleConvertEntitiesClick } from "./convert-entities-tool";
export { handleBlockClick } from "./block-tool";

// Geometry utilities
export {
  lineLineIntersection,
  getPointPosition,
  getLineEndpoints,
  findIntersectionsWithLine,
  reflectPointAcrossLine,
  offsetLine,
} from "./geometry-utils";

// Snap utilities
export { findNearestPoint, findSnapTarget } from "./snap-utils";
