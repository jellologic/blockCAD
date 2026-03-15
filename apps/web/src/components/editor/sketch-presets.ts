/**
 * Generate sketch params for a rectangle centered at origin.
 * Returns a FeatureParams-compatible object for the mock kernel.
 */
export function rectangleSketchParams(width: number, height: number) {
  return {
    width,
    height,
    type: "rectangle" as const,
  };
}
