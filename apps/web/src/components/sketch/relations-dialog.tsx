import { useEditorStore } from "@/stores/editor-store";
import type { SketchEntity2D } from "@blockCAD/kernel";
import {
  ArrowRightLeft,
  ArrowUpDown,
  ArrowLeftRight,
  Equal,
  Minus,
  Circle,
  MoveHorizontal,
  MoveVertical,
  X,
} from "lucide-react";

interface RelationDef {
  kind: string;
  label: string;
  icon: React.ReactNode;
  /** Number of entities required (by type) */
  requires: EntityRequirement;
}

type EntityRequirement =
  | { points: number }
  | { lines: number }
  | { points: 1; lines: 1 }
  | { lines: 2 }
  | { any: number };

const RELATIONS: RelationDef[] = [
  {
    kind: "coincident",
    label: "Coincident",
    icon: <Circle className="h-3.5 w-3.5" />,
    requires: { points: 2 },
  },
  {
    kind: "horizontal",
    label: "Horizontal",
    icon: <MoveHorizontal className="h-3.5 w-3.5" />,
    requires: { lines: 1 },
  },
  {
    kind: "vertical",
    label: "Vertical",
    icon: <MoveVertical className="h-3.5 w-3.5" />,
    requires: { lines: 1 },
  },
  {
    kind: "parallel",
    label: "Parallel",
    icon: <ArrowRightLeft className="h-3.5 w-3.5" />,
    requires: { lines: 2 },
  },
  {
    kind: "perpendicular",
    label: "Perpendicular",
    icon: <ArrowUpDown className="h-3.5 w-3.5" />,
    requires: { lines: 2 },
  },
  {
    kind: "equal",
    label: "Equal",
    icon: <Equal className="h-3.5 w-3.5" />,
    requires: { lines: 2 },
  },
  {
    kind: "collinear",
    label: "Collinear",
    icon: <Minus className="h-3.5 w-3.5" />,
    requires: { lines: 2 },
  },
  {
    kind: "midpoint",
    label: "Midpoint",
    icon: <ArrowLeftRight className="h-3.5 w-3.5" />,
    requires: { points: 3 },
  },
];

function countSelectedTypes(
  selectedIds: string[],
  entities: SketchEntity2D[]
): { points: number; lines: number; circles: number; arcs: number } {
  let points = 0;
  let lines = 0;
  let circles = 0;
  let arcs = 0;
  for (const id of selectedIds) {
    const entity = entities.find((e) => e.id === id);
    if (!entity) continue;
    switch (entity.type) {
      case "point":
        points++;
        break;
      case "line":
        lines++;
        break;
      case "circle":
        circles++;
        break;
      case "arc":
        arcs++;
        break;
    }
  }
  return { points, lines, circles, arcs };
}

function isApplicable(
  req: EntityRequirement,
  counts: { points: number; lines: number; circles: number; arcs: number }
): boolean {
  if ("points" in req && "lines" in req) {
    return counts.points >= req.points && counts.lines >= (req as any).lines;
  }
  if ("points" in req) return counts.points >= req.points;
  if ("lines" in req) return counts.lines >= req.lines;
  if ("any" in req) {
    return counts.points + counts.lines + counts.circles + counts.arcs >= req.any;
  }
  return false;
}

interface RelationsDialogProps {
  open: boolean;
  onClose: () => void;
  selectedEntityIds: string[];
}

export function RelationsDialog({
  open,
  onClose,
  selectedEntityIds,
}: RelationsDialogProps) {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const addSketchConstraint = useEditorStore((s) => s.addSketchConstraint);
  const genConstraintId = useEditorStore((s) => s.genSketchConstraintId);

  if (!open || !sketchSession || selectedEntityIds.length === 0) return null;

  const counts = countSelectedTypes(selectedEntityIds, sketchSession.entities);
  const applicable = RELATIONS.filter((r) => isApplicable(r.requires, counts));

  if (applicable.length === 0) return null;

  const handleAdd = (kind: string) => {
    const id = genConstraintId();
    addSketchConstraint({
      id,
      kind,
      entityIds: selectedEntityIds,
    });
    onClose();
  };

  return (
    <div className="absolute bottom-14 left-1/2 -translate-x-1/2 z-50 bg-[var(--cad-bg-panel)] border border-[var(--cad-border)] rounded-lg shadow-lg p-2">
      <div className="flex items-center justify-between mb-1.5 px-1">
        <span className="text-[11px] font-medium text-[var(--cad-text-secondary)]">
          Add Relation
        </span>
        <button
          onClick={onClose}
          className="p-0.5 hover:bg-[var(--cad-bg-hover)] rounded"
        >
          <X className="h-3 w-3 text-[var(--cad-text-secondary)]" />
        </button>
      </div>
      <div className="flex gap-1">
        {applicable.map((rel) => (
          <button
            key={rel.kind}
            onClick={() => handleAdd(rel.kind)}
            className="flex flex-col items-center gap-0.5 px-2 py-1.5 rounded hover:bg-[var(--cad-bg-hover)] text-[var(--cad-text-primary)] transition-colors"
            title={rel.label}
          >
            {rel.icon}
            <span className="text-[9px]">{rel.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
