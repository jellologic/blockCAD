import { Plus, Trash2 } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

type GuideCurvePoint = [number, number, number];

type TangencyCondition =
  | { type: "None" }
  | { type: "Normal"; weight?: number }
  | { type: "Direction"; direction: [number, number, number]; weight?: number }
  | { type: "Weight"; weight: number };

const TANGENCY_OPTIONS = ["None", "Normal", "Direction", "Weight"] as const;

function TangencySection({
  label,
  testIdPrefix,
  value,
  onChange,
}: {
  label: string;
  testIdPrefix: string;
  value: TangencyCondition;
  onChange: (val: TangencyCondition) => void;
}) {
  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  return (
    <div>
      <h4 className={sectionHeaderClass}>{label}</h4>
      <select
        data-testid={`${testIdPrefix}-type`}
        className={inputClass}
        value={value.type}
        onChange={(e) => {
          const t = e.target.value as TangencyCondition["type"];
          switch (t) {
            case "None":
              onChange({ type: "None" });
              break;
            case "Normal":
              onChange({ type: "Normal", weight: 1 });
              break;
            case "Direction":
              onChange({ type: "Direction", direction: [0, 0, 1], weight: 1 });
              break;
            case "Weight":
              onChange({ type: "Weight", weight: 1 });
              break;
          }
        }}
      >
        {TANGENCY_OPTIONS.map((opt) => (
          <option key={opt} value={opt}>
            {opt}
          </option>
        ))}
      </select>

      {/* Direction inputs */}
      {value.type === "Direction" && (
        <div className="mt-1.5 space-y-1">
          <div className="flex items-center gap-1">
            {(["X", "Y", "Z"] as const).map((label, axis) => (
              <div key={label} className="flex items-center gap-0.5">
                <span className="text-[9px] text-[var(--cad-text-muted)]">{label}</span>
                <input
                  type="number"
                  value={value.direction[axis]}
                  onChange={(e) => {
                    const dir = [...value.direction] as [number, number, number];
                    dir[axis] = Number(e.target.value);
                    onChange({ ...value, direction: dir });
                  }}
                  data-testid={`${testIdPrefix}-dir-${label.toLowerCase()}`}
                  className="w-14 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-1 py-0.5 text-[10px] text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none"
                  step={0.1}
                />
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Weight input */}
      {(value.type === "Normal" || value.type === "Direction" || value.type === "Weight") && (
        <div className="mt-1.5 flex items-center gap-1">
          <span className="text-[9px] text-[var(--cad-text-muted)]">Weight</span>
          <input
            type="number"
            value={value.weight ?? 1}
            onChange={(e) => {
              onChange({ ...value, weight: Math.max(0.01, Number(e.target.value)) } as TangencyCondition);
            }}
            data-testid={`${testIdPrefix}-weight`}
            className="w-20 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-1 py-0.5 text-[10px] text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none"
            min={0.01}
            step={0.1}
          />
        </div>
      )}
    </div>
  );
}

export function LoftPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);

  if (!activeOperation || activeOperation.type !== "loft") return null;

  const {
    guide_curves = [] as GuideCurvePoint[][],
    start_tangency = { type: "None" } as TangencyCondition,
    end_tangency = { type: "None" } as TangencyCondition,
  } = activeOperation.params;

  const sectionHeaderClass =
    "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const addGuideCurve = () => {
    const newCurves = [...guide_curves, [[0, 0, 0], [10, 0, 0]] as GuideCurvePoint[]];
    updateOperationParams({ guide_curves: newCurves });
  };

  const removeGuideCurve = (index: number) => {
    const newCurves = guide_curves.filter((_: GuideCurvePoint[], i: number) => i !== index);
    updateOperationParams({ guide_curves: newCurves });
  };

  const addPointToCurve = (curveIndex: number) => {
    const newCurves = guide_curves.map((curve: GuideCurvePoint[], i: number) =>
      i === curveIndex ? [...curve, [0, 0, 0] as GuideCurvePoint] : curve
    );
    updateOperationParams({ guide_curves: newCurves });
  };

  const removePointFromCurve = (curveIndex: number, pointIndex: number) => {
    const newCurves = guide_curves.map((curve: GuideCurvePoint[], i: number) =>
      i === curveIndex ? curve.filter((_: GuideCurvePoint, j: number) => j !== pointIndex) : curve
    );
    updateOperationParams({ guide_curves: newCurves });
  };

  const updatePoint = (curveIndex: number, pointIndex: number, axis: number, value: number) => {
    const newCurves = guide_curves.map((curve: GuideCurvePoint[], i: number) =>
      i === curveIndex
        ? curve.map((pt: GuideCurvePoint, j: number) =>
            j === pointIndex
              ? (pt.map((v: number, k: number) => (k === axis ? value : v)) as GuideCurvePoint)
              : pt
          )
        : curve
    );
    updateOperationParams({ guide_curves: newCurves });
  };

  return (
    <div className="space-y-3" data-testid="loft-panel">
      {/* Start Tangency */}
      <TangencySection
        label="Start Tangency"
        testIdPrefix="loft-start-tangency"
        value={start_tangency}
        onChange={(val) => updateOperationParams({ start_tangency: val })}
      />

      {/* End Tangency */}
      <TangencySection
        label="End Tangency"
        testIdPrefix="loft-end-tangency"
        value={end_tangency}
        onChange={(val) => updateOperationParams({ end_tangency: val })}
      />

      {/* Guide Curves */}
      <div>
        <div className="flex items-center justify-between">
          <h4 className={sectionHeaderClass}>Guide Curves</h4>
          <button
            onClick={addGuideCurve}
            data-testid="loft-add-guide-curve"
            className="rounded p-0.5 text-[var(--cad-text-muted)] hover:bg-[var(--cad-bg-hover)] hover:text-[var(--cad-text-primary)] transition-colors"
            title="Add guide curve"
          >
            <Plus size={14} />
          </button>
        </div>
        {guide_curves.length === 0 && (
          <p className="text-[10px] text-[var(--cad-text-muted)]">No guide curves defined</p>
        )}
        {guide_curves.map((curve: GuideCurvePoint[], ci: number) => (
          <div
            key={ci}
            className="mt-2 space-y-1.5 border-l-2 border-[var(--cad-border)] pl-3"
          >
            <div className="flex items-center justify-between">
              <span className="text-[10px] font-medium text-[var(--cad-text-secondary)]">
                Curve {ci + 1}
              </span>
              <div className="flex items-center gap-1">
                <button
                  onClick={() => addPointToCurve(ci)}
                  data-testid={`loft-curve-${ci}-add-point`}
                  className="rounded p-0.5 text-[var(--cad-text-muted)] hover:bg-[var(--cad-bg-hover)] hover:text-[var(--cad-text-primary)] transition-colors"
                  title="Add point"
                >
                  <Plus size={12} />
                </button>
                <button
                  onClick={() => removeGuideCurve(ci)}
                  data-testid={`loft-curve-${ci}-remove`}
                  className="rounded p-0.5 text-[var(--cad-text-muted)] hover:bg-[var(--cad-cancel)]/20 hover:text-[var(--cad-cancel)] transition-colors"
                  title="Remove curve"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            </div>
            {curve.map((pt: GuideCurvePoint, pi: number) => (
              <div key={pi} className="flex items-center gap-1">
                <span className="w-4 text-[9px] text-[var(--cad-text-muted)]">P{pi + 1}</span>
                {(["X", "Y", "Z"] as const).map((label, axis) => (
                  <div key={label} className="flex items-center gap-0.5">
                    <span className="text-[9px] text-[var(--cad-text-muted)]">{label}</span>
                    <input
                      type="number"
                      value={pt[axis]}
                      onChange={(e) => updatePoint(ci, pi, axis, Number(e.target.value))}
                      data-testid={`loft-curve-${ci}-point-${pi}-${label.toLowerCase()}`}
                      className="w-14 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-1 py-0.5 text-[10px] text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none"
                      step={1}
                    />
                  </div>
                ))}
                {curve.length > 2 && (
                  <button
                    onClick={() => removePointFromCurve(ci, pi)}
                    data-testid={`loft-curve-${ci}-point-${pi}-remove`}
                    className="rounded p-0.5 text-[var(--cad-text-muted)] hover:bg-[var(--cad-cancel)]/20 hover:text-[var(--cad-cancel)] transition-colors"
                    title="Remove point"
                  >
                    <Trash2 size={10} />
                  </button>
                )}
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
