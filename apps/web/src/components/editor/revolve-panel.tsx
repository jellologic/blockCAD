import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

const AXIS_OPTIONS = [
  { label: "X Axis", direction: [1, 0, 0] },
  { label: "Y Axis", direction: [0, 1, 0] },
  { label: "Z Axis", direction: [0, 0, 1] },
] as const;

function axisKey(dir: number[]): string {
  if (dir[0] === 1 && dir[1] === 0 && dir[2] === 0) return "x";
  if (dir[0] === 0 && dir[1] === 0 && dir[2] === 1) return "z";
  return "y";
}

export function RevolvePanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || (activeOperation.type !== "revolve" && activeOperation.type !== "cut_revolve")) return null;

  const {
    axis_direction = [0, 0, 1],
    angle = Math.PI * 2,
    direction2_enabled = false,
    angle2 = 0,
    symmetric = false,
    thin_feature = false,
    thin_wall_thickness = 1,
    flip_side_to_cut = false,
  } = activeOperation.params;

  const angleDegrees = Math.round((angle * 180) / Math.PI * 100) / 100;
  const angle2Degrees = Math.round((angle2 * 180) / Math.PI * 100) / 100;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="revolve-panel">
      {/* Axis section */}
      <div>
        <h4 className={sectionHeaderClass}>Axis</h4>
        <select
          data-testid="revolve-axis"
          className={inputClass}
          value={axisKey(axis_direction)}
          onChange={(e) => {
            const opt = AXIS_OPTIONS.find((o) => axisKey([...o.direction]) === e.target.value);
            if (opt) {
              updateOperationParams({
                axis_direction: [...opt.direction],
                axis_origin: [0, 0, 0],
              });
            }
          }}
        >
          {AXIS_OPTIONS.map((opt) => (
            <option key={opt.label} value={axisKey([...opt.direction])}>{opt.label}</option>
          ))}
        </select>
      </div>

      {/* Direction 1 Angle */}
      <div>
        <label className={sectionHeaderClass}>Angle</label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={angleDegrees}
            onChange={(e) => {
              const deg = Math.min(360, Math.max(1, Number(e.target.value)));
              updateOperationParams({ angle: (deg * Math.PI) / 180 });
            }}
            data-testid="revolve-angle"
            className={inputClass}
            min={1}
            max={360}
            step={1}
          />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">°</span>
        </div>
      </div>

      {/* Mid Plane (Symmetric) */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="revolve-symmetric"
            checked={symmetric}
            disabled={direction2_enabled}
            onChange={(e) => updateOperationParams({ symmetric: e.target.checked })}
            data-testid="revolve-symmetric"
            className="rounded border-[var(--cad-border)]"
          />
          <label
            htmlFor="revolve-symmetric"
            className={`text-xs ${direction2_enabled ? "text-[var(--cad-text-muted)]" : "text-[var(--cad-text-secondary)]"}`}
          >
            Mid Plane
          </label>
        </div>
        <p className="mt-0.5 pl-5 text-[10px] text-[var(--cad-text-muted)]">
          Revolve equally in both directions
        </p>
      </div>

      {/* Flip side to cut (cut_revolve only) */}
      {activeOperation.type === "cut_revolve" && (
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="revolve-flip-side"
            checked={flip_side_to_cut}
            onChange={(e) => updateOperationParams({ flip_side_to_cut: e.target.checked })}
            data-testid="revolve-flip-side-to-cut"
            className="rounded border-[var(--cad-border)]"
          />
          <label htmlFor="revolve-flip-side" className="text-xs text-[var(--cad-text-secondary)]">
            Flip side to cut
          </label>
        </div>
      )}

      {/* Direction 2 */}
      <div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="revolve-direction2-enabled"
            checked={direction2_enabled}
            disabled={symmetric}
            onChange={(e) => updateOperationParams({ direction2_enabled: e.target.checked })}
            data-testid="revolve-direction2-enabled"
            className="rounded border-[var(--cad-border)]"
          />
          <label
            htmlFor="revolve-direction2-enabled"
            className={`text-xs ${symmetric ? "text-[var(--cad-text-muted)]" : "text-[var(--cad-text-secondary)]"}`}
          >
            Direction 2
          </label>
        </div>
        <p className="mt-0.5 pl-5 text-[10px] text-[var(--cad-text-muted)]">
          Revolve in the opposite direction
        </p>

        {direction2_enabled && (
          <div className="mt-2 space-y-2 border-l-2 border-[var(--cad-border)] pl-3">
            <div>
              <label className={sectionHeaderClass}>Angle</label>
              <div className="flex items-center gap-1">
                <input
                  type="number"
                  value={angle2Degrees}
                  onChange={(e) => {
                    const deg = Math.min(360, Math.max(1, Number(e.target.value)));
                    updateOperationParams({ angle2: (deg * Math.PI) / 180 });
                  }}
                  data-testid="revolve-angle2"
                  className={inputClass}
                  min={1}
                  max={360}
                  step={1}
                />
                <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">°</span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Thin Feature */}
      <div>
        <h4 className={sectionHeaderClass}>Thin Feature</h4>
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="revolve-thin-feature"
              checked={thin_feature}
              onChange={(e) => updateOperationParams({ thin_feature: e.target.checked })}
              data-testid="revolve-thin-feature"
              className="rounded border-[var(--cad-border)]"
            />
            <label htmlFor="revolve-thin-feature" className="text-xs text-[var(--cad-text-secondary)]">
              Thin Feature
            </label>
          </div>
          {thin_feature && (
            <div className="flex items-center gap-1 pl-5">
              <input
                type="number"
                value={thin_wall_thickness}
                onChange={(e) => updateOperationParams({ thin_wall_thickness: Math.max(0.1, Number(e.target.value)) })}
                data-testid="revolve-thin-wall-thickness"
                className={inputClass}
                min={0.1}
                step={0.5}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
