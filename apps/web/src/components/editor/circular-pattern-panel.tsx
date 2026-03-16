import { useEditorStore } from "@/stores/editor-store";

const AXIS_OPTIONS = [
  { label: "X Axis", direction: [1, 0, 0] },
  { label: "Y Axis", direction: [0, 1, 0] },
  { label: "Z Axis", direction: [0, 0, 1] },
] as const;

export function CircularPatternPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);

  if (!activeOperation || activeOperation.type !== "circular_pattern") return null;

  const {
    axis_direction = [0, 0, 1],
    count = 4,
    total_angle = Math.PI * 2,
  } = activeOperation.params;

  const angleDegrees = Math.round((total_angle * 180) / Math.PI * 100) / 100;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="circular-pattern-panel">
      <div>
        <h4 className={sectionHeaderClass}>Axis</h4>
        <select className={inputClass} value={JSON.stringify(axis_direction)} onChange={(e) => updateOperationParams({ axis_direction: JSON.parse(e.target.value), axis_origin: [0, 0, 0] })}>
          {AXIS_OPTIONS.map((opt) => (
            <option key={opt.label} value={JSON.stringify(opt.direction)}>{opt.label}</option>
          ))}
        </select>
      </div>

      <div>
        <label className={sectionHeaderClass}>Count</label>
        <input type="number" value={count} onChange={(e) => updateOperationParams({ count: Math.max(2, Math.round(Number(e.target.value))) })} className={inputClass} min={2} step={1} />
      </div>

      <div>
        <label className={sectionHeaderClass}>Total Angle</label>
        <div className="flex items-center gap-1">
          <input type="number" value={angleDegrees} onChange={(e) => { const deg = Math.min(360, Math.max(1, Number(e.target.value))); updateOperationParams({ total_angle: (deg * Math.PI) / 180 }); }} className={inputClass} min={1} max={360} step={1} />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">°</span>
        </div>
      </div>
    </div>
  );
}
