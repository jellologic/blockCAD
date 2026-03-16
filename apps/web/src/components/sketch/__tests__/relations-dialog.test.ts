import { describe, it, expect } from "vitest";
import type { SketchEntity2D } from "@blockCAD/kernel";
import { RELATIONS, countSelectedTypes, isApplicable } from "../relations-dialog";

describe("relations dialog - countSelectedTypes", () => {
  it("counts points correctly", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
      { type: "point", id: "p2", position: { x: 1, y: 1 } },
    ];
    const counts = countSelectedTypes(["p1", "p2"], entities);
    expect(counts.points).toBe(2);
    expect(counts.lines).toBe(0);
  });

  it("counts lines correctly", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
      { type: "point", id: "p2", position: { x: 1, y: 0 } },
      { type: "line", id: "l1", startId: "p1", endId: "p2" },
    ];
    const counts = countSelectedTypes(["l1"], entities);
    expect(counts.lines).toBe(1);
    expect(counts.points).toBe(0);
  });

  it("counts circles correctly", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "c1", position: { x: 0, y: 0 } },
      { type: "circle", id: "ci1", centerId: "c1", radius: 5 },
      { type: "circle", id: "ci2", centerId: "c1", radius: 3 },
    ];
    const counts = countSelectedTypes(["ci1", "ci2"], entities);
    expect(counts.circles).toBe(2);
  });

  it("ignores non-existent IDs", () => {
    const entities: SketchEntity2D[] = [
      { type: "point", id: "p1", position: { x: 0, y: 0 } },
    ];
    const counts = countSelectedTypes(["nonexistent"], entities);
    expect(counts.points).toBe(0);
    expect(counts.lines).toBe(0);
  });
});

describe("relations dialog - isApplicable", () => {
  it("coincident requires 2 points", () => {
    expect(isApplicable({ points: 2 }, { points: 2, lines: 0, circles: 0, arcs: 0 })).toBe(true);
    expect(isApplicable({ points: 2 }, { points: 1, lines: 0, circles: 0, arcs: 0 })).toBe(false);
  });

  it("horizontal requires 1 line", () => {
    expect(isApplicable({ lines: 1 }, { points: 0, lines: 1, circles: 0, arcs: 0 })).toBe(true);
    expect(isApplicable({ lines: 1 }, { points: 2, lines: 0, circles: 0, arcs: 0 })).toBe(false);
  });

  it("parallel requires 2 lines", () => {
    expect(isApplicable({ lines: 2 }, { points: 0, lines: 2, circles: 0, arcs: 0 })).toBe(true);
    expect(isApplicable({ lines: 2 }, { points: 0, lines: 1, circles: 0, arcs: 0 })).toBe(false);
  });

  it("coradial requires 2 circles", () => {
    expect(isApplicable({ circles: 2 }, { points: 0, lines: 0, circles: 2, arcs: 0 })).toBe(true);
    expect(isApplicable({ circles: 2 }, { points: 0, lines: 0, circles: 1, arcs: 0 })).toBe(false);
  });

  it("tangent requires any 2 entities", () => {
    expect(isApplicable({ any: 2 }, { points: 0, lines: 1, circles: 1, arcs: 0 })).toBe(true);
    expect(isApplicable({ any: 2 }, { points: 1, lines: 0, circles: 0, arcs: 0 })).toBe(false);
  });

  it("fixed requires 1 point", () => {
    expect(isApplicable({ points: 1 }, { points: 1, lines: 0, circles: 0, arcs: 0 })).toBe(true);
    expect(isApplicable({ points: 1 }, { points: 0, lines: 1, circles: 0, arcs: 0 })).toBe(false);
  });
});

describe("relations dialog - constraint filtering integration", () => {
  it("2 points shows coincident and symmetric", () => {
    const counts = { points: 2, lines: 0, circles: 0, arcs: 0 };
    const applicable = RELATIONS.filter(r => isApplicable(r.requires, counts));
    const kinds = applicable.map(r => r.kind);
    expect(kinds).toContain("coincident");
    expect(kinds).toContain("symmetric");
    expect(kinds).not.toContain("parallel");
  });

  it("1 line shows horizontal and vertical", () => {
    const counts = { points: 0, lines: 1, circles: 0, arcs: 0 };
    const applicable = RELATIONS.filter(r => isApplicable(r.requires, counts));
    const kinds = applicable.map(r => r.kind);
    expect(kinds).toContain("horizontal");
    expect(kinds).toContain("vertical");
    expect(kinds).not.toContain("coincident");
  });

  it("2 lines shows parallel, perpendicular, equal, collinear", () => {
    const counts = { points: 0, lines: 2, circles: 0, arcs: 0 };
    const applicable = RELATIONS.filter(r => isApplicable(r.requires, counts));
    const kinds = applicable.map(r => r.kind);
    expect(kinds).toContain("parallel");
    expect(kinds).toContain("perpendicular");
    expect(kinds).toContain("equal");
    expect(kinds).toContain("collinear");
    expect(kinds).toContain("tangent"); // any: 2
  });

  it("2 circles shows coradial", () => {
    const counts = { points: 0, lines: 0, circles: 2, arcs: 0 };
    const applicable = RELATIONS.filter(r => isApplicable(r.requires, counts));
    const kinds = applicable.map(r => r.kind);
    expect(kinds).toContain("coradial");
    expect(kinds).toContain("tangent"); // any: 2
  });

  it("0 entities shows nothing", () => {
    const counts = { points: 0, lines: 0, circles: 0, arcs: 0 };
    const applicable = RELATIONS.filter(r => isApplicable(r.requires, counts));
    expect(applicable).toHaveLength(0);
  });
});
