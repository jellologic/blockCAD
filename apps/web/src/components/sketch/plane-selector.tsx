import { useEditorStore } from "@/stores/editor-store";
import type { SketchPlaneId } from "@blockCAD/kernel";

interface PlaneSelectorProps {
  open: boolean;
  onClose: () => void;
}

const PLANES: { id: SketchPlaneId; label: string }[] = [
  { id: "front", label: "Front Plane" },
  { id: "top", label: "Top Plane" },
  { id: "right", label: "Right Plane" },
];

export function PlaneSelector({ open, onClose }: PlaneSelectorProps) {
  const enterSketchMode = useEditorStore((s) => s.enterSketchMode);

  if (!open) return null;

  return (
    <div className="absolute left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] p-3 shadow-xl">
      <h3 className="mb-2 text-sm font-medium text-[var(--cad-text-primary)]">
        Select Sketch Plane
      </h3>
      <div className="flex flex-col gap-1">
        {PLANES.map((p) => (
          <button
            key={p.id}
            data-testid={`plane-${p.id}`}
            onClick={() => {
              enterSketchMode(p.id);
              onClose();
            }}
            className="rounded px-3 py-1.5 text-left text-xs text-[var(--cad-text-secondary)] hover:bg-white/10 hover:text-[var(--cad-text-primary)]"
          >
            {p.label}
          </button>
        ))}
      </div>
    </div>
  );
}
