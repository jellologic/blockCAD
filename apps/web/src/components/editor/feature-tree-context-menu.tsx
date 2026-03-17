import { useEffect, useRef } from "react";
import {
  Pencil, Trash2, ArrowUp, ArrowDown, EyeOff, Eye, Edit, CornerDownLeft,
} from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

interface ContextMenuProps {
  x: number;
  y: number;
  featureIndex: number;
  featureName: string;
  isSuppressed: boolean;
  isFirst: boolean;
  isLast: boolean;
  onClose: () => void;
  onStartRename: () => void;
}

function MenuItem({
  icon: Icon,
  label,
  onClick,
  disabled = false,
  destructive = false,
}: {
  icon: any;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  destructive?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`flex w-full items-center gap-2 px-3 py-1.5 text-xs transition-colors
        ${disabled
          ? "text-[var(--cad-text-muted)] cursor-not-allowed"
          : destructive
            ? "text-red-400 hover:bg-red-500/10"
            : "text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)]"
        }`}
    >
      <Icon size={13} />
      <span>{label}</span>
    </button>
  );
}

export function FeatureTreeContextMenu({
  x,
  y,
  featureIndex,
  featureName,
  isSuppressed,
  isFirst,
  isLast,
  onClose,
  onStartRename,
}: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);
  const deleteFeature = useEditorStore((s) => s.deleteFeature);
  const editFeature = useEditorStore((s) => s.editFeature);
  const moveFeatureUp = useEditorStore((s) => s.moveFeatureUp);
  const moveFeatureDown = useEditorStore((s) => s.moveFeatureDown);
  const suppressFeature = useEditorStore((s) => s.suppressFeature);
  const unsuppressFeature = useEditorStore((s) => s.unsuppressFeature);
  const rollbackTo = useEditorStore((s) => s.rollbackTo);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    }
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKey);
    };
  }, [onClose]);

  // Adjust position to keep menu in viewport
  const menuStyle: React.CSSProperties = {
    position: "fixed",
    left: x,
    top: y,
    zIndex: 1000,
  };

  return (
    <div
      ref={ref}
      style={menuStyle}
      className="min-w-[160px] rounded border border-[var(--cad-border)] bg-[var(--cad-bg)] py-1 shadow-lg"
      data-testid="feature-context-menu"
    >
      <MenuItem
        icon={Edit}
        label="Edit Feature"
        onClick={() => { editFeature(featureIndex); onClose(); }}
      />
      <MenuItem
        icon={Pencil}
        label="Rename"
        onClick={() => { onStartRename(); onClose(); }}
      />
      <div className="mx-2 my-1 border-t border-[var(--cad-border)]" />
      <MenuItem
        icon={ArrowUp}
        label="Move Up"
        onClick={() => { moveFeatureUp(featureIndex); onClose(); }}
        disabled={isFirst}
      />
      <MenuItem
        icon={ArrowDown}
        label="Move Down"
        onClick={() => { moveFeatureDown(featureIndex); onClose(); }}
        disabled={isLast}
      />
      <div className="mx-2 my-1 border-t border-[var(--cad-border)]" />
      {isSuppressed ? (
        <MenuItem
          icon={Eye}
          label="Unsuppress"
          onClick={() => { unsuppressFeature(featureIndex); onClose(); }}
        />
      ) : (
        <MenuItem
          icon={EyeOff}
          label="Suppress"
          onClick={() => { suppressFeature(featureIndex); onClose(); }}
        />
      )}
      <MenuItem
        icon={CornerDownLeft}
        label="Rollback to Here"
        onClick={() => { rollbackTo(featureIndex); onClose(); }}
      />
      <div className="mx-2 my-1 border-t border-[var(--cad-border)]" />
      <MenuItem
        icon={Trash2}
        label="Delete"
        destructive
        onClick={() => {
          if (window.confirm(`Delete "${featureName}"?`)) {
            deleteFeature(featureIndex);
          }
          onClose();
        }}
      />
    </div>
  );
}
