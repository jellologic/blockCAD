import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function FilletPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "fillet") return null;

  const {
    radius = 1,
    edge_indices = [],
  } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="fillet-panel">
      {/* Edge Selection */}
      <div>
        <h4 className={sectionHeaderClass}>Edges</h4>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            store.setMode(store.mode === "select-face" ? "view" : "select-face");
          }}
          data-testid="fillet-select-edges"
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
        >
          {edge_indices.length > 0
            ? `${edge_indices.length} edge(s) selected — [${edge_indices.join(", ")}]`
            : "Click faces to select edges..."}
        </button>
        <p className="mt-0.5 text-[10px] text-[var(--cad-text-muted)]">
          Click faces to toggle their edges. Click again to deselect.
        </p>
      </div>

      {/* Radius */}
      <div>
        <label className={sectionHeaderClass}>Radius</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={radius}
            onChange={(e) => updateOperationParams({ radius: Math.max(0.1, Number(e.target.value)) })}
            data-testid="fillet-radius"
            className={inputClass}
            min={0.1}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>
    </div>
  );
}
