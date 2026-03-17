import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

export function ScalePanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "scale") return null;

  const {
    uniform = true,
    scale_factor = 1,
    scale_x = 1,
    scale_y = 1,
    scale_z = 1,
    center = [0, 0, 0],
    copy = false,
  } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="scale-panel">
      {/* Uniform / Non-Uniform toggle */}
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="scale-uniform"
          checked={uniform}
          onChange={(e) => {
            const isUniform = e.target.checked;
            if (isUniform) {
              // When switching to uniform, sync all axes to scale_factor
              updateOperationParams({ uniform: true });
            } else {
              // When switching to non-uniform, populate axes from current scale_factor
              updateOperationParams({
                uniform: false,
                scale_x: scale_factor,
                scale_y: scale_factor,
                scale_z: scale_factor,
              });
            }
          }}
          data-testid="scale-uniform-toggle"
          className="rounded border-[var(--cad-border)]"
        />
        <label htmlFor="scale-uniform" className="text-xs text-[var(--cad-text-secondary)]">
          Uniform scaling
        </label>
      </div>

      {/* Scale Factor(s) */}
      {uniform ? (
        <div>
          <h4 className={sectionHeaderClass}>Scale Factor</h4>
          <input
            type="number"
            value={scale_factor}
            onChange={(e) => updateOperationParams({ scale_factor: Math.max(0.001, Number(e.target.value)) })}
            data-testid="scale-factor"
            className={inputClass}
            min={0.001}
            step={0.1}
          />
        </div>
      ) : (
        <div>
          <h4 className={sectionHeaderClass}>Scale Factors</h4>
          <div className="space-y-1.5">
            {(["x", "y", "z"] as const).map((axis) => {
              const key = `scale_${axis}` as const;
              const val = { x: scale_x, y: scale_y, z: scale_z }[axis];
              return (
                <div key={axis} className="flex items-center gap-1">
                  <span className="w-4 text-[10px] font-medium text-[var(--cad-text-muted)] uppercase">{axis}</span>
                  <input
                    type="number"
                    value={val}
                    onChange={(e) => updateOperationParams({ [key]: Math.max(0.001, Number(e.target.value)) })}
                    data-testid={`scale-${key}`}
                    className={inputClass}
                    min={0.001}
                    step={0.1}
                  />
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Center Point */}
      <div>
        <h4 className={sectionHeaderClass}>Center Point</h4>
        <div className="space-y-1.5">
          {(["x", "y", "z"] as const).map((axis, i) => (
            <div key={axis} className="flex items-center gap-1">
              <span className="w-4 text-[10px] font-medium text-[var(--cad-text-muted)] uppercase">{axis}</span>
              <input
                type="number"
                value={center[i]}
                onChange={(e) => {
                  const newCenter = [...center] as [number, number, number];
                  newCenter[i] = Number(e.target.value);
                  updateOperationParams({ center: newCenter });
                }}
                data-testid={`scale-center-${axis}`}
                className={inputClass}
                step={1}
              />
              <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Copy toggle */}
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="scale-copy"
          checked={copy}
          onChange={(e) => updateOperationParams({ copy: e.target.checked })}
          data-testid="scale-copy"
          className="rounded border-[var(--cad-border)]"
        />
        <label htmlFor="scale-copy" className="text-xs text-[var(--cad-text-secondary)]">
          Create copy (keep original)
        </label>
      </div>
    </div>
  );
}
