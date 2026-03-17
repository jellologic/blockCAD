import { useState, useEffect } from "react";
import {
  Box, ChevronRight, ChevronDown, Link, EyeOff, Eye,
  Package, Trash2,
} from "lucide-react";
import { useAssemblyStore } from "@/stores/assembly-store";
import { ConfigPanel } from "./config-panel";

function SectionHeader({ label, count, expanded, onToggle }: {
  label: string; count: number; expanded: boolean; onToggle: () => void;
}) {
  return (
    <button
      onClick={onToggle}
      className="flex items-center gap-1 w-full px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--cad-text-muted)] hover:bg-white/5"
    >
      {expanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
      {label}
      <span className="ml-auto text-[9px] opacity-60">{count}</span>
    </button>
  );
}

/** Get DOF status color for a component */
function getDofColor(dofInfo: any): string | undefined {
  if (!dofInfo) return undefined;
  if (dofInfo.grounded) return "var(--cad-confirm)"; // green
  const status = dofInfo.status;
  if (status === "FullyConstrained") return "var(--cad-confirm)"; // green
  if (typeof status === "object" && "OverConstrained" in status) return "var(--cad-cancel)"; // red
  if (typeof status === "object" && "UnderConstrained" in status) return "#f59e0b"; // yellow/amber
  return undefined;
}

export function AssemblyTreePanel() {
  const parts = useAssemblyStore((s) => s.parts);
  const components = useAssemblyStore((s) => s.components);
  const mates = useAssemblyStore((s) => s.mates);
  const selectedComponentId = useAssemblyStore((s) => s.selectedComponentId);
  const selectComponent = useAssemblyStore((s) => s.selectComponent);
  const suppressComponent = useAssemblyStore((s) => s.suppressComponent);
  const unsuppressComponent = useAssemblyStore((s) => s.unsuppressComponent);
  const removeComponent = useAssemblyStore((s) => s.removeComponent);
  const dofAnalysis = useAssemblyStore((s) => s.dofAnalysis);
  const refreshDofAnalysis = useAssemblyStore((s) => s.refreshDofAnalysis);

  const [partsExpanded, setPartsExpanded] = useState(true);
  const [compsExpanded, setCompsExpanded] = useState(true);
  const [matesExpanded, setMatesExpanded] = useState(true);

  // Refresh DOF analysis when components or mates change
  useEffect(() => {
    refreshDofAnalysis();
  }, [components.length, mates.length]);

  const dofMap = new Map<string, any>();
  if (dofAnalysis) {
    for (const info of dofAnalysis) {
      dofMap.set(info.component_id, info);
    }
  }

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel)] border-r border-[var(--cad-border)]">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--cad-border)] px-3 py-2">
        <span className="text-sm font-medium text-[var(--cad-text-primary)]">Assembly</span>
        <span className="text-[10px] text-[var(--cad-text-muted)]">
          {components.length} components
        </span>
      </div>

      {/* Configuration panel */}
      <ConfigPanel />

      <div className="flex-1 overflow-y-auto py-1">
        {/* Parts section */}
        <SectionHeader label="Parts" count={parts.length} expanded={partsExpanded} onToggle={() => setPartsExpanded(!partsExpanded)} />
        {partsExpanded && parts.map((part) => (
          <div key={part.id} className="flex items-center gap-2 px-4 py-0.5 text-xs text-[var(--cad-text-secondary)]">
            <Package size={14} style={{ color: "var(--cad-icon-feature)" }} />
            <span>{part.name}</span>
            <span className="ml-auto text-[9px] text-[var(--cad-text-muted)]">{part.id}</span>
          </div>
        ))}
        {partsExpanded && parts.length === 0 && (
          <div className="px-4 py-1 text-[10px] text-[var(--cad-text-muted)] italic">No parts</div>
        )}

        {/* Components section */}
        <SectionHeader label="Components" count={components.length} expanded={compsExpanded} onToggle={() => setCompsExpanded(!compsExpanded)} />
        {compsExpanded && components.map((comp, index) => {
          const dofInfo = dofMap.get(comp.id);
          const dofColor = getDofColor(dofInfo);
          return (
            <div
              key={comp.id}
              onClick={() => selectComponent(comp.id)}
              className={`flex items-center gap-2 px-4 py-0.5 text-xs cursor-pointer transition-colors ${
                selectedComponentId === comp.id
                  ? "bg-[var(--cad-accent)]/15 text-[var(--cad-accent)]"
                  : comp.suppressed
                    ? "text-[var(--cad-text-muted)] opacity-50"
                    : "text-[var(--cad-text-secondary)] hover:bg-white/5"
              }`}
            >
              {dofColor && (
                <span
                  className="w-2 h-2 rounded-full flex-shrink-0"
                  style={{ backgroundColor: dofColor }}
                  title={dofInfo?.grounded ? "Grounded" : JSON.stringify(dofInfo?.status)}
                />
              )}
              <Box size={14} style={{ color: comp.suppressed ? undefined : "var(--cad-icon-feature)" }} />
              <span className={comp.suppressed ? "line-through" : ""}>{comp.name}</span>
              <span className="ml-auto text-[9px] text-[var(--cad-text-muted)]">{comp.partId}</span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  comp.suppressed ? unsuppressComponent(index) : suppressComponent(index);
                }}
                className="p-0.5 rounded hover:bg-white/10"
                title={comp.suppressed ? "Show" : "Hide"}
              >
                {comp.suppressed ? <EyeOff size={12} /> : <Eye size={12} />}
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  removeComponent(comp.id);
                }}
                className="p-0.5 rounded hover:bg-[var(--cad-cancel)]/20"
                title="Delete component"
                data-testid={`delete-component-${comp.id}`}
              >
                <Trash2 size={12} className="text-[var(--cad-cancel)]" />
              </button>
            </div>
          );
        })}
        {compsExpanded && components.length === 0 && (
          <div className="px-4 py-1 text-[10px] text-[var(--cad-text-muted)] italic">No components — insert from Parts</div>
        )}

        {/* Mates section */}
        <SectionHeader label="Mates" count={mates.length} expanded={matesExpanded} onToggle={() => setMatesExpanded(!matesExpanded)} />
        {matesExpanded && mates.map((mate) => (
          <div key={mate.id} className="flex items-center gap-2 px-4 py-0.5 text-xs text-[var(--cad-text-secondary)]">
            <Link size={14} style={{ color: "var(--cad-icon-sketch)" }} />
            <span>{mate.kind}: {mate.compA} <span className="opacity-50">-</span> {mate.compB}</span>
          </div>
        ))}
        {matesExpanded && mates.length === 0 && (
          <div className="px-4 py-1 text-[10px] text-[var(--cad-text-muted)] italic">No mates</div>
        )}
      </div>

      {/* Footer */}
      <div className="border-t border-[var(--cad-border)] px-3 py-1">
        <span data-testid="assembly-component-count" className="text-[10px] text-[var(--cad-text-muted)]">
          {components.filter(c => !c.suppressed).length} active / {components.length} total
        </span>
      </div>
    </div>
  );
}
