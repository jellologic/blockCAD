import { Plus, Trash2 } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

type GuideCurvePoint = [number, number, number];

export function SweepPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const updateOperationParams = useEditorStore((s) => s.updateOperationParams);

  if (!activeOperation || activeOperation.type !== "sweep") return null;

  const {
    guide_curves = [] as GuideCurvePoint[][],
    orientation = "FollowPath",
    total_twist = 0,
  } = activeOperation.params;

  const inputClass =
    "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
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

  const twistDegrees = Math.round((total_twist * 180) / Math.PI * 100) / 100;

  return (
    <div className="space-y-3" data-testid="sweep-panel">
      {/* Orientation Mode */}
      <div>
        <h4 className={sectionHeaderClass}>Orientation</h4>
        <select
          data-testid="sweep-orientation"
          className={inputClass}
          value={orientation}
          onChange={(e) => updateOperationParams({ orientation: e.target.value })}
        >
          <option value="FollowPath">Follow Path</option>
          <option value="KeepNormal">Keep Normal</option>
          <option value="FollowPathAndGuide">Follow Path + Guide</option>
          <option value="TwistAlongPath">Twist Along Path</option>
        </select>
      </div>

      {/* Twist Angle (only for TwistAlongPath) */}
      {orientation === "TwistAlongPath" && (
        <div>
          <label className={sectionHeaderClass}>Total Twist</label>
          <div className="flex items-center gap-1">
            <input
              type="number"
              value={twistDegrees}
              onChange={(e) => {
                const deg = Number(e.target.value);
                updateOperationParams({ total_twist: (deg * Math.PI) / 180 });
              }}
              data-testid="sweep-total-twist"
              className={inputClass}
              step={5}
            />
            <span className="flex-shrink-0 text-[10px] text-[var(--cad-text-muted)]">deg</span>
          </div>
        </div>
      )}

      {/* Guide Curves */}
      <div>
        <div className="flex items-center justify-between">
          <h4 className={sectionHeaderClass}>Guide Curves</h4>
          <button
            onClick={addGuideCurve}
            data-testid="sweep-add-guide-curve"
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
                  data-testid={`sweep-curve-${ci}-add-point`}
                  className="rounded p-0.5 text-[var(--cad-text-muted)] hover:bg-[var(--cad-bg-hover)] hover:text-[var(--cad-text-primary)] transition-colors"
                  title="Add point"
                >
                  <Plus size={12} />
                </button>
                <button
                  onClick={() => removeGuideCurve(ci)}
                  data-testid={`sweep-curve-${ci}-remove`}
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
                      data-testid={`sweep-curve-${ci}-point-${pi}-${label.toLowerCase()}`}
                      className="w-14 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-1 py-0.5 text-[10px] text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none"
                      step={1}
                    />
                  </div>
                ))}
                {curve.length > 2 && (
                  <button
                    onClick={() => removePointFromCurve(ci, pi)}
                    data-testid={`sweep-curve-${ci}-point-${pi}-remove`}
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
