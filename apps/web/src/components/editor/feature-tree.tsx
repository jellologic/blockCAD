import { useState, useRef, useCallback, lazy, Suspense } from "react";
import {
  FileText, Gauge, StickyNote, Gem, Square, Crosshair,
  Pencil, Box, RotateCw, ChevronRight, ChevronDown,
  Scissors, Circle, Octagon, Grid3x3, FlipHorizontal2,
  Move, Scaling, Spline, Layers,
} from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

// Lazy-load context menu to avoid initialization order issues
const FeatureTreeMenu = lazy(() =>
  import("./feature-tree-menu").then((m) => ({ default: m.FeatureTreeMenu }))
);

const FEATURE_ICONS: Record<string, { icon: any; color: string }> = {
  sketch: { icon: Pencil, color: "var(--cad-icon-sketch)" },
  extrude: { icon: Box, color: "var(--cad-icon-feature)" },
  cut_extrude: { icon: Scissors, color: "var(--cad-icon-feature)" },
  revolve: { icon: RotateCw, color: "var(--cad-icon-feature)" },
  cut_revolve: { icon: RotateCw, color: "var(--cad-icon-feature)" },
  fillet: { icon: Circle, color: "var(--cad-icon-feature)" },
  chamfer: { icon: Octagon, color: "var(--cad-icon-feature)" },
  shell: { icon: Box, color: "var(--cad-icon-feature)" },
  linear_pattern: { icon: Grid3x3, color: "var(--cad-icon-feature)" },
  circular_pattern: { icon: RotateCw, color: "var(--cad-icon-feature)" },
  mirror: { icon: FlipHorizontal2, color: "var(--cad-icon-feature)" },
  move_copy: { icon: Move, color: "var(--cad-icon-feature)" },
  scale: { icon: Scaling, color: "var(--cad-icon-feature)" },
  sweep: { icon: Spline, color: "var(--cad-icon-feature)" },
  loft: { icon: Layers, color: "var(--cad-icon-feature)" },
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
function StatusDot({ suppressed }: { suppressed: boolean }) {
  if (suppressed) {
    return (
      <span
        className="inline-block h-2 w-2 rounded-full bg-yellow-500 flex-shrink-0"
        title="Suppressed"
      />
    );
  }
  return (
    <span
      className="inline-block h-2 w-2 rounded-full bg-green-500 flex-shrink-0"
      title="Evaluated"
    />
  );
}

interface ContextMenuState {
  x: number;
  y: number;
  featureId: string;
  featureIndex: number;
}

export function FeatureTree() {
  const features = useEditorStore((s) => s.features);
  const selectedFeatureId = useEditorStore((s) => s.selectedFeatureId);
  const selectFeature = useEditorStore((s) => s.selectFeature);
  const editFeature = useEditorStore((s) => s.editFeature);
  const renameFeature = useEditorStore((s) => s.renameFeature);
  const moveFeatureUp = useEditorStore((s) => s.moveFeatureUp);
  const moveFeatureDown = useEditorStore((s) => s.moveFeatureDown);
  const suppressFeature = useEditorStore((s) => s.suppressFeature);
  const unsuppressFeature = useEditorStore((s) => s.unsuppressFeature);
  const deleteFeature = useEditorStore((s) => s.deleteFeature);
  const rollbackTo = useEditorStore((s) => s.rollbackTo);
  const rollForward = useEditorStore((s) => s.rollForward);

  const [headerExpanded, setHeaderExpanded] = useState(false);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [renamingIndex, setRenamingIndex] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const renameInputRef = useRef<HTMLInputElement>(null);

  // Rollback bar state: index of the last visible feature (-1 means show all)
  const [rollbackIndex, setRollbackIndex] = useState<number>(-1);
  const rollbackDragging = useRef(false);
  const treeRef = useRef<HTMLDivElement>(null);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, featureId: string, index: number) => {
      e.preventDefault();
      selectFeature(featureId);
      setContextMenu({ x: e.clientX, y: e.clientY, featureId, featureIndex: index });
    },
    [selectFeature]
  );

  const startRename = useCallback(
    (index: number) => {
      const feature = features[index];
      if (!feature) return;
      setRenamingIndex(index);
      setRenameValue(feature.name);
      // Focus will happen via useEffect on the input
      setTimeout(() => renameInputRef.current?.select(), 0);
    },
    [features]
  );

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

  // Handle F2 key for rename
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, index: number) => {
      if (e.key === "F2") {
        e.preventDefault();
        startRename(index);
      } else if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        deleteFeature(index);
      }
    },
    [startRename, deleteFeature]
  );

  // Find the rollback position: last non-suppressed feature index
  const lastActiveIndex = features.reduce(
    (acc, f, i) => (f.suppressed ? acc : i),
    -1
  );

  // Rollback bar drag handlers
  const handleRollbackDragStart = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      rollbackDragging.current = true;

      const handleMove = (moveEvent: MouseEvent) => {
        if (!rollbackDragging.current || !treeRef.current) return;
        const featureButtons = treeRef.current.querySelectorAll("[data-feature-index]");
        let closestIndex = features.length - 1;
        let closestDist = Infinity;

        featureButtons.forEach((btn) => {
          const rect = btn.getBoundingClientRect();
          const mid = rect.top + rect.height / 2;
          const dist = Math.abs(moveEvent.clientY - mid);
          const idx = parseInt(btn.getAttribute("data-feature-index") || "0", 10);
          if (dist < closestDist) {
            closestDist = dist;
            closestIndex = idx;
          }
        });

        setRollbackIndex(closestIndex);
      };

      const handleUp = () => {
        rollbackDragging.current = false;
        document.removeEventListener("mousemove", handleMove);
        document.removeEventListener("mouseup", handleUp);

        // Apply rollback
        const idx = useEditorStore.getState().features.length - 1;
        const currentRollback = rollbackIndex >= 0 ? rollbackIndex : idx;
        if (currentRollback < idx) {
          rollbackTo(currentRollback);
        } else {
          rollForward();
        }
      };

      document.addEventListener("mousemove", handleMove);
      document.addEventListener("mouseup", handleUp);
    },
    [features.length, rollbackIndex, rollbackTo, rollForward]
  );

  // Compute effective rollback position for display
  const effectiveRollback = rollbackIndex >= 0 ? rollbackIndex : lastActiveIndex;

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
      <div ref={treeRef} className="flex-1 overflow-y-auto py-1">
        {features.map((feature, index) => {
          const iconConfig = FEATURE_ICONS[feature.type] || { icon: Box, color: "var(--cad-text-muted)" };
          const FeatureIcon = iconConfig.icon;
          const isSelected = selectedFeatureId === feature.id;
          const isSuppressed = feature.suppressed;
          const isRenaming = renamingIndex === index;

          return (
            <div key={feature.id}>
              <button
                data-testid={`feature-${feature.id}`}
                data-feature-index={index}
                onClick={() => selectFeature(feature.id)}
                onContextMenu={(e) => handleContextMenu(e, feature.id, index)}
                onDoubleClick={() => {
                  if (feature.type !== "sketch") editFeature(index);
                }}
                onKeyDown={(e) => handleKeyDown(e, index)}
                className={`flex w-full items-center gap-2 px-3 py-1 text-xs transition-colors ${
                  isSelected
                    ? "bg-[var(--cad-accent)]/15 text-[var(--cad-accent)]"
                    : "text-[var(--cad-text-secondary)] hover:bg-white/5 hover:text-[var(--cad-text-primary)]"
                } ${isSuppressed ? "opacity-40 line-through" : ""}`}
              >
                <StatusDot suppressed={isSuppressed} />
                <FeatureIcon
                  size={14}
                  style={{ color: isSuppressed ? "var(--cad-text-muted)" : iconConfig.color }}
                />
                {isRenaming ? (
                  <input
                    ref={renameInputRef}
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    onBlur={confirmRename}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        confirmRename();
                      } else if (e.key === "Escape") {
                        e.preventDefault();
                        cancelRename();
                      }
                      e.stopPropagation();
                    }}
                    onClick={(e) => e.stopPropagation()}
                    className="flex-1 min-w-0 bg-[var(--cad-bg-input)] border border-[var(--cad-accent)] rounded px-1 py-0 text-xs text-[var(--cad-text-primary)] outline-none"
                    autoFocus
                  />
                ) : (
                  <span className={`truncate text-left ${isSuppressed ? "line-through" : ""}`}>
                    {feature.name}
                  </span>
                )}
              </button>

              {/* Rollback bar after the effective rollback position */}
              {index === effectiveRollback && index < features.length - 1 && (
                <div
                  className="mx-2 my-0.5 flex items-center cursor-ns-resize group"
                  onMouseDown={handleRollbackDragStart}
                  title="Drag to rollback/roll forward features"
                >
                  <div className="flex-1 h-0.5 rounded bg-[var(--cad-accent)] group-hover:h-1 transition-all" />
                </div>
              )}
            </div>
          );
        })}

        {/* Rollback bar at the end (when all features are active) */}
        {features.length > 0 && effectiveRollback === features.length - 1 && (
          <div
            className="mx-2 my-0.5 flex items-center cursor-ns-resize group"
            onMouseDown={handleRollbackDragStart}
            title="Drag to rollback features"
          >
            <div className="flex-1 h-0.5 rounded bg-[var(--cad-accent)] group-hover:h-1 transition-all" />
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="border-t border-[var(--cad-border)] px-3 py-1.5">
        <span className="text-[10px] text-[var(--cad-text-muted)]" data-testid="feature-count">
          {features.length} feature{features.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Context menu (lazy-loaded) */}
      {contextMenu && (
        <Suspense fallback={null}>
          <FeatureTreeMenu
            x={contextMenu.x}
            y={contextMenu.y}
            featureId={contextMenu.featureId}
            featureIndex={contextMenu.featureIndex}
            isSuppressed={features[contextMenu.featureIndex]?.suppressed ?? false}
            isFirst={contextMenu.featureIndex === 0}
            isLast={contextMenu.featureIndex === features.length - 1}
            featureType={features[contextMenu.featureIndex]?.type ?? ""}
            onEdit={editFeature}
            onRename={startRename}
            onMoveUp={moveFeatureUp}
            onMoveDown={moveFeatureDown}
            onSuppress={suppressFeature}
            onUnsuppress={unsuppressFeature}
            onDelete={deleteFeature}
            onClose={() => setContextMenu(null)}
          />
        </Suspense>
      )}
    </div>
  );
}
