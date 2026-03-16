import { useState } from "react";
import { useEditorStore } from "@/stores/editor-store";
import {
  usePreferencesStore,
  type UnitSystem,
} from "@/stores/preferences-store";

const UNIT_OPTIONS: { value: UnitSystem; label: string }[] = [
  { value: "mm", label: "mm" },
  { value: "cm", label: "cm" },
  { value: "m", label: "m" },
  { value: "in", label: "in" },
  { value: "ft", label: "ft" },
];

function getSketchStatusText(): string {
  const state = useEditorStore.getState();
  const session = state.sketchSession;
  if (!session) return "Editing Sketch";

  const tool = session.activeTool;
  const pending = session.pendingPoints.length;
  const dimPending = session.dimensionPending;

  let toolText = "";
  if (!tool) {
    toolText = "Select a tool or entity";
  } else if (tool === "line") {
    toolText = pending === 0
      ? "Line: Click to start"
      : "Line: Click to set endpoint (Esc to end chain)";
  } else if (tool === "rectangle") {
    toolText = pending === 0
      ? "Rectangle: Click first corner"
      : "Rectangle: Click opposite corner";
  } else if (tool === "circle") {
    toolText = pending === 0
      ? "Circle: Click center"
      : "Circle: Click to set radius";
  } else if (tool === "arc") {
    if (pending === 0) toolText = "Arc: Click start point";
    else if (pending === 1) toolText = "Arc: Click end point";
    else toolText = "Arc: Click to set curvature";
  } else if (tool === "dimension") {
    if (dimPending) {
      toolText = "Dimension: Click to place, or click another entity";
    } else {
      toolText = "Dimension: Click an entity";
    }
  } else if (tool === "measure") {
    toolText = pending === 0
      ? "Measure: Click first point"
      : "Measure: Click second point";
  }

  // Append DOF status
  const dof = state.sketchDofStatus;
  if (dof === "fully_constrained") {
    return `${toolText} — Fully Defined`;
  } else if (dof === "over_constrained") {
    return `${toolText} — Over Defined`;
  } else if (dof === "under_constrained") {
    return `${toolText} — Under Defined`;
  }

  return toolText;
}

function UnitSelector() {
  const [open, setOpen] = useState(false);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);
  const setUnitSystem = usePreferencesStore((s) => s.setUnitSystem);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="px-1.5 py-0.5 rounded hover:bg-white/10 text-[var(--cad-text-muted)] hover:text-[var(--cad-text-primary)] transition-colors cursor-pointer"
        title="Change units"
      >
        {unitSystem}
      </button>
      {open && (
        <div className="absolute bottom-full right-0 mb-1 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-lg py-0.5 z-50">
          {UNIT_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              onClick={() => {
                setUnitSystem(opt.value);
                setOpen(false);
              }}
              className={`block w-full px-3 py-1 text-left text-[10px] hover:bg-white/10 ${
                unitSystem === opt.value
                  ? "text-[var(--cad-accent)] font-medium"
                  : "text-[var(--cad-text-secondary)]"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

export function StatusBar() {
  const meshData = useEditorStore((s) => s.meshData);
  const mode = useEditorStore((s) => s.mode);
  const selectedFaceIndex = useEditorStore((s) => s.selectedFaceIndex);
  const activeOperation = useEditorStore((s) => s.activeOperation);
  // Subscribe to sketch state changes for reactive updates
  useEditorStore((s) => s.sketchSession?.activeTool);
  useEditorStore((s) => s.sketchSession?.pendingPoints.length ?? 0);
  useEditorStore((s) => s.sketchSession?.dimensionPending);
  useEditorStore((s) => s.sketchDofStatus);

  let statusText = "Ready";
  if (mode === "sketch") {
    statusText = getSketchStatusText();
  } else if (mode === "select-plane") {
    statusText = "Select a sketch plane";
  } else if (activeOperation) {
    statusText = `Editing ${activeOperation.type}`;
  } else if (mode === "select-face" && selectedFaceIndex !== null) {
    statusText = `Face ${selectedFaceIndex} selected`;
  } else if (mode === "select-face") {
    statusText = "Select a face";
  }

  const isSketchActive = mode === "sketch" || mode === "select-plane";

  return (
    <div className={`flex items-center justify-between border-t border-[var(--cad-border)] px-3 ${
      isSketchActive
        ? "bg-[#1a2a3a] text-[11px] text-[#88bbff] py-0.5"
        : "bg-[var(--cad-bg-panel)] text-[10px] text-[var(--cad-text-muted)]"
    }`}>
      <span data-testid="status-text">{statusText}</span>
      <div className="flex items-center gap-3">
        {meshData && meshData.vertexCount > 0 && (
          <>
            <span data-testid="vertex-count">Verts: {meshData.vertexCount}</span>
            <span>Tris: {meshData.triangleCount}</span>
          </>
        )}
        <UnitSelector />
      </div>
    </div>
  );
}
