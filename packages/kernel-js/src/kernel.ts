import type { FeatureKind, FeatureParams } from "./types";
import type { MeshData } from "./mesh";

/**
 * High-level TypeScript client wrapping the WASM KernelHandle.
 * Provides ergonomic methods for feature tree manipulation.
 */
export class KernelClient {
  // private handle: KernelHandle; // from WASM

  constructor() {
    // TODO: Initialize from WASM KernelHandle
  }

  get featureCount(): number {
    // return this.handle.feature_count();
    return 0;
  }

  get cursor(): number {
    // return this.handle.cursor();
    return -1;
  }

  addFeature(_kind: FeatureKind, _params: FeatureParams): string {
    throw new Error("Not yet implemented");
  }

  tessellate(
    _chordTolerance: number = 0.01,
    _angleTolerance: number = 0.5
  ): MeshData {
    throw new Error("Not yet implemented");
  }

  serialize(): string {
    throw new Error("Not yet implemented");
  }

  static deserialize(_json: string): KernelClient {
    throw new Error("Not yet implemented");
  }
}
