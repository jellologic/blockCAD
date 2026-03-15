import { useState } from "react";
import { useEditorStore } from "@/stores/editor-store";
import { ExtrudeDialog } from "./extrude-dialog";

function ToolbarButton({
  active,
  onClick,
  children,
}: {
  active?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`rounded px-3 py-1.5 text-xs font-medium transition-colors ${
        active
          ? "bg-blue-600/30 text-blue-300"
          : "text-white/60 hover:bg-white/10 hover:text-white"
      }`}
    >
      {children}
    </button>
  );
}

export function Toolbar() {
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);
  const toggleWireframe = useEditorStore((s) => s.toggleWireframe);
  const toggleEdges = useEditorStore((s) => s.toggleEdges);
  const mode = useEditorStore((s) => s.mode);
  const setMode = useEditorStore((s) => s.setMode);

  const [extrudeOpen, setExtrudeOpen] = useState(false);

  return (
    <>
      <div className="flex items-center gap-1 border-b border-white/10 bg-[#12121a] px-3 py-1.5">
        <span className="mr-2 text-xs text-white/30 uppercase tracking-wider">
          Create
        </span>
        <ToolbarButton onClick={() => setExtrudeOpen(true)}>
          Extrude
        </ToolbarButton>

        <div className="mx-2 h-4 w-px bg-white/10" />

        <span className="mr-2 text-xs text-white/30 uppercase tracking-wider">
          Select
        </span>
        <ToolbarButton
          active={mode === "select-face"}
          onClick={() =>
            setMode(mode === "select-face" ? "view" : "select-face")
          }
        >
          Face
        </ToolbarButton>

        <div className="mx-2 h-4 w-px bg-white/10" />

        <span className="mr-2 text-xs text-white/30 uppercase tracking-wider">
          Display
        </span>
        <ToolbarButton active={!wireframe} onClick={toggleWireframe}>
          Shaded
        </ToolbarButton>
        <ToolbarButton active={wireframe} onClick={toggleWireframe}>
          Wireframe
        </ToolbarButton>
        <div className="mx-2 h-4 w-px bg-white/10" />
        <ToolbarButton active={showEdges} onClick={toggleEdges}>
          Edges
        </ToolbarButton>
      </div>

      <ExtrudeDialog open={extrudeOpen} onClose={() => setExtrudeOpen(false)} />
    </>
  );
}
