import { useState } from "react";
import { Ruler, Check, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

export function MeasurePanel() {
  const components = useAssemblyStore((s) => s.components);
  const measureDistance = useAssemblyStore((s) => s.measureDistance);
  const cancelOp = useAssemblyStore((s) => s.cancelOp);

  const [compA, setCompA] = useState(components[0]?.id || "");
  const [compB, setCompB] = useState(components[1]?.id || "");
  const [faceA, setFaceA] = useState(0);
  const [faceB, setFaceB] = useState(0);
  const [result, setResult] = useState<{ distance: number; point_a: number[]; point_b: number[] } | null>(null);

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const handleMeasure = () => {
    const r = measureDistance(compA, faceA, compB, faceB);
    setResult(r);
  };

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <div className="flex items-center gap-1">
          <Ruler size={14} />
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">Measure</span>
        </div>
        <div className="flex items-center gap-1">
          <button onClick={handleMeasure} data-testid="measure-confirm" className="rounded p-1 transition-colors hover:bg-[var(--cad-confirm)]/20">
            <Check size={18} style={{ color: "var(--cad-confirm)" }} />
          </button>
          <button onClick={cancelOp} data-testid="measure-cancel" className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20">
            <X size={18} style={{ color: "var(--cad-cancel)" }} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        <div>
          <label className={sectionHeaderClass}>Component A</label>
          <select value={compA} onChange={(e) => setCompA(e.target.value)} className={inputClass}>
            {components.filter((c) => !c.suppressed).map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
          <div className="mt-1">
            <span className="text-[9px] text-[var(--cad-text-muted)]">Face Index</span>
            <input type="number" value={faceA} onChange={(e) => setFaceA(Number(e.target.value))} className={inputClass} min={0} />
          </div>
        </div>

        <div>
          <label className={sectionHeaderClass}>Component B</label>
          <select value={compB} onChange={(e) => setCompB(e.target.value)} className={inputClass}>
            {components.filter((c) => !c.suppressed).map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
          <div className="mt-1">
            <span className="text-[9px] text-[var(--cad-text-muted)]">Face Index</span>
            <input type="number" value={faceB} onChange={(e) => setFaceB(Number(e.target.value))} className={inputClass} min={0} />
          </div>
        </div>

        {result && (
          <div className="rounded border border-[var(--cad-accent)]/30 bg-[var(--cad-accent)]/5 p-2" data-testid="measure-result">
            <div className="text-xs text-[var(--cad-text-primary)] font-medium">
              Distance: {result.distance.toFixed(4)}
            </div>
            <div className="text-[9px] text-[var(--cad-text-muted)] mt-1">
              A: ({result.point_a.map((v) => v.toFixed(2)).join(", ")})
            </div>
            <div className="text-[9px] text-[var(--cad-text-muted)]">
              B: ({result.point_b.map((v) => v.toFixed(2)).join(", ")})
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
