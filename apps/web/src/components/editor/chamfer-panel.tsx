import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function ChamferPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "chamfer") return null;

  const {
    distance = 1,
    distance2 = null,
    edge_indices = [],
  } = activeOperation.params;

  const asymmetric = distance2 != null;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="chamfer-panel">
      {/* Edge Selection */}
      <div>
        <h4 className={sectionHeaderClass}>Edges</h4>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            store.setMode(store.mode === "select-face" ? "view" : "select-face");
          }}
          data-testid="chamfer-select-edges"
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
        >
          {edge_indices.length > 0
            ? `${edge_indices.length} edge(s) selected`
            : "Click faces to select edges..."}
        </button>
        <p className="mt-0.5 text-[10px] text-[var(--cad-text-muted)]">
          Click on faces to select their edges
        </p>
      </div>

      {/* Distance */}
      <div>
        <label className={sectionHeaderClass}>Distance</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={distance}
            onChange={(e) => updateOperationParams({ distance: Math.max(0.1, Number(e.target.value)) })}
            data-testid="chamfer-distance"
            className={inputClass}
            min={0.1}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>

      {/* Asymmetric toggle */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="chamfer-asymmetric"
            checked={asymmetric}
            onChange={(e) => updateOperationParams({
              distance2: e.target.checked ? distance : null,
            })}
            data-testid="chamfer-asymmetric"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="chamfer-asymmetric" className="text-xs text-[var(--cad-text-secondary)]">
            Asymmetric
          </label>
        </div>
        {asymmetric && (
          <div className="mt-1.5 flex items-center gap-1 pl-5">
            <input
              type="number"
              value={distance2 ?? distance}
              onChange={(e) => updateOperationParams({ distance2: Math.max(0.1, Number(e.target.value)) })}
              data-testid="chamfer-distance2"
              className={inputClass}
              min={0.1}
              step={0.5}
            />
            <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
          </div>
        )}
      </div>
    </div>
  );
}
