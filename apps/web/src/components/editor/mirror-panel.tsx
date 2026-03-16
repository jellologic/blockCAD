import { useEditorStore } from "@/stores/editor-store";

const PLANE_OPTIONS = [
  { label: "Front (XY)", origin: [0, 0, 0], normal: [0, 0, 1] },
  { label: "Top (XZ)", origin: [0, 0, 0], normal: [0, 1, 0] },
  { label: "Right (YZ)", origin: [0, 0, 0], normal: [1, 0, 0] },
] as const;

export function MirrorPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);

  if (!activeOperation || activeOperation.type !== "mirror") return null;

  const { plane_normal = [1, 0, 0] } = activeOperation.params;

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div className="space-y-3" data-testid="mirror-panel">
      <div>
        <h4 className={sectionHeaderClass}>Mirror Plane</h4>
        <select className={inputClass} value={JSON.stringify(plane_normal)} onChange={(e) => { const opt = PLANE_OPTIONS.find((o) => JSON.stringify(o.normal) === e.target.value); if (opt) { updateOperationParams({ plane_origin: [...opt.origin], plane_normal: [...opt.normal] }); } }}>
          {PLANE_OPTIONS.map((opt) => (
            <option key={opt.label} value={JSON.stringify(opt.normal)}>{opt.label}</option>
          ))}
        </select>
      </div>
    </div>
  );
}
