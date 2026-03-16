import { useEffect } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { useEditorStore } from "@/stores/editor-store";
import { CommandManager } from "@/components/editor/command-manager";
import { LeftPanel } from "@/components/editor/left-panel";
import { CadViewport } from "@/components/viewport/cad-viewport";
import { StatusBar } from "@/components/editor/status-bar";
import { useKeyboardShortcuts } from "@/hooks/use-keyboard-shortcuts";

export const Route = createFileRoute("/editor")({
  component: EditorPage,
});

function EditorPage() {
  const { isLoading, error, initKernel } = useEditorStore();

  useKeyboardShortcuts();

  useEffect(() => {
    initKernel();
  }, [initKernel]);

  // Expose store for e2e testing
  useEffect(() => {
    (window as any).__editorStore = useEditorStore;
  }, []);

  if (error) {
    return (
      <div className="flex h-svh items-center justify-center bg-[var(--cad-bg-viewport)] text-[var(--cad-icon-error)]">
        <p>Kernel error: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="grid h-svh grid-rows-[auto_1fr_24px]">
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
    </div>
  );
}
