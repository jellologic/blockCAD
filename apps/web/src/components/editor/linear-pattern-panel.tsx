import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

const DIR_OPTIONS = [
  { label: "X Axis", value: [1, 0, 0] },
  { label: "Y Axis", value: [0, 1, 0] },
  { label: "Z Axis", value: [0, 0, 1] },
] as const;

export function LinearPatternPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  if (!activeOperation || activeOperation.type !== "linear_pattern") return null;

  const {
    direction = [1, 0, 0],
    spacing = 10,
    count = 2,
    direction2 = null,
    spacing2 = 10,
    count2 = 2,
  } = activeOperation.params;

  const dir2Enabled = direction2 != null;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="linear-pattern-panel">
      {/* Direction 1 */}
      <div>
        <h4 className={sectionHeaderClass}>Direction 1</h4>
        <select
          className={inputClass}
          value={JSON.stringify(direction)}
          onChange={(e) => updateOperationParams({ direction: JSON.parse(e.target.value) })}
        >
          {DIR_OPTIONS.map((opt) => (
            <option key={opt.label} value={JSON.stringify(opt.value)}>{opt.label}</option>
          ))}
        </select>
      </div>

      {/* Spacing */}
      <div>
        <label className={sectionHeaderClass}>Spacing</label>
        <div className="flex items-center gap-1">
          <input type="number" value={spacing} onChange={(e) => updateOperationParams({ spacing: Math.max(0.1, Number(e.target.value)) })} className={inputClass} min={0.1} step={1} />
          <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
        </div>
      </div>

      {/* Count */}
      <div>
        <label className={sectionHeaderClass}>Count</label>
        <input type="number" value={count} onChange={(e) => updateOperationParams({ count: Math.max(1, Math.round(Number(e.target.value))) })} className={inputClass} min={1} step={1} />
      </div>

      {/* Direction 2 toggle */}
      <div>
        <div className="flex items-center gap-2">
          <input type="checkbox" id="lp-dir2" checked={dir2Enabled} onChange={(e) => updateOperationParams({ direction2: e.target.checked ? [0, 1, 0] : null })} className="rounded border-[var(--cad-border)]" />
          <label htmlFor="lp-dir2" className="text-xs text-[var(--cad-text-secondary)]">Direction 2</label>
        </div>
        {dir2Enabled && (
          <div className="mt-2 space-y-2 border-l-2 border-[var(--cad-border)] pl-3">
            <select className={inputClass} value={JSON.stringify(direction2)} onChange={(e) => updateOperationParams({ direction2: JSON.parse(e.target.value) })}>
              {DIR_OPTIONS.map((opt) => (
                <option key={opt.label} value={JSON.stringify(opt.value)}>{opt.label}</option>
              ))}
            </select>
            <div>
              <label className={sectionHeaderClass}>Spacing</label>
              <div className="flex items-center gap-1">
                <input type="number" value={spacing2} onChange={(e) => updateOperationParams({ spacing2: Math.max(0.1, Number(e.target.value)) })} className={inputClass} min={0.1} step={1} />
                <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">{unitSystem}</span>
              </div>
            </div>
            <div>
              <label className={sectionHeaderClass}>Count</label>
              <input type="number" value={count2} onChange={(e) => updateOperationParams({ count2: Math.max(1, Math.round(Number(e.target.value))) })} className={inputClass} min={1} step={1} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
