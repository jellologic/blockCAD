import { SketchHandle } from "@blockCAD/kernel-wasm";
import type { SketchPlane } from "./types";
import { KernelError } from "./errors";

export interface SolveResult {
  converged: boolean;
  iterations: number;
  entities: SolvedEntity[];
}

export type SolvedEntity =
  | { type: "point"; x: number; y: number }
  | { type: "line" }
  | { type: "circle"; radius: number }
  | { type: "arc" };

/**
 * High-level TypeScript client for sketch editing operations.
 * Wraps the WASM SketchHandle for real-time constraint solving.
 */
export class SketchClient {
  private handle: SketchHandle;

  constructor(plane?: SketchPlane) {
    if (plane) {
      this.handle = SketchHandle.new_on_plane(
        JSON.stringify({
          origin: plane.origin,
          normal: plane.normal,
          uAxis: plane.uAxis,
          vAxis: plane.vAxis,
        })
      );
    } else {
      this.handle = new SketchHandle();
    }
  }

  get entityCount(): number {
    return this.handle.entity_count();
  }

  get constraintCount(): number {
    return this.handle.constraint_count();
  }

  /** Add a point entity. Returns entity index. */
  addPoint(x: number, y: number): number {
    return this.handle.add_entity(JSON.stringify({ type: "point", x, y }));
  }

  /** Add a line entity between two point indices. Returns entity index. */
  addLine(startIndex: number, endIndex: number): number {
    return this.handle.add_entity(
      JSON.stringify({ type: "line", startIndex, endIndex })
    );
  }

  /** Add a circle entity. Returns entity index. */
  addCircle(centerIndex: number, radius: number): number {
    return this.handle.add_entity(
      JSON.stringify({ type: "circle", centerIndex, radius })
    );
  }

  /** Add an arc entity. Returns entity index. */
  addArc(centerIndex: number, startIndex: number, endIndex: number): number {
    return this.handle.add_entity(
      JSON.stringify({ type: "arc", centerIndex, startIndex, endIndex })
    );
  }

  /** Add a constraint. Returns constraint index. */
  addConstraint(
    kind: string,
    entityIndices: number[],
    value?: number
  ): number {
    return this.handle.add_constraint(
      JSON.stringify({ kind, entityIndices, value: value ?? null })
    );
  }

  /** Solve constraints. Returns solved positions. */
  solve(): SolveResult {
    try {
      const json = this.handle.solve();
      return JSON.parse(json) as SolveResult;
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Get all entities as JSON. */
  getEntities(): SolvedEntity[] {
    try {
      return JSON.parse(this.handle.get_entities_json());
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Update a point position (for dragging). */
  updatePoint(entityIndex: number, x: number, y: number): void {
    this.handle.update_point(entityIndex, x, y);
  }

  /** Free WASM memory. */
  dispose(): void {
    this.handle.free();
  }
}
