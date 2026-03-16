import { Check, X } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";
import { ExtrudePanel } from "./extrude-panel";
import { RevolvePanel } from "./revolve-panel";
import { FilletPanel } from "./fillet-panel";
import { ChamferPanel } from "./chamfer-panel";
import { LinearPatternPanel } from "./linear-pattern-panel";
import { CircularPatternPanel } from "./circular-pattern-panel";
import { MirrorPanel } from "./mirror-panel";

export function PropertyManager() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const confirmOperation = useEditorStore((s) => s.confirmOperation);
  const cancelOperation = useEditorStore((s) => s.cancelOperation);

  if (!activeOperation) return null;

  const displayNames: Record<string, string> = {
    extrude: "Extrude",
    cut_extrude: "Cut Extrude",
    revolve: "Revolve",
    cut_revolve: "Cut Revolve",
    fillet: "Fillet",
    chamfer: "Chamfer",
    linear_pattern: "Linear Pattern",
    circular_pattern: "Circular Pattern",
    mirror: "Mirror",
  };
  const title = displayNames[activeOperation.type] ??
    activeOperation.type.charAt(0).toUpperCase() + activeOperation.type.slice(1);

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      {/* Title bar with confirm/cancel */}
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <span className="text-sm font-medium text-[var(--cad-text-primary)]">{title}</span>
        <div className="flex items-center gap-1">
          <button
            onClick={confirmOperation}
            data-testid="operation-confirm"
            className="rounded p-1 transition-colors hover:bg-[var(--cad-confirm)]/20"
            title="Confirm (Enter)"
          >
            <Check size={18} style={{ color: "var(--cad-confirm)" }} />
          </button>
          <button
            onClick={cancelOperation}
            data-testid="operation-cancel"
            className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20"
            title="Cancel (Escape)"
          >
            <X size={18} style={{ color: "var(--cad-cancel)" }} />
          </button>
        </div>
      </div>

      {/* Operation-specific panel */}
      <div className="flex-1 overflow-y-auto p-3">
        {(activeOperation.type === "extrude" || activeOperation.type === "cut_extrude") && <ExtrudePanel />}
        {(activeOperation.type === "revolve" || activeOperation.type === "cut_revolve") && <RevolvePanel />}
        {activeOperation.type === "fillet" && <FilletPanel />}
        {activeOperation.type === "chamfer" && <ChamferPanel />}
        {activeOperation.type === "linear_pattern" && <LinearPatternPanel />}
        {activeOperation.type === "circular_pattern" && <CircularPatternPanel />}
        {activeOperation.type === "mirror" && <MirrorPanel />}
      </div>
    </div>
  );
}
