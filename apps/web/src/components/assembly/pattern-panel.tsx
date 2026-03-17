import { useState } from "react";
import { Check, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

type PatternType = "linear" | "circular";

export function PatternPanel() {
  const components = useAssemblyStore((s) => s.components);
  const createLinearPattern = useAssemblyStore((s) => s.createLinearPattern);
  const createCircularPattern = useAssemblyStore((s) => s.createCircularPattern);
  const cancelOp = useAssemblyStore((s) => s.cancelOp);

  const [patternType, setPatternType] = useState<PatternType>("linear");
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  // Linear pattern fields
  const [dirX, setDirX] = useState(1);
  const [dirY, setDirY] = useState(0);
  const [dirZ, setDirZ] = useState(0);
  const [spacing, setSpacing] = useState(20);
  const [count, setCount] = useState(3);

  // Circular pattern fields
  const [originX, setOriginX] = useState(0);
  const [originY, setOriginY] = useState(0);
  const [originZ, setOriginZ] = useState(0);
  const [axisX, setAxisX] = useState(0);
  const [axisY, setAxisY] = useState(0);
  const [axisZ, setAxisZ] = useState(1);
  const [angleSpacing, setAngleSpacing] = useState(60);
  const [circCount, setCircCount] = useState(6);

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const activeComponents = components.filter((c) => !c.suppressed);

  const toggleComponent = (id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
    );
  };

  const handleConfirm = () => {
    if (selectedIds.length === 0) return;
    if (patternType === "linear") {
      createLinearPattern(selectedIds, [dirX, dirY, dirZ], spacing, count);
    } else {
      createCircularPattern(
        selectedIds,
        [originX, originY, originZ],
        [axisX, axisY, axisZ],
        angleSpacing,
        circCount,
      );
    }
    cancelOp();
  };

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)]">
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <span className="text-sm font-medium text-[var(--cad-text-primary)]">
          Assembly Pattern
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={handleConfirm}
            data-testid="pattern-confirm"
            className="rounded p-1 transition-colors hover:bg-[var(--cad-confirm)]/20"
            title="Apply Pattern"
            disabled={selectedIds.length === 0}
          >
            <Check size={18} style={{ color: "var(--cad-confirm)" }} />
          </button>
          <button
            onClick={cancelOp}
            data-testid="pattern-cancel"
            className="rounded p-1 transition-colors hover:bg-[var(--cad-cancel)]/20"
            title="Cancel"
          >
            <X size={18} style={{ color: "var(--cad-cancel)" }} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {/* Pattern Type */}
        <div>
          <label className={sectionHeaderClass}>Pattern Type</label>
          <select
            value={patternType}
            onChange={(e) => setPatternType(e.target.value as PatternType)}
            className={inputClass}
            data-testid="pattern-type-select"
          >
            <option value="linear">Linear</option>
            <option value="circular">Circular</option>
          </select>
        </div>

        {/* Source Components */}
        <div>
          <label className={sectionHeaderClass}>Source Components</label>
          {activeComponents.length === 0 ? (
            <p className="text-[10px] text-[var(--cad-text-muted)]">
              No components available
            </p>
          ) : (
            <div className="space-y-1 max-h-32 overflow-y-auto" data-testid="pattern-source-list">
              {activeComponents.map((c) => (
                <label
                  key={c.id}
                  className="flex items-center gap-2 text-xs text-[var(--cad-text-primary)] cursor-pointer hover:bg-white/5 rounded px-1 py-0.5"
                >
                  <input
                    type="checkbox"
                    checked={selectedIds.includes(c.id)}
                    onChange={() => toggleComponent(c.id)}
                    className="accent-[var(--cad-accent)]"
                    data-testid={`pattern-source-${c.id}`}
                  />
                  {c.name}
                </label>
              ))}
            </div>
          )}
        </div>

        {/* Linear fields */}
        {patternType === "linear" && (
          <>
            <div>
              <label className={sectionHeaderClass}>Direction</label>
              <div className="grid grid-cols-3 gap-1">
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">X</span>
                  <input
                    type="number"
                    value={dirX}
                    onChange={(e) => setDirX(Number(e.target.value))}
                    className={inputClass}
                    step={1}
                    data-testid="pattern-dir-x"
                  />
                </div>
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">Y</span>
                  <input
                    type="number"
                    value={dirY}
                    onChange={(e) => setDirY(Number(e.target.value))}
                    className={inputClass}
                    step={1}
                    data-testid="pattern-dir-y"
                  />
                </div>
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">Z</span>
                  <input
                    type="number"
                    value={dirZ}
                    onChange={(e) => setDirZ(Number(e.target.value))}
                    className={inputClass}
                    step={1}
                    data-testid="pattern-dir-z"
                  />
                </div>
              </div>
            </div>

            <div>
              <label className={sectionHeaderClass}>Spacing</label>
              <input
                type="number"
                value={spacing}
                onChange={(e) => setSpacing(Number(e.target.value))}
                className={inputClass}
                step={5}
                min={0.1}
                data-testid="pattern-spacing"
              />
            </div>

            <div>
              <label className={sectionHeaderClass}>Count</label>
              <input
                type="number"
                value={count}
                onChange={(e) => setCount(Math.max(2, Number(e.target.value)))}
                className={inputClass}
                min={2}
                data-testid="pattern-count"
              />
            </div>
          </>
        )}

        {/* Circular fields */}
        {patternType === "circular" && (
          <>
            <div>
              <label className={sectionHeaderClass}>Axis Origin</label>
              <div className="grid grid-cols-3 gap-1">
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">X</span>
                  <input
                    type="number"
                    value={originX}
                    onChange={(e) => setOriginX(Number(e.target.value))}
                    className={inputClass}
                    step={5}
                    data-testid="pattern-origin-x"
                  />
                </div>
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">Y</span>
                  <input
                    type="number"
                    value={originY}
                    onChange={(e) => setOriginY(Number(e.target.value))}
                    className={inputClass}
                    step={5}
                    data-testid="pattern-origin-y"
                  />
                </div>
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">Z</span>
                  <input
                    type="number"
                    value={originZ}
                    onChange={(e) => setOriginZ(Number(e.target.value))}
                    className={inputClass}
                    step={5}
                    data-testid="pattern-origin-z"
                  />
                </div>
              </div>
            </div>

            <div>
              <label className={sectionHeaderClass}>Axis Direction</label>
              <div className="grid grid-cols-3 gap-1">
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">X</span>
                  <input
                    type="number"
                    value={axisX}
                    onChange={(e) => setAxisX(Number(e.target.value))}
                    className={inputClass}
                    step={1}
                    data-testid="pattern-axis-x"
                  />
                </div>
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">Y</span>
                  <input
                    type="number"
                    value={axisY}
                    onChange={(e) => setAxisY(Number(e.target.value))}
                    className={inputClass}
                    step={1}
                    data-testid="pattern-axis-y"
                  />
                </div>
                <div>
                  <span className="text-[9px] text-[var(--cad-text-muted)]">Z</span>
                  <input
                    type="number"
                    value={axisZ}
                    onChange={(e) => setAxisZ(Number(e.target.value))}
                    className={inputClass}
                    step={1}
                    data-testid="pattern-axis-z"
                  />
                </div>
              </div>
            </div>

            <div>
              <label className={sectionHeaderClass}>Angle Spacing (deg)</label>
              <input
                type="number"
                value={angleSpacing}
                onChange={(e) => setAngleSpacing(Number(e.target.value))}
                className={inputClass}
                step={15}
                min={1}
                max={360}
                data-testid="pattern-angle-spacing"
              />
            </div>

            <div>
              <label className={sectionHeaderClass}>Count</label>
              <input
                type="number"
                value={circCount}
                onChange={(e) => setCircCount(Math.max(2, Number(e.target.value)))}
                className={inputClass}
                min={2}
                data-testid="pattern-circ-count"
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
