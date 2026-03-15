interface ToolbarProps {
  wireframe: boolean;
  showEdges: boolean;
  onToggleWireframe: () => void;
  onToggleEdges: () => void;
}

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

export function Toolbar({
  wireframe,
  showEdges,
  onToggleWireframe,
  onToggleEdges,
}: ToolbarProps) {
  return (
    <div className="flex items-center gap-1 border-b border-white/10 bg-[#12121a] px-3 py-1.5">
      <span className="mr-2 text-xs text-white/30 uppercase tracking-wider">
        Display
      </span>
      <ToolbarButton active={!wireframe} onClick={onToggleWireframe}>
        Shaded
      </ToolbarButton>
      <ToolbarButton active={wireframe} onClick={onToggleWireframe}>
        Wireframe
      </ToolbarButton>
      <div className="mx-2 h-4 w-px bg-white/10" />
      <ToolbarButton active={showEdges} onClick={onToggleEdges}>
        Edges
      </ToolbarButton>
    </div>
  );
}
