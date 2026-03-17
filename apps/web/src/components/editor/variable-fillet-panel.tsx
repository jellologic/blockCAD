import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

interface ControlPoint {
  parameter: number;
  radius: number;
}

export function VariableFilletPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "variable_fillet") return null;

  const {
    edge_indices = [],
    control_points = [
      { parameter: 0, radius: 1 },
      { parameter: 1, radius: 1 },
    ],
    smooth_transition = true,
  } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const updateControlPoint = (index: number, field: keyof ControlPoint, value: number) => {
    const updated = control_points.map((cp: ControlPoint, i: number) =>
      i === index ? { ...cp, [field]: value } : cp
    );
    updateOperationParams({ control_points: updated });
  };

  const addControlPoint = () => {
    // Insert a new point at parameter 0.5 with radius 1
    const newPoint: ControlPoint = { parameter: 0.5, radius: 1 };
    const updated = [...control_points, newPoint].sort(
      (a: ControlPoint, b: ControlPoint) => a.parameter - b.parameter
    );
    updateOperationParams({ control_points: updated });
  };

  const removeControlPoint = (index: number) => {
    if (control_points.length <= 2) return; // Must keep at least start and end
    const updated = control_points.filter((_: ControlPoint, i: number) => i !== index);
    updateOperationParams({ control_points: updated });
  };

  return (
    <div className="space-y-3" data-testid="variable-fillet-panel">
      {/* Edge Selection */}
      <div>
        <h4 className={sectionHeaderClass}>Edges</h4>
        <button
          onClick={() => {
            const store = useEditorStore.getState();
            store.setMode(store.mode === "select-face" ? "view" : "select-face");
          }}
          data-testid="variable-fillet-select-edges"
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

      {/* Control Points */}
      <div>
        <div className="flex items-center justify-between mb-1.5">
          <h4 className={sectionHeaderClass}>Control Points</h4>
          <button
            onClick={addControlPoint}
            data-testid="variable-fillet-add-point"
            className="rounded border border-[var(--cad-border)] px-1.5 py-0.5 text-[10px] text-[var(--cad-text-secondary)] hover:bg-[var(--cad-bg-hover)] transition-colors"
          >
            + Add
          </button>
        </div>
        <div className="space-y-2">
          {control_points.map((cp: ControlPoint, index: number) => (
            <div key={index} className="flex items-center gap-1" data-testid={`control-point-${index}`}>
              <div className="flex-1 space-y-1">
                <div className="flex items-center gap-1">
                  <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)] w-6">t</span>
                  <input
                    type="number"
                    value={cp.parameter}
                    onChange={(e) =>
                      updateControlPoint(index, "parameter", Math.max(0, Math.min(1, Number(e.target.value))))
                    }
                    data-testid={`control-point-${index}-parameter`}
                    className={inputClass}
                    min={0}
                    max={1}
                    step={0.1}
                  />
                </div>
                <div className="flex items-center gap-1">
                  <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)] w-6">R</span>
                  <input
                    type="number"
                    value={cp.radius}
                    onChange={(e) =>
                      updateControlPoint(index, "radius", Math.max(0.01, Number(e.target.value)))
                    }
                    data-testid={`control-point-${index}-radius`}
                    className={inputClass}
                    min={0.01}
                    step={0.5}
                  />
                  <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
                </div>
              </div>
              {control_points.length > 2 && (
                <button
                  onClick={() => removeControlPoint(index)}
                  data-testid={`control-point-${index}-remove`}
                  className="flex-shrink-0 rounded p-1 text-[var(--cad-text-muted)] hover:bg-[var(--cad-cancel)]/20 hover:text-[var(--cad-cancel)] transition-colors"
                  title="Remove point"
                >
                  <span className="text-xs">&times;</span>
                </button>
              )}
            </div>
          ))}
        </div>
        <p className="mt-1 text-[10px] text-[var(--cad-text-muted)]">
          Parameter t ranges from 0 (start) to 1 (end) along the edge.
        </p>
      </div>

      {/* Smooth Transition */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="variable-fillet-smooth"
            checked={smooth_transition}
            onChange={(e) => updateOperationParams({ smooth_transition: e.target.checked })}
            data-testid="variable-fillet-smooth"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="variable-fillet-smooth" className="text-xs text-[var(--cad-text-secondary)]">
            Smooth transition
          </label>
        </div>
        <p className="mt-0.5 pl-5 text-[10px] text-[var(--cad-text-muted)]">
          Blend smoothly between control point radii.
        </p>
      </div>
    </div>
  );
}
