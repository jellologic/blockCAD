import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

type HoleType = "simple" | "counterbore" | "countersink";

export function HoleWizardPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "hole_wizard") return null;

  const {
    hole_type = "simple" as HoleType,
    diameter = 5,
    depth = 10,
    through_all = false,
    position = [0, 0, 0],
    direction = [0, -1, 0],
    cbore_diameter = 8,
    cbore_depth = 3,
    csink_diameter = 10,
    csink_angle = 82,
  } = activeOperation.params;

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="hole-wizard-panel">
      {/* Hole Type */}
      <div>
        <h4 className={sectionHeaderClass}>Hole Type</h4>
        <select
          data-testid="hole-wizard-type"
          className={inputClass}
          value={hole_type}
          onChange={(e) => updateOperationParams({ hole_type: e.target.value as HoleType })}
        >
          <option value="simple">Simple</option>
          <option value="counterbore">Counterbore</option>
          <option value="countersink">Countersink</option>
        </select>
      </div>

      {/* Diameter */}
      <div>
        <label className={sectionHeaderClass}>Diameter</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={diameter}
            onChange={(e) => updateOperationParams({ diameter: Math.max(0.1, Number(e.target.value)) })}
            data-testid="hole-wizard-diameter"
            className={inputClass}
            min={0.1}
            step={0.5}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>

      {/* Through All */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="hole-wizard-through-all"
            checked={through_all}
            onChange={(e) => updateOperationParams({ through_all: e.target.checked })}
            data-testid="hole-wizard-through-all"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="hole-wizard-through-all" className="text-xs text-[var(--cad-text-secondary)]">
            Through All
          </label>
        </div>
      </div>

      {/* Depth (disabled when through_all) */}
      {!through_all && (
        <div>
          <label className={sectionHeaderClass}>Depth</label>
          <div className="flex items-center gap-1">
            <input
              type="number"
              value={depth}
              onChange={(e) => updateOperationParams({ depth: Math.max(0.1, Number(e.target.value)) })}
              data-testid="hole-wizard-depth"
              className={inputClass}
              min={0.1}
              step={0.5}
            />
            <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
          </div>
        </div>
      )}

      {/* Counterbore parameters */}
      {hole_type === "counterbore" && (
        <div className="space-y-2 border-l-2 border-[var(--cad-border)] pl-3">
          <h4 className={sectionHeaderClass}>Counterbore</h4>
          <div>
            <label className={sectionHeaderClass}>C&apos;bore Diameter</label>
            <div className="flex items-center gap-1">
              <input
                type="number"
                value={cbore_diameter}
                onChange={(e) => updateOperationParams({ cbore_diameter: Math.max(diameter + 0.1, Number(e.target.value)) })}
                data-testid="hole-wizard-cbore-diameter"
                className={inputClass}
                min={diameter + 0.1}
                step={0.5}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
            </div>
          </div>
          <div>
            <label className={sectionHeaderClass}>C&apos;bore Depth</label>
            <div className="flex items-center gap-1">
              <input
                type="number"
                value={cbore_depth}
                onChange={(e) => updateOperationParams({ cbore_depth: Math.max(0.1, Number(e.target.value)) })}
                data-testid="hole-wizard-cbore-depth"
                className={inputClass}
                min={0.1}
                step={0.5}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
            </div>
          </div>
        </div>
      )}

      {/* Countersink parameters */}
      {hole_type === "countersink" && (
        <div className="space-y-2 border-l-2 border-[var(--cad-border)] pl-3">
          <h4 className={sectionHeaderClass}>Countersink</h4>
          <div>
            <label className={sectionHeaderClass}>C&apos;sink Diameter</label>
            <div className="flex items-center gap-1">
              <input
                type="number"
                value={csink_diameter}
                onChange={(e) => updateOperationParams({ csink_diameter: Math.max(diameter + 0.1, Number(e.target.value)) })}
                data-testid="hole-wizard-csink-diameter"
                className={inputClass}
                min={diameter + 0.1}
                step={0.5}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
            </div>
          </div>
          <div>
            <label className={sectionHeaderClass}>C&apos;sink Angle</label>
            <div className="flex items-center gap-1">
              <input
                type="number"
                value={csink_angle}
                onChange={(e) => updateOperationParams({ csink_angle: Math.min(120, Math.max(1, Number(e.target.value))) })}
                data-testid="hole-wizard-csink-angle"
                className={inputClass}
                min={1}
                max={120}
                step={1}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">deg</span>
            </div>
          </div>
        </div>
      )}

      {/* Position */}
      <div>
        <h4 className={sectionHeaderClass}>Position</h4>
        <div className="space-y-1">
          {(["X", "Y", "Z"] as const).map((axis, i) => (
            <div key={axis} className="flex items-center gap-1">
              <span className="w-4 flex-shrink-0 text-[10px] font-medium text-[var(--cad-text-muted)]">{axis}</span>
              <input
                type="number"
                value={position[i]}
                onChange={(e) => {
                  const newPos = [...position];
                  newPos[i] = Number(e.target.value);
                  updateOperationParams({ position: newPos });
                }}
                data-testid={`hole-wizard-pos-${axis.toLowerCase()}`}
                className={inputClass}
                step={0.5}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
            </div>
          ))}
        </div>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            store.setMode(store.mode === "select-face" ? "view" : "select-face");
          }}
          data-testid="hole-wizard-select-face"
          className="mt-1.5 w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
        >
          Click face to set position...
        </button>
      </div>

      {/* Direction */}
      <div>
        <h4 className={sectionHeaderClass}>Direction</h4>
        <div className="space-y-1">
          {(["X", "Y", "Z"] as const).map((axis, i) => (
            <div key={axis} className="flex items-center gap-1">
              <span className="w-4 flex-shrink-0 text-[10px] font-medium text-[var(--cad-text-muted)]">{axis}</span>
              <input
                type="number"
                value={direction[i]}
                onChange={(e) => {
                  const newDir = [...direction];
                  newDir[i] = Number(e.target.value);
                  updateOperationParams({ direction: newDir });
                }}
                data-testid={`hole-wizard-dir-${axis.toLowerCase()}`}
                className={inputClass}
                step={0.1}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
