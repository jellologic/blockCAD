import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

const TRANSFORM_TYPES = [
  { label: "Translate", value: "translate" },
  { label: "Rotate", value: "rotate" },
  { label: "Translate + Rotate", value: "translate_rotate" },
] as const;

const AXIS_OPTIONS = [
  { label: "X Axis", value: [1, 0, 0] },
  { label: "Y Axis", value: [0, 1, 0] },
  { label: "Z Axis", value: [0, 0, 1] },
] as const;

export function MoveCopyPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "move_copy") return null;

  const {
    transform_type = "translate",
    translate_x = 0,
    translate_y = 0,
    translate_z = 0,
    rotate_axis_direction = [0, 0, 1],
    rotate_angle = 0,
    rotate_center = [0, 0, 0],
    copy = false,
  } = activeOperation.params;

  const showTranslate = transform_type === "translate" || transform_type === "translate_rotate";
  const showRotate = transform_type === "rotate" || transform_type === "translate_rotate";

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="move-copy-panel">
      {/* Transform Type */}
      <div>
        <h4 className={sectionHeaderClass}>Transform Type</h4>
        <select
          className={inputClass}
          value={transform_type}
          data-testid="move-copy-transform-type"
          onChange={(e) => updateOperationParams({ transform_type: e.target.value })}
        >
          {TRANSFORM_TYPES.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>

      {/* Translate */}
      {showTranslate && (
        <div>
          <h4 className={sectionHeaderClass}>Translation Distance</h4>
          <div className="space-y-1.5">
            {(["x", "y", "z"] as const).map((axis) => {
              const key = `translate_${axis}` as const;
              const val = { x: translate_x, y: translate_y, z: translate_z }[axis];
              return (
                <div key={axis} className="flex items-center gap-1">
                  <span className="w-4 text-[10px] font-medium text-[var(--cad-text-muted)] uppercase">{axis}</span>
                  <input
                    type="number"
                    value={val}
                    onChange={(e) => updateOperationParams({ [key]: Number(e.target.value) })}
                    data-testid={`move-copy-${key}`}
                    className={inputClass}
                    step={1}
                  />
                  <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Rotate */}
      {showRotate && (
        <div>
          <h4 className={sectionHeaderClass}>Rotation</h4>
          <div className="space-y-2">
            {/* Axis Direction */}
            <div>
              <label className="text-[10px] text-[var(--cad-text-muted)]">Axis Direction</label>
              <select
                className={inputClass}
                value={JSON.stringify(rotate_axis_direction)}
                data-testid="move-copy-rotate-axis"
                onChange={(e) => updateOperationParams({ rotate_axis_direction: JSON.parse(e.target.value) })}
              >
                {AXIS_OPTIONS.map((opt) => (
                  <option key={opt.label} value={JSON.stringify(opt.value)}>{opt.label}</option>
                ))}
              </select>
            </div>

            {/* Angle */}
            <div>
              <label className="text-[10px] text-[var(--cad-text-muted)]">Angle</label>
              <div className="flex items-center gap-1">
                <input
                  type="number"
                  value={rotate_angle}
                  onChange={(e) => updateOperationParams({ rotate_angle: Number(e.target.value) })}
                  data-testid="move-copy-rotate-angle"
                  className={inputClass}
                  step={5}
                />
                <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">deg</span>
              </div>
            </div>

            {/* Center Point */}
            <div>
              <label className="text-[10px] text-[var(--cad-text-muted)]">Center Point</label>
              <div className="space-y-1">
                {(["x", "y", "z"] as const).map((axis, i) => (
                  <div key={axis} className="flex items-center gap-1">
                    <span className="w-4 text-[10px] font-medium text-[var(--cad-text-muted)] uppercase">{axis}</span>
                    <input
                      type="number"
                      value={rotate_center[i]}
                      onChange={(e) => {
                        const newCenter = [...rotate_center] as [number, number, number];
                        newCenter[i] = Number(e.target.value);
                        updateOperationParams({ rotate_center: newCenter });
                      }}
                      data-testid={`move-copy-rotate-center-${axis}`}
                      className={inputClass}
                      step={1}
                    />
                    <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Copy toggle */}
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="mc-copy"
          checked={copy}
          onChange={(e) => updateOperationParams({ copy: e.target.checked })}
          data-testid="move-copy-copy"
          className="rounded border-[var(--cad-border)]"
        />
        <label htmlFor="mc-copy" className="text-xs text-[var(--cad-text-secondary)]">
          Create copy (keep original)
        </label>
      </div>
    </div>
  );
}
