import { useEditorStore } from "@/stores/editor-store";

export function ExtrudePanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);

  if (!activeOperation || activeOperation.type !== "extrude") return null;

  const { depth = 10, symmetric = false, draft_angle = 0 } = activeOperation.params;

  return (
    <div className="space-y-4">
      {/* Direction section */}
      <div>
        <h4 className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
          Direction
        </h4>
        <select
          className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-primary)]"
          defaultValue="blind"
        >
          <option value="blind">Blind</option>
          <option value="through_all" disabled>Through All</option>
        </select>
      </div>

      {/* Depth */}
      <div>
        <label className="mb-1 block text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
          Depth
        </label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={depth}
            onChange={(e) => updateOperationParams({ depth: Number(e.target.value) })}
            className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-primary)]"
            min={0.1}
            step={0.5}
          />
          <span className="text-[10px] text-[var(--cad-text-muted)]">mm</span>
        </div>
      </div>

      {/* Symmetric */}
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="symmetric"
          checked={symmetric}
          onChange={(e) => updateOperationParams({ symmetric: e.target.checked })}
          className="rounded border-[var(--cad-border)]"
        />
        <label htmlFor="symmetric" className="text-xs text-[var(--cad-text-secondary)]">
          Symmetric
        </label>
      </div>

      {/* Draft angle */}
      <div>
        <label className="mb-1 block text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
          Draft Angle
        </label>
        <div className="flex items-center gap-1">
          <input
            type="number"
            value={draft_angle}
            onChange={(e) => updateOperationParams({ draft_angle: Number(e.target.value) })}
            className="w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1.5 text-xs text-[var(--cad-text-primary)]"
            min={0}
            max={89}
            step={0.5}
          />
          <span className="text-[10px] text-[var(--cad-text-muted)]">°</span>
        </div>
      </div>
    </div>
  );
}
