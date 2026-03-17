import { useState, useEffect } from "react";
import { X, RefreshCw } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

function formatNum(n: number, decimals: number = 4): string {
  if (Math.abs(n) < 1e-10) return "0";
  return n.toFixed(decimals);
}

function formatVec3(v: [number, number, number], decimals: number = 4): string {
  return `(${formatNum(v[0], decimals)}, ${formatNum(v[1], decimals)}, ${formatNum(v[2], decimals)})`;
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
        {label}
      </span>
      <div className="rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-2 py-1.5">
        {children}
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-2 py-0.5">
      <span className="text-[10px] text-[var(--cad-text-muted)] shrink-0">{label}</span>
      <span className="text-[10px] text-[var(--cad-text-primary)] font-mono text-right">{value}</span>
    </div>
  );
}

interface MassPropertiesPanelProps {
  onClose: () => void;
}

export function MassPropertiesPanel({ onClose }: MassPropertiesPanelProps) {
  const massProperties = useEditorStore((s) => s.massProperties);
  const computeMassProperties = useEditorStore((s) => s.computeMassProperties);
  const meshData = useEditorStore((s) => s.meshData);

  const [density, setDensity] = useState(1.0);

  // Re-compute when model changes
  useEffect(() => {
    if (meshData) {
      computeMassProperties();
    }
  }, [meshData, computeMassProperties]);

  const props = massProperties;
  const mass = props ? props.volume * density : 0;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      data-testid="mass-properties-dialog"
    >
      <div className="w-[440px] max-h-[80vh] rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-2xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-4 py-3 shrink-0">
          <span className="text-sm font-medium text-[var(--cad-text-primary)]">
            Mass Properties
          </span>
          <div className="flex items-center gap-1">
            <button
              onClick={computeMassProperties}
              data-testid="mass-props-refresh"
              className="rounded p-1 hover:bg-white/10"
              title="Refresh"
            >
              <RefreshCw size={14} className="text-[var(--cad-text-muted)]" />
            </button>
            <button
              onClick={onClose}
              data-testid="mass-props-close"
              className="rounded p-1 hover:bg-white/10"
            >
              <X size={16} className="text-[var(--cad-text-muted)]" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
          {!props ? (
            <p className="py-4 text-center text-xs text-[var(--cad-text-muted)]">
              No model to analyze. Create geometry first.
            </p>
          ) : (
            <>
              {/* Density input */}
              <div className="flex items-center gap-2">
                <label className="text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)]">
                  Density
                </label>
                <input
                  type="number"
                  value={density}
                  min={0.001}
                  step={0.1}
                  onChange={(e) => setDensity(Math.max(0.001, parseFloat(e.target.value) || 1.0))}
                  data-testid="mass-props-density"
                  className="w-24 rounded border border-[var(--cad-border)] bg-[var(--cad-bg-panel-alt)] px-2 py-1 text-xs text-[var(--cad-text-primary)] focus:border-[var(--cad-accent)] focus:outline-none"
                />
                <span className="text-[9px] text-[var(--cad-text-muted)]">g/cm3</span>
              </div>

              {/* Volume & Surface Area */}
              <Section label="Volume & Surface Area">
                <Row label="Volume" value={`${formatNum(props.volume)} unit3`} />
                <Row label="Surface Area" value={`${formatNum(props.surface_area)} unit2`} />
                <Row label="Mass (Volume x Density)" value={`${formatNum(mass)} g`} />
              </Section>

              {/* Center of Mass */}
              <Section label="Center of Mass">
                <Row label="X" value={formatNum(props.center_of_mass[0])} />
                <Row label="Y" value={formatNum(props.center_of_mass[1])} />
                <Row label="Z" value={formatNum(props.center_of_mass[2])} />
              </Section>

              {/* Bounding Box */}
              <Section label="Bounding Box">
                <Row label="Min" value={formatVec3(props.bbox_min)} />
                <Row label="Max" value={formatVec3(props.bbox_max)} />
                <Row
                  label="Size"
                  value={formatVec3([
                    props.bbox_max[0] - props.bbox_min[0],
                    props.bbox_max[1] - props.bbox_min[1],
                    props.bbox_max[2] - props.bbox_min[2],
                  ])}
                />
              </Section>

              {/* Inertia Tensor */}
              <Section label="Inertia Tensor (about CoM, density-scaled)">
                <div className="font-mono text-[10px] text-[var(--cad-text-primary)]">
                  <table className="w-full" data-testid="inertia-tensor">
                    <tbody>
                      {props.inertia_tensor.map((row, i) => (
                        <tr key={i}>
                          {row.map((val, j) => (
                            <td key={j} className="px-1 py-0.5 text-right">
                              {formatNum(val * density, 6)}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </Section>

              {/* Principal Moments & Axes */}
              <Section label="Principal Moments of Inertia">
                <Row label="I1" value={formatNum(props.principal_moments[0] * density, 6)} />
                <Row label="I2" value={formatNum(props.principal_moments[1] * density, 6)} />
                <Row label="I3" value={formatNum(props.principal_moments[2] * density, 6)} />
              </Section>

              <Section label="Principal Axes">
                <Row label="Axis 1" value={formatVec3(props.principal_axes[0], 6)} />
                <Row label="Axis 2" value={formatVec3(props.principal_axes[1], 6)} />
                <Row label="Axis 3" value={formatVec3(props.principal_axes[2], 6)} />
              </Section>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
