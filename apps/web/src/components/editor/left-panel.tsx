import { useEditorStore } from "@/stores/editor-store";
import { FeatureTree } from "./feature-tree";
import { PropertyManager } from "./property-manager";
import { SketchPropertyPanel } from "@/components/sketch/sketch-property-panel";

export function LeftPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const mode = useEditorStore((s) => s.mode);

  if (mode === "sketch") {
    return <SketchPropertyPanel />;
  }

  if (activeOperation) {
    return <PropertyManager />;
  }

  return <FeatureTree />;
}
