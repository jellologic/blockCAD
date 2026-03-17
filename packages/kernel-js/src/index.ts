export { initKernel, isKernelInitialized } from "./init";
export { KernelClient, transformFeatureParams } from "./kernel";
export { SketchClient } from "./sketch";
export { AssemblyClient } from "./assembly";
export type { SolveResult, SolvedEntity } from "./sketch";
export { parseMeshBytes } from "./mesh";
export type { MeshData } from "./mesh";
export type {
  FeatureEntry,
  FeatureKind,
  ClientFeatureKind,
  ServerFeatureKind,
  FeatureParams,
  KernelDocument,
  Vec3,
  Point3,
  SketchPoint2D,
  SketchPlaneId,
  SketchPlane,
  SketchEntity2D,
  SketchConstraint2D,
  SketchFeatureData,
  RustPlane,
  RustEntityStore,
  RustSketchFeatureData,
  // Feature param types
  ExtrudeParams,
  RevolveParams,
  FilletParams,
  ChamferParams,
  ChamferMode,
  LinearPatternParams,
  CircularPatternParams,
  MirrorParams,
  ShellParams,
  // Batch 2
  RadiusPoint,
  VariableFilletParams,
  FaceFilletParams,
  TransformKind,
  MoveBodyParams,
  ScaleBodyParams,
  // Batch 3
  HoleType,
  HoleParams,
  DomeParams,
  RibParams,
  SplitKeep,
  SplitParams,
  CombineOperation,
  CombineParams,
  CurvePatternParams,
  // Reference geometry
  DatumPlaneKind,
  DatumPlaneParams,
  ReferenceAxisParams,
  ReferencePointParams,
  CoordinateSystemParams,
  // Server-only operations
  DraftParams,
  SweepParams,
  SweepOrientation,
  LoftParams,
  TangencyCondition,
  // Extended result types
  StepExportOptions,
  MassProperties,
} from "./types";
export { FRONT_PLANE, TOP_PLANE, RIGHT_PLANE } from "./types";
export { KernelError } from "./errors";
export type { KernelErrorKind } from "./errors";
