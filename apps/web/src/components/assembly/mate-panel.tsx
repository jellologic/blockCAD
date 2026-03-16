import { useState } from "react";
import { Check, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

const MATE_TYPES = [
  { value: "coincident", label: "Coincident" },
  { value: "distance", label: "Distance", hasValue: true },
  { value: "angle", label: "Angle", hasValue: true },
  { value: "concentric", label: "Concentric" },
  { value: "parallel", label: "Parallel" },
  { value: "perpendicular", label: "Perpendicular" },
  { value: "tangent", label: "Tangent" },
  { value: "lock", label: "Lock" },
];

export function MatePanel() {
  const components = useAssemblyStore((s) => s.components);
  const addMate = useAssemblyStore((s) => s.addMate);
  const cancelOp = useAssemblyStore((s) => s.cancelOp);

  const [kind, setKind] = useState("coincident");
  const [compA, setCompA] = useState(components[0]?.id || "");
  const [compB, setCompB] = useState(components[1]?.id || "");
  const [faceA, setFaceA] = useState(0);
  const [faceB, setFaceB] = useState(0);
  const [value, setValue] = useState(5);

  const mateType = MATE_TYPES.find((m) => m.value === kind);
  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const handleConfirm = () => {
    if (!compA || !compB) return;
    addMate(kind, compA, compB, faceA, faceB, mateType?.hasValue ? value : undefined);
    cancelOp();
  };

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <span className="text-sm font-medium text-[var(--cad-text-primary)]">Add Mate</span>
        <div className="flex items-center gap-1">
          <button onClick={handleConfirm} data-testid="mate-confirm" className="rounded p-1 transition-colors hover:bg-[var(--cad-confirm)]/20">
            <Check size={18} style={{ color: "var(--cad-confirm)" }} />
          </button>
          <button onClick={cancelOp} data-testid="mate-cancel" className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20">
            <X size={18} style={{ color: "var(--cad-cancel)" }} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        <div>
          <label className={sectionHeaderClass}>Mate Type</label>
          <select value={kind} onChange={(e) => setKind(e.target.value)} className={inputClass} data-testid="mate-type-select">
            {MATE_TYPES.map((m) => <option key={m.value} value={m.value}>{m.label}</option>)}
          </select>
        </div>

        <div>
          <label className={sectionHeaderClass}>Component A</label>
          <select value={compA} onChange={(e) => setCompA(e.target.value)} className={inputClass} data-testid="mate-comp-a-select">
            {components.filter(c => !c.suppressed).map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
          <div className="mt-1">
            <span className="text-[9px] text-[var(--cad-text-muted)]">Face Index</span>
            <input type="number" value={faceA} onChange={(e) => setFaceA(Number(e.target.value))} className={inputClass} min={0} />
          </div>
        </div>

        <div>
          <label className={sectionHeaderClass}>Component B</label>
          <select value={compB} onChange={(e) => setCompB(e.target.value)} className={inputClass} data-testid="mate-comp-b-select">
            {components.filter(c => !c.suppressed).map((c) => <option key={c.id} value={c.id}>{c.name}</option>)}
          </select>
          <div className="mt-1">
            <span className="text-[9px] text-[var(--cad-text-muted)]">Face Index</span>
            <input type="number" value={faceB} onChange={(e) => setFaceB(Number(e.target.value))} className={inputClass} min={0} />
          </div>
        </div>

        {mateType?.hasValue && (
          <div>
            <label className={sectionHeaderClass}>{kind === "angle" ? "Angle (rad)" : "Distance"}</label>
            <input type="number" value={value} onChange={(e) => setValue(Number(e.target.value))} className={inputClass} step={kind === "angle" ? 0.1 : 1} />
          </div>
        )}
      </div>
    </div>
  );
}
