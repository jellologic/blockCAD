import { useEffect } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { useEditorStore } from "@/stores/editor-store";
import { CadViewport } from "@/components/viewport/cad-viewport";
import { FeatureTree } from "@/components/editor/feature-tree";
import { Toolbar } from "@/components/editor/toolbar";

export const Route = createFileRoute("/editor")({
  component: EditorPage,
});

function EditorPage() {
  const { isLoading, error, initKernel } = useEditorStore();

  useEffect(() => {
    initKernel();
  }, [initKernel]);

  if (error) {
    return (
      <div className="flex h-full items-center justify-center bg-[#1a1a2e] text-red-400">
        <p>Kernel error: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="grid h-full grid-cols-[280px_1fr] overflow-hidden">
      <FeatureTree />
      <div className="flex flex-col overflow-hidden">
        <Toolbar />
        <div className="relative flex-1">
          {isLoading ? (
            <div className="flex h-full items-center justify-center bg-[#1a1a2e]">
              <p className="text-white/40">Loading kernel...</p>
            </div>
          ) : (
            <CadViewport />
          )}
        </div>
      </div>
    </div>
  );
}
