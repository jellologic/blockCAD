import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/** Handle a click in dimension mode -- find the nearest line entity and prompt for dimension */
export function handleDimensionClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  // Find the closest line to the click position
  const lines = session.entities.filter((e) => e.type === "line");
  let bestLine: (typeof lines)[0] | null = null;
  let bestDist = Infinity;

  for (const line of lines) {
    if (line.type !== "line") continue;
    const startPt = session.entities.find((e) => e.id === line.startId);
    const endPt = session.entities.find((e) => e.id === line.endId);
    if (!startPt || startPt.type !== "point" || !endPt || endPt.type !== "point")
      continue;

    // Distance from click to line midpoint (simple heuristic)
    const mx = (startPt.position.x + endPt.position.x) / 2;
    const my = (startPt.position.y + endPt.position.y) / 2;
    const dist = Math.sqrt((clickPos.x - mx) ** 2 + (clickPos.y - my) ** 2);

    if (dist < bestDist) {
      bestDist = dist;
      bestLine = line;
    }
  }

  if (bestLine && bestLine.type === "line" && bestDist < 5) {
    // Found a nearby line -- show dimension input at click position
    store.showDimensionInput(clickPos, [bestLine.startId, bestLine.endId], "distance");
  }
}
