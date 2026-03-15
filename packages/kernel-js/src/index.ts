export { initKernel, isKernelInitialized } from "./init";
export { KernelClient } from "./kernel";
export { SketchClient } from "./sketch";
export type { SolveResult, SolvedEntity } from "./sketch";
export type { MeshData } from "./mesh";
export type {
  FeatureEntry,
  FeatureKind,
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
} from "./types";
export { FRONT_PLANE, TOP_PLANE, RIGHT_PLANE } from "./types";
export { KernelError } from "./errors";
export type { KernelErrorKind } from "./errors";
