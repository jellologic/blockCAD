import { useState } from "react";
import { Check, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function ComponentInsertPanel() {
  const parts = useAssemblyStore((s) => s.parts);
  const insertComponent = useAssemblyStore((s) => s.insertComponent);
  const cancelOp = useAssemblyStore((s) => s.cancelOp);

  const [partId, setPartId] = useState(parts[0]?.id || "");
  const [name, setName] = useState("Component");
  const [x, setX] = useState(0);
  const [y, setY] = useState(0);
  const [z, setZ] = useState(0);

  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const handleConfirm = () => {
    if (!partId) return;
    insertComponent(partId, name, [x, y, z]);
    cancelOp();
  };

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <span className="text-sm font-medium text-[var(--cad-text-primary)]">Insert Component</span>
        <div className="flex items-center gap-1">
          <button onClick={handleConfirm} data-testid="insert-confirm" className="rounded p-1 transition-colors hover:bg-[var(--cad-confirm)]/20" title="Confirm">
            <Check size={18} style={{ color: "var(--cad-confirm)" }} />
          </button>
          <button onClick={cancelOp} data-testid="insert-cancel" className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20" title="Cancel">
            <X size={18} style={{ color: "var(--cad-cancel)" }} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        <div>
          <label className={sectionHeaderClass}>Part</label>
          <select
            value={partId}
            onChange={(e) => setPartId(e.target.value)}
            className={inputClass}
            data-testid="insert-part-select"
          >
            {parts.map((p) => (
              <option key={p.id} value={p.id}>{p.name}</option>
            ))}
          </select>
          {parts.length === 0 && (
            <p className="mt-1 text-[10px] text-[var(--cad-text-muted)]">No parts available — add a part first</p>
          )}
        </div>

        <div>
          <label className={sectionHeaderClass}>Name</label>
          <input type="text" value={name} onChange={(e) => setName(e.target.value)} className={inputClass} data-testid="insert-name" />
        </div>

        <div>
          <label className={sectionHeaderClass}>Position</label>
          <div className="grid grid-cols-3 gap-1">
            <div>
              <span className="text-[9px] text-[var(--cad-text-muted)]">X</span>
              <input type="number" value={x} onChange={(e) => setX(Number(e.target.value))} className={inputClass} step={5} />
            </div>
            <div>
              <span className="text-[9px] text-[var(--cad-text-muted)]">Y</span>
              <input type="number" value={y} onChange={(e) => setY(Number(e.target.value))} className={inputClass} step={5} />
            </div>
            <div>
              <span className="text-[9px] text-[var(--cad-text-muted)]">Z</span>
              <input type="number" value={z} onChange={(e) => setZ(Number(e.target.value))} className={inputClass} step={5} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
