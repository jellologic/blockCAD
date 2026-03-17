import { Check, X, Pencil } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";
import { SketchToolPanel } from "./sketch-tool-panel";

const PLANE_NAMES: Record<string, string> = {
  front: "Front Plane",
  top: "Top Plane",
  right: "Right Plane",
};

export function SketchPropertyPanel() {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const exitSketchMode = useEditorStore((s) => s.exitSketchMode);
  const dofStatus = useEditorStore((s) => s.sketchDofStatus);

  if (!sketchSession) return null;

  const pointCount = sketchSession.entities.filter(
    (e) => e.type === "point"
  ).length;
  const lineCount = sketchSession.entities.filter(
    (e) => e.type === "line"
  ).length;
  const circleCount = sketchSession.entities.filter(
    (e) => e.type === "circle"
  ).length;
  const arcCount = sketchSession.entities.filter(
    (e) => e.type === "arc"
  ).length;
  const constraintCount = sketchSession.constraints.length;

  // Use real DOF from solver, fall back to heuristic
  let status: string;
  let statusColor: string;
  if (dofStatus === "fully_constrained") {
    status = "Fully Defined";
    statusColor = "var(--cad-icon-sketch)";
  } else if (dofStatus === "over_constrained") {
    status = "Over Constrained";
    statusColor = "#cc2222";
  } else if (dofStatus === "under_constrained") {
    status = "Under Defined";
    statusColor = "#4488ff";
  } else if (constraintCount === 0 && pointCount === 0) {
    status = "Empty";
    statusColor = "var(--cad-text-muted)";
  } else {
    status = "Not Constrained";
    statusColor = "#4488ff";
  }

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <div className="flex items-center gap-2">
          <Pencil size={16} style={{ color: "var(--cad-icon-sketch)" }} />
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">
            Sketch
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => exitSketchMode(true)}
            data-testid="sketch-confirm"
            className="rounded p-1 transition-colors hover:bg-[var(--cad-confirm)]/20"
            title="Confirm Sketch (Enter)"
          >
            <Check size={18} style={{ color: "var(--cad-confirm)" }} />
          </button>
          <button
            onClick={() => exitSketchMode(false)}
            data-testid="sketch-cancel"
            className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20"
            title="Cancel Sketch (Escape)"
          >
            <X size={18} style={{ color: "var(--cad-cancel)" }} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        <div>
          <h4 className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
            Plane
          </h4>
          <p className="text-xs text-[var(--cad-text-secondary)]">
            {PLANE_NAMES[sketchSession.planeId] || sketchSession.planeId}
          </p>
        </div>

        <div>
          <h4 className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
            Entities
          </h4>
          <div className="text-xs text-[var(--cad-text-secondary)] space-y-0.5">
            <p>{pointCount} points</p>
            <p>{lineCount} lines</p>
            <p>{circleCount} circles</p>
            <p>{arcCount} arcs</p>
          </div>
        </div>

        <div>
          <h4 className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
            Constraints
          </h4>
          <p className="text-xs text-[var(--cad-text-secondary)]">
            {constraintCount} constraints
          </p>
        </div>

        <div>
          <h4 className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
            Status
          </h4>
          <p className="text-xs" style={{ color: statusColor }}>{status}</p>
        </div>
      </div>

      {/* Tool-specific parameter panel */}
      <SketchToolPanel />
    </div>
  );
}
