import { useState } from "react";
import { createFileRoute } from "@tanstack/react-router";

import { useKernel } from "@/hooks/use-kernel";
import { CadViewport } from "@/components/viewport/cad-viewport";
import { FeatureTree } from "@/components/editor/feature-tree";
import { Toolbar } from "@/components/editor/toolbar";

export const Route = createFileRoute("/editor")({
  component: EditorPage,
});

function EditorPage() {
  const { meshData, features, isLoading, error } = useKernel();
  const [selectedFeature, setSelectedFeature] = useState<string | null>(null);
  const [wireframe, setWireframe] = useState(false);
  const [showEdges, setShowEdges] = useState(true);

  if (error) {
    return (
      <div className="flex h-full items-center justify-center bg-[#1a1a2e] text-red-400">
        <p>Kernel error: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="grid h-full grid-cols-[280px_1fr] overflow-hidden">
      <FeatureTree
        features={features}
        selectedId={selectedFeature}
        onSelect={setSelectedFeature}
      />
      <div className="flex flex-col overflow-hidden">
        <Toolbar
          wireframe={wireframe}
          showEdges={showEdges}
          onToggleWireframe={() => setWireframe((w) => !w)}
          onToggleEdges={() => setShowEdges((e) => !e)}
        />
        <div className="relative flex-1">
          {isLoading || !meshData ? (
            <div className="flex h-full items-center justify-center bg-[#1a1a2e]">
              <p className="text-white/40">Loading kernel...</p>
            </div>
          ) : (
            <CadViewport
              meshData={meshData}
              wireframe={wireframe}
              showEdges={showEdges}
            />
          )}
        </div>
      </div>
    </div>
  );
}
