import { Check, X } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";
import { ExtrudePanel } from "./extrude-panel";

export function PropertyManager() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const confirmOperation = useEditorStore((s) => s.confirmOperation);
  const cancelOperation = useEditorStore((s) => s.cancelOperation);

  if (!activeOperation) return null;

  const title = activeOperation.type.charAt(0).toUpperCase() + activeOperation.type.slice(1);

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
        {activeOperation.type === "extrude" && <ExtrudePanel />}
        {activeOperation.type === "revolve" && (
          <p className="text-xs text-[var(--cad-text-muted)]">Revolve parameters coming soon</p>
        )}
      </div>
    </div>
  );
}
