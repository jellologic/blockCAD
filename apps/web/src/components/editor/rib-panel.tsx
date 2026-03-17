import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function RibPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "rib") return null;

  const {
    thickness = 1,
    direction = [0, 0, 1],
    flip = false,
    both_sides = false,
  } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="rib-panel">
      {/* Thickness */}
      <div>
        <label className={sectionHeaderClass}>Thickness</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={thickness}
            onChange={(e) => updateOperationParams({ thickness: Math.max(0.01, Number(e.target.value)) })}
            data-testid="rib-thickness"
            className={inputClass}
            min={0.01}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>

      {/* Direction */}
      <div>
        <h4 className={sectionHeaderClass}>Direction</h4>
        <div className="space-y-1">
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
                data-testid={`rib-direction-${axis.toLowerCase()}`}
                className={inputClass}
                step={0.1}
              />
            </div>
          ))}
        </div>
      </div>

      {/* Flip */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="rib-flip"
            checked={flip}
            onChange={(e) => updateOperationParams({ flip: e.target.checked })}
            data-testid="rib-flip"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="rib-flip" className="text-xs text-[var(--cad-text-secondary)]">
            Flip Direction
          </label>
        </div>
      </div>

      {/* Both Sides */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="rib-both-sides"
            checked={both_sides}
            onChange={(e) => updateOperationParams({ both_sides: e.target.checked })}
            data-testid="rib-both-sides"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="rib-both-sides" className="text-xs text-[var(--cad-text-secondary)]">
            Both Sides
          </label>
        </div>
      </div>
    </div>
  );
}
