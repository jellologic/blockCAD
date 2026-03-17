import { useState } from "react";
import { Check, X } from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";

interface MateTypeConfig {
  value: string;
  label: string;
  category: "standard" | "mechanical" | "advanced";
  /** Which parameter inputs to show */
  inputs?: string[];
}

const MATE_TYPES: MateTypeConfig[] = [
  // Standard mates
  { value: "coincident", label: "Coincident", category: "standard" },
  { value: "distance", label: "Distance", category: "standard", inputs: ["value"] },
  { value: "angle", label: "Angle", category: "standard", inputs: ["value"] },
  { value: "concentric", label: "Concentric", category: "standard" },
  { value: "parallel", label: "Parallel", category: "standard" },
  { value: "perpendicular", label: "Perpendicular", category: "standard" },
  { value: "tangent", label: "Tangent", category: "standard" },
  { value: "lock", label: "Lock", category: "standard" },
  // Mechanical mates
  { value: "hinge", label: "Hinge", category: "mechanical" },
  { value: "gear", label: "Gear", category: "mechanical", inputs: ["ratio"] },
  { value: "screw", label: "Screw", category: "mechanical", inputs: ["pitch"] },
  { value: "limit", label: "Limit", category: "mechanical", inputs: ["min", "max"] },
  { value: "rack_pinion", label: "Rack Pinion", category: "mechanical", inputs: ["pitch_radius"] },
  { value: "cam", label: "Cam", category: "mechanical", inputs: ["lift", "base_radius"] },
  { value: "universal_joint", label: "Universal Joint", category: "mechanical" },
  // Advanced mates
  { value: "width", label: "Width", category: "advanced" },
  { value: "symmetric", label: "Symmetric", category: "advanced" },
  { value: "slot", label: "Slot", category: "advanced", inputs: ["axis_x", "axis_y", "axis_z"] },
];

const INPUT_LABELS: Record<string, string> = {
  value: "Value",
  ratio: "Ratio",
  pitch: "Pitch",
  min: "Min",
  max: "Max",
  pitch_radius: "Pitch Radius",
  lift: "Lift",
  base_radius: "Base Radius",
  axis_x: "Axis X",
  axis_y: "Axis Y",
  axis_z: "Axis Z",
};

const INPUT_DEFAULTS: Record<string, number> = {
  value: 5,
  ratio: 1,
  pitch: 1,
  min: 0,
  max: 10,
  pitch_radius: 5,
  lift: 3,
  base_radius: 5,
  axis_x: 1,
  axis_y: 0,
  axis_z: 0,
};

const INPUT_STEPS: Record<string, number> = {
  value: 1,
  ratio: 0.1,
  pitch: 0.1,
  min: 1,
  max: 1,
  pitch_radius: 0.5,
  lift: 0.5,
  base_radius: 0.5,
  axis_x: 0.1,
  axis_y: 0.1,
  axis_z: 0.1,
};

const CATEGORY_LABELS: Record<string, string> = {
  standard: "Standard",
  mechanical: "Mechanical",
  advanced: "Advanced",
};

export function MatePanel() {
  const components = useAssemblyStore((s) => s.components);
  const addMate = useAssemblyStore((s) => s.addMate);
  const cancelOp = useAssemblyStore((s) => s.cancelOp);

  const [kind, setKind] = useState("coincident");
  const [compA, setCompA] = useState(components[0]?.id || "");
  const [compB, setCompB] = useState(components[1]?.id || "");
  const [faceA, setFaceA] = useState(0);
  const [faceB, setFaceB] = useState(0);
  const [inputValues, setInputValues] = useState<Record<string, number>>({ ...INPUT_DEFAULTS });

  const mateType = MATE_TYPES.find((m) => m.value === kind);
  const inputClass = "w-full rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none";
  const sectionHeaderClass = "mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]";

  const updateInput = (key: string, val: number) => {
    setInputValues((prev) => ({ ...prev, [key]: val }));
  };

  const handleConfirm = () => {
    if (!compA || !compB) return;

    const inputs = mateType?.inputs;

    if (!inputs || inputs.length === 0) {
      // No parameters — simple string kind
      addMate(kind, compA, compB, faceA, faceB);
    } else if (inputs.length === 1 && inputs[0] === "value") {
      // distance / angle — uses legacy value param
      addMate(kind, compA, compB, faceA, faceB, inputValues.value);
    } else if (kind === "slot") {
      // slot uses axis array
      const axis: [number, number, number] = [
        inputValues.axis_x ?? 1,
        inputValues.axis_y ?? 0,
        inputValues.axis_z ?? 0,
      ];
      addMate(kind, compA, compB, faceA, faceB, undefined, { axis });
    } else {
      // Other parameterized mates — build params object
      const params: Record<string, number> = {};
      for (const key of inputs) {
        params[key] = inputValues[key] ?? INPUT_DEFAULTS[key] ?? 0;
      }
      addMate(kind, compA, compB, faceA, faceB, undefined, params);
    }
    cancelOp();
  };

  // Group mate types by category for the optgroup display
  const categories = ["standard", "mechanical", "advanced"] as const;

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
            {categories.map((cat) => (
              <optgroup key={cat} label={CATEGORY_LABELS[cat]}>
                {MATE_TYPES.filter((m) => m.category === cat).map((m) => (
                  <option key={m.value} value={m.value}>{m.label}</option>
                ))}
              </optgroup>
            ))}
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

        {mateType?.inputs && mateType.inputs.length > 0 && (
          <div className="space-y-2" data-testid="mate-params-section">
            <label className={sectionHeaderClass}>Parameters</label>
            {mateType.inputs.map((inputKey) => (
              <div key={inputKey}>
                <span className="text-[9px] text-[var(--cad-text-muted)]">
                  {kind === "angle" && inputKey === "value" ? "Angle (degrees)" : INPUT_LABELS[inputKey] || inputKey}
                </span>
                <input
                  type="number"
                  data-testid={`mate-param-${inputKey}`}
                  value={inputValues[inputKey] ?? INPUT_DEFAULTS[inputKey] ?? 0}
                  onChange={(e) => updateInput(inputKey, Number(e.target.value))}
                  className={inputClass}
                  step={INPUT_STEPS[inputKey] ?? 1}
                />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
