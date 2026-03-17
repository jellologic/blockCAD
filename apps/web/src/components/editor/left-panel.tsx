import { useEditorStore } from "@/stores/editor-store";
import { useAssemblyStore } from "@/stores/assembly-store";
import { FeatureTree } from "./feature-tree";
import { PropertyManager } from "./property-manager";
import { SketchPropertyPanel } from "@/components/sketch/sketch-property-panel";
import { AssemblyTreePanel } from "@/components/assembly/assembly-tree-panel";
import { ComponentInsertPanel } from "@/components/assembly/component-insert-panel";
import { MatePanel } from "@/components/assembly/mate-panel";
import { PatternPanel } from "@/components/assembly/pattern-panel";

export function LeftPanel() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const mode = useEditorStore((s) => s.mode);
  const isAssemblyMode = useAssemblyStore((s) => s.isAssemblyMode);
  const activeOp = useAssemblyStore((s) => s.activeOp);

  if (mode === "sketch") {
    return <SketchPropertyPanel />;
  }

  if (isAssemblyMode) {
    if (activeOp?.type === "insert-component") {
      return <ComponentInsertPanel />;
    }
    if (activeOp?.type === "add-mate" || activeOp?.type === "edit-mate") {
      return <MatePanel />;
    }
    if (activeOp?.type === "add-pattern") {
      return <PatternPanel />;
    }
    return <AssemblyTreePanel />;
  }

  if (activeOperation) {
    return <PropertyManager />;
  }

  return <FeatureTree />;
}
