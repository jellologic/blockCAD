import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function DomePanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const selectedFaceIndex = useEditorStore((s) => s.selectedFaceIndex);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "dome") return null;

  const {
    face_index = null,
    height = 5,
    elliptical = false,
    direction = null,
  } = activeOperation.params;

  const hasDirection = direction != null;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="dome-panel">
      {/* Face Selection */}
      <div>
        <h4 className={sectionHeaderClass}>Face</h4>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            if (store.mode === "select-face") {
              store.setMode("view");
            } else {
              store.setMode("select-face");
            }
          }}
          data-testid="dome-select-face"
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
        >
          {face_index != null
            ? `Face ${face_index} selected`
            : "Click a face to select..."}
        </button>
        {selectedFaceIndex != null && selectedFaceIndex !== face_index && (
          <button
            onClick={() => updateOperationParams({ face_index: selectedFaceIndex })}
            data-testid="dome-set-face"
            className="mt-1 w-full rounded border border-[var(--cad-accent)]/30 bg-[var(--cad-accent)]/10 px-2 py-1 text-xs text-[var(--cad-accent)] hover:bg-[var(--cad-accent)]/20 transition-colors"
          >
            Use face {selectedFaceIndex}
          </button>
        )}
        <p className="mt-0.5 text-[10px] text-[var(--cad-text-muted)]">
          Select the face to dome
        </p>
      </div>

      {/* Height */}
      <div>
        <label className={sectionHeaderClass}>Height</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={height}
            onChange={(e) => updateOperationParams({ height: Math.max(0.01, Number(e.target.value)) })}
            data-testid="dome-height"
            className={inputClass}
            min={0.01}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>

      {/* Elliptical toggle */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="dome-elliptical"
            checked={elliptical}
            onChange={(e) => updateOperationParams({ elliptical: e.target.checked })}
            data-testid="dome-elliptical"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="dome-elliptical" className="text-xs text-[var(--cad-text-secondary)]">
            Elliptical
          </label>
        </div>
      </div>

      {/* Direction Override */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="dome-direction-override"
            checked={hasDirection}
            onChange={(e) => updateOperationParams({
              direction: e.target.checked ? [0, 0, 1] : null,
            })}
            data-testid="dome-direction-override"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="dome-direction-override" className="text-xs text-[var(--cad-text-secondary)]">
            Direction Override
          </label>
        </div>
        {hasDirection && (
          <div className="mt-1.5 space-y-1 pl-5">
            {(["X", "Y", "Z"] as const).map((axis, i) => (
              <div key={axis} className="flex items-center gap-1">
                <span className="w-3 text-[10px] text-[var(--cad-text-muted)]">{axis}</span>
                <input
                  type="number"
                  value={direction[i]}
                  onChange={(e) => {
                    const d = [...direction];
                    d[i] = Number(e.target.value);
                    updateOperationParams({ direction: d });
                  }}
                  data-testid={`dome-direction-${axis.toLowerCase()}`}
                  className={inputClass}
                  step={0.1}
                />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
