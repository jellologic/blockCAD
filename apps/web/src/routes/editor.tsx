import { useEffect, useState, lazy, Suspense } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { useEditorStore } from "@/stores/editor-store";
import { useAssemblyStore } from "@/stores/assembly-store";
import { MenuBar } from "@/components/editor/menu-bar";
import { CommandManager } from "@/components/editor/command-manager";
import { LeftPanel } from "@/components/editor/left-panel";
import { CadViewport } from "@/components/viewport/cad-viewport";
import { StatusBar } from "@/components/editor/status-bar";
import { BomDialog } from "@/components/assembly/bom-dialog";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts";

const CommandPalette = lazy(() =>
  import("@/components/editor/command-palette").then((m) => ({ default: m.CommandPalette }))
);

export const Route = createFileRoute("/editor")({
  component: EditorPage,
});

function EditorPage() {
  const { isLoading, error, initKernel } = useEditorStore();
  const [showCommandPalette, setShowCommandPalette] = useState(false);

  useKeyboardShortcuts();

  useEffect(() => {
    initKernel();
  }, [initKernel]);

  // Expose stores for e2e testing
  useEffect(() => {
    (window as any).__editorStore = useEditorStore;
    (window as any).__assemblyStore = useAssemblyStore;
  }, []);

  // Ctrl+Shift+P to open command palette
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === "P") {
        e.preventDefault();
        setShowCommandPalette((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  if (error) {
    return (
      <div className="flex h-svh items-center justify-center bg-[var(--cad-bg-viewport)] text-[var(--cad-icon-error)]">
        <p>Kernel error: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="grid h-svh grid-rows-[auto_auto_1fr_24px]">
      <MenuBar />
      <CommandManager />
      <div className="grid grid-cols-[280px_1fr] overflow-hidden">
        <LeftPanel />
        <div className="relative overflow-hidden">
          {isLoading ? (
            <div className="flex h-full items-center justify-center bg-[var(--cad-bg-viewport)]">
              <p className="text-[var(--cad-text-muted)]">Loading kernel...</p>
            </div>
          ) : (
            <CadViewport />
          )}
        </div>
      </div>
      <StatusBar />
      <BomDialog />
      {showCommandPalette && (
        <Suspense fallback={null}>
          <CommandPalette onClose={() => setShowCommandPalette(false)} />
        </Suspense>
      )}
    </div>
  );
}
