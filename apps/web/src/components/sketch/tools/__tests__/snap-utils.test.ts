import { describe, it, expect } from "vitest";
import type { SketchEntity2D } from "@blockCAD/kernel";
import { findNearestPoint, findSnapTarget } from "../snap-utils";

describe("snap-utils", () => {
  it("findNearestPoint returns closest point within threshold", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
      { type: "point", id: "p2", position: { x: 10, y: 0 } },
    ];
    const result = findNearestPoint({ x: 0.1, y: 0.1 }, entities);
    expect(result).not.toBeNull();
    expect(result!.id).toBe("p1");
  });

  it("findNearestPoint returns null when no point in range", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
    ];
    const result = findNearestPoint({ x: 100, y: 100 }, entities);
    expect(result).toBeNull();
  });

  it("findSnapTarget finds midpoint snap", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
      { type: "point", id: "p2", position: { x: 10, y: 0 } },
      { type: "line", id: "l1", startId: "p1", endId: "p2" },
    ];
    // Cursor near midpoint (5, 0) but not near any existing point
    const result = findSnapTarget({ x: 5.1, y: 0.1 }, entities);
    expect(result).not.toBeNull();
    expect(result!.type).toBe("midpoint");
  });

  it("findSnapTarget finds center snap", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "c1", position: { x: 5, y: 5 } },
      { type: "circle", id: "circle1", centerId: "c1", radius: 3 },
    ];
    // Cursor near center but not exactly on the point
    const result = findSnapTarget({ x: 5.1, y: 5.1 }, entities);
    expect(result).not.toBeNull();
    // Could be coincident (point entity) or center
    expect(["coincident", "center"]).toContain(result!.type);
  });

  it("findSnapTarget prefers coincident over midpoint", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
      { type: "point", id: "p2", position: { x: 10, y: 0 } },
      { type: "line", id: "l1", startId: "p1", endId: "p2" },
      // Place a point right at the midpoint
      { type: "point", id: "p3", position: { x: 5, y: 0 } },
    ];
    // Cursor near both point p3 and the line midpoint
    const result = findSnapTarget({ x: 5.1, y: 0.1 }, entities);
    expect(result).not.toBeNull();
    expect(result!.type).toBe("coincident");
  });
});
