import {
  Square,
  Box,
  Maximize2,
  Layers,
  Network,
  Eye,
  ArrowUp,
  ArrowRight,
  CircleDot,
} from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

function HudButton({
  icon: Icon,
  title,
  active,
  onClick,
}: {
  icon: any;
  title: string;
  active?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className={`flex h-7 w-7 items-center justify-center rounded transition-colors ${
        active
          ? "bg-[var(--cad-accent)]/30 text-[var(--cad-accent)]"
          : "text-white/70 hover:bg-white/20 hover:text-white"
      }`}
    >
      <Icon size={16} />
    </button>
  );
}

export function HeadsUpToolbar() {
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);
  const toggleWireframe = useEditorStore((s) => s.toggleWireframe);
  const toggleEdges = useEditorStore((s) => s.toggleEdges);
  const mode = useEditorStore((s) => s.mode);
  const setMode = useEditorStore((s) => s.setMode);
  const setCameraTarget = useEditorStore((s) => s.setCameraTarget);
  const fitAll = useEditorStore((s) => s.fitAll);

  return (
    <div className="absolute left-1/2 top-2 z-10 flex -translate-x-1/2 items-center gap-0.5 rounded-md border border-white/10 bg-black/50 px-1.5 py-0.5 backdrop-blur-sm">
      {/* View orientations */}
      <HudButton icon={Square} title="Front (1)" onClick={() => setCameraTarget([0, 0, 30])} />
      <HudButton icon={ArrowUp} title="Top (5)" onClick={() => setCameraTarget([0, 30, 0])} />
      <HudButton icon={ArrowRight} title="Right (3)" onClick={() => setCameraTarget([30, 0, 0])} />
      <HudButton icon={Box} title="Isometric (0)" onClick={() => setCameraTarget([20, 15, 20])} />
      <HudButton icon={Maximize2} title="Fit All" onClick={fitAll} />

      <div className="mx-1 h-4 w-px bg-white/20" />

      {/* Display toggles */}
      <HudButton
        icon={wireframe ? Layers : Eye}
        title={wireframe ? "Wireframe (W)" : "Shaded (W)"}
        onClick={toggleWireframe}
      />
      <HudButton
        icon={Network}
        title="Edges"
        active={showEdges}
        onClick={toggleEdges}
      />

      <div className="mx-1 h-4 w-px bg-white/20" />

      {/* Selection mode */}
      <HudButton
        icon={CircleDot}
        title="Select Face (F)"
        active={mode === "select-face"}
        onClick={() => setMode(mode === "select-face" ? "view" : "select-face")}
      />
    </div>
  );
}
