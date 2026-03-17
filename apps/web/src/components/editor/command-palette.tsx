import { useState, useEffect, useRef, useMemo } from "react";
import { Search } from "lucide-react";
import { useEditorStore } from "@/stores/editor-store";

interface Command {
  id: string;
  name: string;
  shortcut?: string;
  category: string;
  action: () => void;
}

function buildCommands(): Command[] {
  const store = useEditorStore.getState();
  return [
    // Features
    { id: "sketch", name: "Sketch", shortcut: "S", category: "Features", action: () => store.startSketchFlow() },
    { id: "extrude", name: "Extrude", shortcut: "E", category: "Features", action: () => store.startOperation("extrude") },
    { id: "cut-extrude", name: "Cut Extrude", shortcut: "X", category: "Features", action: () => store.startOperation("cut_extrude") },
    { id: "revolve", name: "Revolve", shortcut: "V", category: "Features", action: () => store.startOperation("revolve") },
    { id: "cut-revolve", name: "Cut Revolve", category: "Features", action: () => store.startOperation("cut_revolve") },
    { id: "fillet", name: "Fillet", shortcut: "G", category: "Modify", action: () => store.startOperation("fillet") },
    { id: "chamfer", name: "Chamfer", shortcut: "H", category: "Modify", action: () => store.startOperation("chamfer") },
    { id: "shell", name: "Shell", category: "Modify", action: () => store.startOperation("shell") },
    { id: "linear-pattern", name: "Linear Pattern", category: "Pattern", action: () => store.startOperation("linear_pattern") },
    { id: "circular-pattern", name: "Circular Pattern", category: "Pattern", action: () => store.startOperation("circular_pattern") },
    { id: "mirror", name: "Mirror", category: "Pattern", action: () => store.startOperation("mirror") },
    { id: "sweep", name: "Sweep", category: "Features", action: () => store.startOperation("sweep") },
    { id: "loft", name: "Loft", category: "Features", action: () => store.startOperation("loft") },
    { id: "hole-wizard", name: "Hole Wizard", category: "Modify", action: () => store.startOperation("hole_wizard") },
    { id: "dome", name: "Dome", category: "Modify", action: () => store.startOperation("dome") },
    { id: "rib", name: "Rib", category: "Modify", action: () => store.startOperation("rib") },
    { id: "move-copy", name: "Move/Copy Body", category: "Transform", action: () => store.startOperation("move_copy") },
    { id: "scale", name: "Scale Body", category: "Transform", action: () => store.startOperation("scale") },
    // View
    { id: "wireframe", name: "Toggle Wireframe", shortcut: "W", category: "View", action: () => store.toggleWireframe() },
    { id: "edges", name: "Toggle Edges", category: "View", action: () => store.toggleEdges() },
    { id: "front-view", name: "Front View", shortcut: "1", category: "View", action: () => store.setCameraTarget([0, 0, 30]) },
    { id: "right-view", name: "Right View", shortcut: "3", category: "View", action: () => store.setCameraTarget([30, 0, 0]) },
    { id: "top-view", name: "Top View", shortcut: "5", category: "View", action: () => store.setCameraTarget([0, 30, 0]) },
    { id: "isometric", name: "Isometric View", shortcut: "0", category: "View", action: () => store.setCameraTarget([20, 15, 20]) },
    { id: "fit-all", name: "Fit All", shortcut: ".", category: "View", action: () => store.fitAll() },
    { id: "rebuild", name: "Rebuild", shortcut: "F5", category: "View", action: () => store.rebuild() },
    // Export
    { id: "export-stl", name: "Export STL", category: "Export", action: () => store.exportSTL(true) },
    { id: "export-obj", name: "Export OBJ", category: "Export", action: () => store.exportOBJ() },
    { id: "export-3mf", name: "Export 3MF", category: "Export", action: () => store.export3MF() },
    { id: "export-glb", name: "Export GLB", category: "Export", action: () => store.exportGLB() },
    { id: "export-step", name: "Export STEP", category: "Export", action: () => store.exportSTEP() },
    // Face selection
    { id: "select-face", name: "Select Face Mode", shortcut: "F", category: "Selection", action: () => store.setMode(store.mode === "select-face" ? "view" : "select-face") },
  ];
}

export function CommandPalette({ onClose }: { onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const commands = useMemo(() => buildCommands(), []);

  const filtered = useMemo(() => {
    if (!query.trim()) return commands;
    const q = query.toLowerCase();
    return commands.filter(
      (cmd) =>
        cmd.name.toLowerCase().includes(q) ||
        cmd.category.toLowerCase().includes(q) ||
        cmd.id.includes(q)
    );
  }, [commands, query]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  // Scroll selected item into view
  useEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const selected = list.children[selectedIndex] as HTMLElement;
    if (selected) {
      selected.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const cmd = filtered[selectedIndex];
      if (cmd) {
        cmd.action();
        onClose();
      }
    }
  };

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-[200] bg-black/40"
        onClick={onClose}
      />
      {/* Palette */}
      <div className="fixed top-[15%] left-1/2 z-[201] w-[480px] -translate-x-1/2 rounded-lg border border-[var(--cad-border)] bg-[var(--cad-bg-panel)] shadow-2xl overflow-hidden">
        {/* Search input */}
        <div className="flex items-center gap-2 border-b border-[var(--cad-border)] px-3 py-2">
          <Search size={16} className="text-[var(--cad-text-muted)]" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command..."
            className="flex-1 bg-transparent text-sm text-[var(--cad-text-primary)] outline-none placeholder:text-[var(--cad-text-muted)]"
          />
        </div>

        {/* Results */}
        <div ref={listRef} className="max-h-[300px] overflow-y-auto py-1">
          {filtered.length === 0 && (
            <div className="px-3 py-4 text-center text-xs text-[var(--cad-text-muted)]">
              No commands found
            </div>
          )}
          {filtered.map((cmd, i) => (
            <button
              key={cmd.id}
              onClick={() => {
                cmd.action();
                onClose();
              }}
              onMouseEnter={() => setSelectedIndex(i)}
              className={`flex w-full items-center justify-between px-3 py-1.5 text-xs transition-colors ${
                i === selectedIndex
                  ? "bg-[var(--cad-accent)]/15 text-[var(--cad-text-primary)]"
                  : "text-[var(--cad-text-secondary)] hover:bg-white/5"
              }`}
            >
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-[var(--cad-text-muted)] w-16">{cmd.category}</span>
                <span>{cmd.name}</span>
              </div>
              {cmd.shortcut && (
                <kbd className="rounded bg-white/10 px-1.5 py-0.5 text-[10px] text-[var(--cad-text-muted)]">
                  {cmd.shortcut}
                </kbd>
              )}
            </button>
          ))}
        </div>
      </div>
    </>
  );
}
