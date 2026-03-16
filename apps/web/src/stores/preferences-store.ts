import { create } from "zustand";
import { persist } from "zustand/middleware";

export type UnitSystem = "mm" | "cm" | "m" | "in" | "ft";
export type InteractionStyle = "solidworks" | "fusion360";

/** Conversion factors: how many mm per 1 unit */
const MM_PER_UNIT: Record<UnitSystem, number> = {
  mm: 1,
  cm: 10,
  m: 1000,
  in: 25.4,
  ft: 304.8,
};

const UNIT_LABELS: Record<UnitSystem, string> = {
  mm: "mm",
  cm: "cm",
  m: "m",
  in: "in",
  ft: "ft",
};

interface PreferencesState {
  unitSystem: UnitSystem;
  interactionStyle: InteractionStyle;
  dimensionDecimals: number;

  setUnitSystem: (unit: UnitSystem) => void;
  setInteractionStyle: (style: InteractionStyle) => void;
  setDimensionDecimals: (decimals: number) => void;
}

export const usePreferencesStore = create<PreferencesState>()(
  persist(
    (set) => ({
      unitSystem: "mm",
      interactionStyle: "fusion360",
      dimensionDecimals: 2,

      setUnitSystem: (unit) => set({ unitSystem: unit }),
      setInteractionStyle: (style) => set({ interactionStyle: style }),
      setDimensionDecimals: (decimals) => set({ dimensionDecimals: decimals }),
    }),
    { name: "blockcad-preferences" }
  )
);

/**
 * Convert a value from the display unit to mm (internal storage unit).
 */
export function toMm(value: number, unit: UnitSystem): number {
  return value * MM_PER_UNIT[unit];
}

/**
 * Convert a value from mm (internal) to the display unit.
 */
export function fromMm(valueMm: number, unit: UnitSystem): number {
  return valueMm / MM_PER_UNIT[unit];
}

/**
 * Format a dimension value (stored in mm) for display.
 */
export function formatDimension(valueMm: number): string {
  const { unitSystem, dimensionDecimals } = usePreferencesStore.getState();
  const displayValue = fromMm(valueMm, unitSystem);
  return `${displayValue.toFixed(dimensionDecimals)} ${UNIT_LABELS[unitSystem]}`;
}

/**
 * Format a dimension for a specific kind.
 */
export function formatDimensionWithPrefix(valueMm: number, kind: string): string {
  const formatted = formatDimension(valueMm);
  if (kind === "radius") return `R ${formatted}`;
  if (kind === "diameter") return `\u00D8 ${formatted}`; // Ø
  if (kind === "angle") {
    const { dimensionDecimals } = usePreferencesStore.getState();
    // Angles are stored in radians internally, display in degrees
    const degrees = valueMm * (180 / Math.PI);
    return `${degrees.toFixed(dimensionDecimals)}°`;
  }
  return formatted;
}

/**
 * Parse user input string into mm value.
 * Supports:
 *  - Plain numbers: "25" → 25 in current unit → converted to mm
 *  - Unit suffix: "25mm", "2in", "1ft", "10cm" → converted to mm
 *  - Simple expressions: "25/2", "10+5", "2*3" → evaluated then converted
 */
export function parseUserInput(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  // Check for explicit unit suffix
  const unitMatch = trimmed.match(/^(.+?)\s*(mm|cm|m|in|ft)$/i);
  if (unitMatch) {
    const expr = unitMatch[1]!.trim();
    const unit = unitMatch[2]!.toLowerCase() as UnitSystem;
    const value = safeEval(expr);
    if (value === null || isNaN(value)) return null;
    return toMm(value, unit);
  }

  // No unit suffix — interpret in current document unit
  const value = safeEval(trimmed);
  if (value === null || isNaN(value)) return null;
  const { unitSystem } = usePreferencesStore.getState();
  return toMm(value, unitSystem);
}

/**
 * Parse user input for angle dimensions (degrees → radians).
 */
export function parseAngleInput(input: string): number | null {
  const trimmed = input.trim().replace(/°$/, "");
  const value = safeEval(trimmed);
  if (value === null || isNaN(value)) return null;
  return value * (Math.PI / 180); // degrees to radians
}

/**
 * Safely evaluate a simple math expression.
 * Only allows numbers, +, -, *, /, (, ), and whitespace.
 */
function safeEval(expr: string): number | null {
  // Only allow safe characters
  if (!/^[\d\s+\-*/().]+$/.test(expr)) return null;
  try {
    const result = new Function(`"use strict"; return (${expr});`)() as number;
    if (typeof result !== "number" || !isFinite(result)) return null;
    return result;
  } catch {
    return null;
  }
}
