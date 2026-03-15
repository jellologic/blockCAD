import { useState } from "react";
import {
  FileText, Gauge, StickyNote, Gem, Square, Crosshair,
  Pencil, Box, RotateCw, ChevronRight, ChevronDown,
} from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

const FEATURE_ICONS: Record<string, { icon: any; color: string }> = {
  sketch: { icon: Pencil, color: "var(--cad-icon-sketch)" },
  extrude: { icon: Box, color: "var(--cad-icon-feature)" },
  revolve: { icon: RotateCw, color: "var(--cad-icon-feature)" },
  fillet: { icon: Box, color: "var(--cad-icon-feature)" },
  chamfer: { icon: Box, color: "var(--cad-icon-feature)" },
};

function TreeItem({
  icon: Icon,
  color,
  label,
  indent = 0,
  muted = false,
}: {
  icon: any;
  color?: string;
  label: string;
  indent?: number;
  muted?: boolean;
}) {
  return (
    <div
      className={`flex items-center gap-2 px-2 py-0.5 text-xs ${
        muted ? "text-[var(--cad-text-muted)]" : "text-[var(--cad-text-secondary)]"
      }`}
      style={{ paddingLeft: `${8 + indent * 16}px` }}
    >
      <Icon size={14} style={color ? { color } : undefined} />
      <span>{label}</span>
    </div>
  );
}

export function FeatureTree() {
  const features = useEditorStore((s) => s.features);
  const selectedFeatureId = useEditorStore((s) => s.selectedFeatureId);
  const selectFeature = useEditorStore((s) => s.selectFeature);
  const [headerExpanded, setHeaderExpanded] = useState(false);

  const handleContextMenu = (e: React.MouseEvent, featureId: string, _index: number) => {
    e.preventDefault();
    // Context menu will be added later — for now just select
    selectFeature(featureId);
  };

  return (
    <div className="flex h-full flex-col bg-[var(--cad-bg-panel-alt)] border-r border-[var(--cad-border)] overflow-hidden">
      {/* Part header */}
      <div className="border-b border-[var(--cad-border)]">
        <button
          onClick={() => setHeaderExpanded(!headerExpanded)}
          className="flex w-full items-center gap-1.5 px-2 py-1.5 text-xs font-medium text-[var(--cad-text-primary)] hover:bg-white/5"
        >
          {headerExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
          <FileText size={14} className="text-[var(--cad-text-secondary)]" />
          <span>Part1</span>
        </button>
        {headerExpanded && (
          <div className="pb-1">
            <TreeItem icon={Gauge} label="Sensors" indent={1} muted />
            <TreeItem icon={StickyNote} label="Annotations" indent={1} muted />
            <TreeItem icon={Gem} label="Material <not specified>" indent={1} muted />
            <TreeItem icon={Square} label="Front Plane" indent={1} muted />
            <TreeItem icon={Square} label="Top Plane" indent={1} muted />
            <TreeItem icon={Square} label="Right Plane" indent={1} muted />
            <TreeItem icon={Crosshair} label="Origin" indent={1} muted />
          </div>
        )}
      </div>

      {/* Feature list */}
      <div className="flex-1 overflow-y-auto py-1">
        {features.map((feature, index) => {
          const iconConfig = FEATURE_ICONS[feature.type] || { icon: Box, color: "var(--cad-text-muted)" };
          const FeatureIcon = iconConfig.icon;
          const isSelected = selectedFeatureId === feature.id;
          const isSuppressed = feature.suppressed;

          return (
            <button
              key={feature.id}
              onClick={() => selectFeature(feature.id)}
              onContextMenu={(e) => handleContextMenu(e, feature.id, index)}
              className={`flex w-full items-center gap-2 px-3 py-1 text-xs transition-colors ${
                isSelected
                  ? "bg-[var(--cad-accent)]/15 text-[var(--cad-accent)]"
                  : "text-[var(--cad-text-secondary)] hover:bg-white/5 hover:text-[var(--cad-text-primary)]"
              } ${isSuppressed ? "opacity-40 line-through" : ""}`}
            >
              <FeatureIcon
                size={14}
                style={{ color: isSuppressed ? "var(--cad-text-muted)" : iconConfig.color }}
              />
              <span className="truncate text-left">{feature.name}</span>
            </button>
          );
        })}

        {/* Rollback bar (visual only) */}
        {features.length > 0 && (
          <div className="mx-2 my-1 h-0.5 rounded bg-[var(--cad-accent)]" />
        )}
      </div>

      {/* Footer */}
      <div className="border-t border-[var(--cad-border)] px-3 py-1.5">
        <span className="text-[10px] text-[var(--cad-text-muted)]">
          {features.length} feature{features.length !== 1 ? "s" : ""}
        </span>
      </div>
    </div>
  );
}
