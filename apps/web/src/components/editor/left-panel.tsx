import { useEditorStore } from "@/stores/editor-store";
import { FeatureTree } from "./feature-tree";
import { PropertyManager } from "./property-manager";

export function LeftPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);

  if (activeOperation) {
    return <PropertyManager />;
  }

  return <FeatureTree />;
}
