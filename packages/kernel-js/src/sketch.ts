/**
 * High-level TypeScript client for sketch editing operations.
 */
export class SketchClient {
  // private handle: SketchHandle; // from WASM

  constructor() {
    // TODO: Initialize from WASM SketchHandle
  }

  get entityCount(): number {
    return 0;
  }

  get constraintCount(): number {
    return 0;
  }

  solve(): void {
    throw new Error("Not yet implemented");
  }
}
