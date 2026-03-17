import { useState } from "react";
import { Scissors, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

const PRESET_NORMALS: Record<string, [number, number, number]> = {
  "X": [1, 0, 0],
  "Y": [0, 1, 0],
  "Z": [0, 0, 1],
};

export function SectionPanel() {
  const setSectionPlane = useAssemblyStore((s) => s.setSectionPlane);
  const clearSectionPlane = useAssemblyStore((s) => s.clearSectionPlane);
  const hasSectionPlane = useAssemblyStore((s) => s.hasSectionPlane);

  const [axis, setAxis] = useState<"X" | "Y" | "Z">("X");
  const [offset, setOffset] = useState(0);

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";

  const handleApply = () => {
    const normal = PRESET_NORMALS[axis];
    setSectionPlane(normal, offset);
  };

  return (
    <div className="border-b border-[var(--cad-border)] px-3 py-2" data-testid="section-panel">
      <div className="flex items-center gap-1 mb-1">
        <Scissors size={12} className="text-[var(--cad-text-muted)]" />
        <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
          Section View
        </span>
      </div>

      <div className="flex gap-1 items-center mb-1">
        <span className="text-[9px] text-[var(--cad-text-muted)] w-10">Axis</span>
        <select
          value={axis}
          onChange={(e) => setAxis(e.target.value as "X" | "Y" | "Z")}
          className={inputClass}
          data-testid="section-axis"
        >
          <option value="X">X</option>
          <option value="Y">Y</option>
          <option value="Z">Z</option>
        </select>
      </div>

      <div className="flex gap-1 items-center mb-1">
        <span className="text-[9px] text-[var(--cad-text-muted)] w-10">Offset</span>
        <input
          type="range"
          min={-100}
          max={100}
          step={0.5}
          value={offset}
          onChange={(e) => setOffset(Number(e.target.value))}
          className="flex-1"
          data-testid="section-offset"
        />
        <span className="text-[9px] text-[var(--cad-text-muted)] w-8 text-right">{offset}</span>
      </div>

      <div className="flex gap-1">
        <button
          onClick={handleApply}
          className="flex-1 rounded bg-[var(--cad-accent)] px-2 py-0.5 text-[10px] text-white hover:opacity-80"
          data-testid="section-apply"
        >
          Apply
        </button>
        {hasSectionPlane && (
          <button
            onClick={clearSectionPlane}
            className="rounded border border-[var(--cad-border)] px-2 py-0.5 text-[10px] text-[var(--cad-text-muted)] hover:bg-white/5"
            data-testid="section-clear"
          >
            <X size={10} />
          </button>
        )}
      </div>
    </div>
  );
}
