import { describe, it, expect, beforeEach } from "vitest";
import { toMm, fromMm, formatDimension, formatDimensionWithPrefix, parseUserInput, parseAngleInput, usePreferencesStore } from "@/stores/preferences-store";

describe("preferences store - unit conversion", () => {
  beforeEach(() => {
    usePreferencesStore.setState({ unitSystem: "mm", dimensionDecimals: 2 });
  });

  // toMm tests
  it("converts mm to mm (identity)", () => { expect(toMm(1, "mm")).toBe(1); });
  it("converts inches to mm", () => { expect(toMm(1, "in")).toBeCloseTo(25.4); });
  it("converts feet to mm", () => { expect(toMm(1, "ft")).toBeCloseTo(304.8); });
  it("converts cm to mm", () => { expect(toMm(1, "cm")).toBe(10); });
  it("converts m to mm", () => { expect(toMm(1, "m")).toBe(1000); });

  // fromMm tests
  it("round-trip conversion preserves value", () => {
    expect(fromMm(toMm(5, "in"), "in")).toBeCloseTo(5);
  });
});

describe("preferences store - parseUserInput", () => {
  beforeEach(() => {
    usePreferencesStore.setState({ unitSystem: "mm", dimensionDecimals: 2 });
  });

  it("parses plain number in current unit (mm)", () => {
    expect(parseUserInput("25")).toBeCloseTo(25);
  });

  it("parses number with mm suffix", () => {
    expect(parseUserInput("25mm")).toBeCloseTo(25);
  });

  it("parses number with in suffix", () => {
    expect(parseUserInput("2in")).toBeCloseTo(50.8);
  });

  it("parses number with ft suffix", () => {
    expect(parseUserInput("1ft")).toBeCloseTo(304.8);
  });

  it("parses number with cm suffix", () => {
    expect(parseUserInput("10cm")).toBeCloseTo(100);
  });

  it("parses math expression", () => {
    expect(parseUserInput("25/2")).toBeCloseTo(12.5);
  });

  it("parses addition expression", () => {
    expect(parseUserInput("10+5")).toBeCloseTo(15);
  });

  it("parses multiplication expression", () => {
    expect(parseUserInput("2*3")).toBeCloseTo(6);
  });

  it("parses expression with unit suffix", () => {
    expect(parseUserInput("25/2 mm")).toBeCloseTo(12.5);
  });

  it("returns null for empty string", () => {
    expect(parseUserInput("")).toBeNull();
  });

  it("returns null for non-numeric input", () => {
    expect(parseUserInput("abc")).toBeNull();
  });

  it("returns null for code injection attempt", () => {
    expect(parseUserInput("alert(1)")).toBeNull();
  });

  it("accepts negative numbers", () => {
    expect(parseUserInput("-5")).toBeCloseTo(-5);
  });

  it("respects current unit system for plain numbers", () => {
    usePreferencesStore.setState({ unitSystem: "in" });
    // "25" in inches = 25 * 25.4 mm
    expect(parseUserInput("25")).toBeCloseTo(25 * 25.4);
  });
});

describe("preferences store - parseAngleInput", () => {
  it("parses degrees to radians", () => {
    expect(parseAngleInput("90")).toBeCloseTo(Math.PI / 2);
  });

  it("strips degree symbol", () => {
    expect(parseAngleInput("90°")).toBeCloseTo(Math.PI / 2);
  });

  it("parses expression", () => {
    expect(parseAngleInput("45/2")).toBeCloseTo(Math.PI / 8);
  });

  it("returns null for empty string", () => {
    expect(parseAngleInput("")).toBeNull();
  });
});

describe("preferences store - formatDimension", () => {
  it("formats in mm with 2 decimals", () => {
    usePreferencesStore.setState({ unitSystem: "mm", dimensionDecimals: 2 });
    expect(formatDimension(25.4)).toBe("25.40 mm");
  });

  it("converts mm to inches for display", () => {
    usePreferencesStore.setState({ unitSystem: "in", dimensionDecimals: 2 });
    expect(formatDimension(25.4)).toBe("1.00 in");
  });

  it("respects dimensionDecimals setting", () => {
    usePreferencesStore.setState({ unitSystem: "mm", dimensionDecimals: 0 });
    expect(formatDimension(25.4)).toBe("25 mm");
  });
});

describe("preferences store - formatDimensionWithPrefix", () => {
  beforeEach(() => {
    usePreferencesStore.setState({ unitSystem: "mm", dimensionDecimals: 2 });
  });

  it("adds R prefix for radius", () => {
    expect(formatDimensionWithPrefix(5, "radius")).toBe("R 5.00 mm");
  });

  it("adds \u00D8 prefix for diameter", () => {
    expect(formatDimensionWithPrefix(10, "diameter")).toBe("\u00D8 10.00 mm");
  });

  it("formats angle in degrees", () => {
    const result = formatDimensionWithPrefix(Math.PI / 2, "angle");
    expect(result).toBe("90.00\u00B0");
  });
});
