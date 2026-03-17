import { useState, useRef, useEffect, useCallback } from "react";
import {
  FileText, Gauge, StickyNote, Gem, Square, Crosshair,
  Pencil, Box, RotateCw, ChevronRight, ChevronDown,
} from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";
import { FeatureTreeContextMenu } from "./feature-tree-context-menu";

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

/** Status dot indicating feature evaluation state */
function StatusDot({ suppressed, state }: { suppressed: boolean; state?: string }) {
  let color = "#22c55e"; // green: evaluated / ok
  if (suppressed) {
    color = "#eab308"; // yellow: suppressed
  } else if (state === "failed") {
    color = "#ef4444"; // red: failed
  }
  return (
    <span
      className="inline-block h-1.5 w-1.5 flex-shrink-0 rounded-full"
      style={{ backgroundColor: color }}
      data-testid="feature-status-dot"
    />
  );
}

export function FeatureTree() {
  const features = useEditorStore((s) => s.features);
  const selectedFeatureId = useEditorStore((s) => s.selectedFeatureId);
  const selectFeature = useEditorStore((s) => s.selectFeature);
  const editFeature = useEditorStore((s) => s.editFeature);
  const renameFeature = useEditorStore((s) => s.renameFeature);
  const rollbackTo = useEditorStore((s) => s.rollbackTo);
  const rollForward = useEditorStore((s) => s.rollForward);
  const kernel = useEditorStore((s) => s.kernel);

  const [headerExpanded, setHeaderExpanded] = useState(false);
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    featureIndex: number;
  } | null>(null);
  const [renamingIndex, setRenamingIndex] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const renameInputRef = useRef<HTMLInputElement>(null);

  // Get cursor position from kernel for rollback bar
  const cursorIndex = kernel ? kernel.cursor : -1;

  // Listen for F2 rename event from keyboard shortcuts
  useEffect(() => {
    function handleRenameEvent(e: Event) {
      const detail = (e as CustomEvent).detail;
      if (detail?.featureId) {
        const idx = features.findIndex((f) => f.id === detail.featureId);
        if (idx >= 0) {
          startRename(idx);
        }
      }
    }
    window.addEventListener("blockcad:rename-feature", handleRenameEvent);
    return () => window.removeEventListener("blockcad:rename-feature", handleRenameEvent);
  }, [features, startRename]);

  // Focus rename input when it appears
  useEffect(() => {
    if (renamingIndex !== null && renameInputRef.current) {
      renameInputRef.current.focus();
      renameInputRef.current.select();
    }
  }, [renamingIndex]);

  const handleContextMenu = useCallback((e: React.MouseEvent, featureIndex: number) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, featureIndex });
  }, []);

  const handleDoubleClick = useCallback((index: number) => {
    editFeature(index);
  }, [editFeature]);

  const startRename = useCallback((index: number) => {
    setRenamingIndex(index);
    setRenameValue(features[index]?.name ?? "");
  }, [features]);

  const confirmRename = useCallback(() => {
    if (renamingIndex !== null && renameValue.trim()) {
      renameFeature(renamingIndex, renameValue.trim());
    }
    setRenamingIndex(null);
    setRenameValue("");
  }, [renamingIndex, renameValue, renameFeature]);

  const cancelRename = useCallback(() => {
    setRenamingIndex(null);
    setRenameValue("");
  }, []);

  // Determine if features are after the rollback cursor (grayed out)
  const isRolledBack = cursorIndex >= 0 && cursorIndex < features.length - 1;

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
          const isBeyondCursor = isRolledBack && index > cursorIndex;
          const isRenaming = renamingIndex === index;

          return (
            <div key={feature.id}>
              <button
                data-testid={`feature-${feature.id}`}
                onClick={() => selectFeature(feature.id)}
                onContextMenu={(e) => handleContextMenu(e, index)}
                onDoubleClick={() => handleDoubleClick(index)}
                className={`flex w-full items-center gap-2 px-3 py-1 text-xs transition-colors ${
                  isSelected
                    ? "bg-[var(--cad-accent)]/15 text-[var(--cad-accent)]"
                    : "text-[var(--cad-text-secondary)] hover:bg-white/5 hover:text-[var(--cad-text-primary)]"
                } ${isSuppressed ? "opacity-40 line-through" : ""} ${isBeyondCursor ? "opacity-30" : ""}`}
              >
                <StatusDot suppressed={isSuppressed} state={(feature as any).state} />
                <FeatureIcon
                  size={14}
                  style={{ color: isSuppressed ? "var(--cad-text-muted)" : iconConfig.color }}
                />
                {isRenaming ? (
                  <input
                    ref={renameInputRef}
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") confirmRename();
                      if (e.key === "Escape") cancelRename();
                      e.stopPropagation();
                    }}
                    onBlur={confirmRename}
                    className="flex-1 bg-transparent border border-[var(--cad-accent)] rounded px-1 py-0 text-xs text-[var(--cad-text-primary)] outline-none"
                    data-testid="feature-rename-input"
                  />
                ) : (
                  <span className="truncate text-left">{feature.name}</span>
                )}
              </button>

              {/* Rollback bar — show after the cursor feature */}
              {isRolledBack && index === cursorIndex && (
                <div
                  className="mx-2 my-0.5 flex items-center gap-1 cursor-pointer group"
                  onClick={() => rollForward()}
                  title="Double-click to roll forward"
                  data-testid="rollback-bar"
                >
                  <div className="flex-1 h-0.5 rounded bg-[var(--cad-accent)] group-hover:bg-[var(--cad-accent-hover)]" />
                  <span className="text-[9px] text-[var(--cad-accent)] group-hover:text-[var(--cad-accent-hover)]">
                    rollback
                  </span>
                </div>
              )}
            </div>
          );
        })}

        {/* Rollback bar at the end when fully rolled forward */}
        {features.length > 0 && !isRolledBack && (
          <div className="mx-2 my-1 h-0.5 rounded bg-[var(--cad-accent)]" />
        )}
      </div>

      {/* Context menu */}
      {contextMenu && (
        <FeatureTreeContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          featureIndex={contextMenu.featureIndex}
          featureName={features[contextMenu.featureIndex]?.name ?? ""}
          isSuppressed={features[contextMenu.featureIndex]?.suppressed ?? false}
          isFirst={contextMenu.featureIndex === 0}
          isLast={contextMenu.featureIndex === features.length - 1}
          onClose={() => setContextMenu(null)}
          onStartRename={() => startRename(contextMenu.featureIndex)}
        />
      )}

      {/* Footer */}
      <div className="border-t border-[var(--cad-border)] px-3 py-1.5">
        <span className="text-[10px] text-[var(--cad-text-muted)]" data-testid="feature-count">
          {features.length} feature{features.length !== 1 ? "s" : ""}
        </span>
      </div>
    </div>
  );
}
