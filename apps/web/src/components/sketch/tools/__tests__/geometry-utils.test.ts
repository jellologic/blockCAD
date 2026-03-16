import { describe, it, expect } from "vitest";
import type { SketchEntity2D } from "@blockCAD/kernel";
import { lineLineIntersection, reflectPointAcrossLine, offsetLine, getPointPosition } from "../geometry-utils";

describe("geometry-utils", () => {
  it("lineLineIntersection finds crossing point", () => {
    const result = lineLineIntersection(
      { x: 0, y: 0 }, { x: 10, y: 10 },
      { x: 10, y: 0 }, { x: 0, y: 10 },
    );
    expect(result).not.toBeNull();
    expect(result!.point.x).toBeCloseTo(5);
    expect(result!.point.y).toBeCloseTo(5);
    expect(result!.t).toBeCloseTo(0.5);
    expect(result!.u).toBeCloseTo(0.5);
  });

  it("lineLineIntersection returns null for parallel lines", () => {
    const result = lineLineIntersection(
      { x: 0, y: 0 }, { x: 10, y: 0 },
      { x: 0, y: 5 }, { x: 10, y: 5 },
    );
    expect(result).toBeNull();
  });

  it("reflectPointAcrossLine reflects correctly", () => {
    // Reflect (1, 0) across the y-axis (line from (0,0) to (0,10))
    const reflected = reflectPointAcrossLine(
      { x: 1, y: 0 },
      { x: 0, y: 0 },
      { x: 0, y: 10 },
    );
    expect(reflected.x).toBeCloseTo(-1);
    expect(reflected.y).toBeCloseTo(0);
  });

  it("offsetLine offsets perpendicular", () => {
    // Offset a horizontal line (y=0) upward by distance 2
    const result = offsetLine(
      { x: 0, y: 0 },
      { x: 10, y: 0 },
      2,
    );
    expect(result.start.x).toBeCloseTo(0);
    expect(result.start.y).toBeCloseTo(2);
    expect(result.end.x).toBeCloseTo(10);
    expect(result.end.y).toBeCloseTo(2);
  });

  it("getPointPosition finds point by id", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 3, y: 7 } },
      { type: "point", id: "p2", position: { x: 8, y: 2 } },
    ];
    const pos = getPointPosition(entities, "p2");
    expect(pos).not.toBeNull();
    expect(pos!.x).toBe(8);
    expect(pos!.y).toBe(2);
  });
});
