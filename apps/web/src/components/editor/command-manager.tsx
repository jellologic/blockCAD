import { useState } from "react";
import {
  Box, RotateCw, Circle, Octagon, Grid3x3, RefreshCw, FlipHorizontal2,
  Minus, Square, Lock, MoveHorizontal, MoveVertical, Ruler,
  Eye, Layers, Network, Maximize2, BoxSelect,
} from "lucide-react";
import { RibbonButton } from "./ribbon-button";
import { useEditorStore } from "@/stores/editor-store";

type TabId = "features" | "sketch" | "view";

function RibbonGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center border-r border-[var(--cad-border)] px-2 last:border-r-0">
      <div className="flex items-center gap-0.5 py-1">{children}</div>
      <span className="text-[9px] text-[var(--cad-text-muted)] pb-0.5">{label}</span>
    </div>
  );
}

export function CommandManager() {
  const [activeTab, setActiveTab] = useState<TabId>("features");
  const startOperation = useEditorStore((s) => s.startOperation);
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);
  const toggleWireframe = useEditorStore((s) => s.toggleWireframe);
  const toggleEdges = useEditorStore((s) => s.toggleEdges);

  const tabs: { id: TabId; label: string }[] = [
    { id: "features", label: "Features" },
    { id: "sketch", label: "Sketch" },
    { id: "view", label: "View" },
  ];

  return (
    <div className="bg-[var(--cad-bg-ribbon)] border-b border-[var(--cad-border)]">
      {/* Tab bar */}
      <div className="flex">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-1.5 text-xs font-medium transition-colors ${
              activeTab === tab.id
                ? "bg-[var(--cad-bg-ribbon-tab)] text-[var(--cad-text-primary)] border-b-2 border-[var(--cad-accent)]"
                : "text-[var(--cad-text-secondary)] hover:text-[var(--cad-text-primary)] hover:bg-white/5"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Ribbon content */}
      <div className="flex items-stretch px-1 min-h-[62px]">
        {activeTab === "features" && (
          <>
            <RibbonGroup label="Extrude">
              <RibbonButton icon={Box} label="Extrude" shortcut="E" onClick={() => startOperation("extrude")} />
              <RibbonButton icon={BoxSelect} label="Cut" disabled onClick={() => {}} />
            </RibbonGroup>
            <RibbonGroup label="Revolve">
              <RibbonButton icon={RotateCw} label="Revolve" onClick={() => startOperation("revolve")} />
            </RibbonGroup>
            <RibbonGroup label="Modify">
              <div className="flex flex-col gap-0.5">
                <RibbonButton icon={Circle} label="Fillet" size="small" disabled onClick={() => {}} />
                <RibbonButton icon={Octagon} label="Chamfer" size="small" disabled onClick={() => {}} />
              </div>
            </RibbonGroup>
            <RibbonGroup label="Pattern">
              <div className="flex flex-col gap-0.5">
                <RibbonButton icon={Grid3x3} label="Linear" size="small" disabled onClick={() => {}} />
                <RibbonButton icon={RefreshCw} label="Circular" size="small" disabled onClick={() => {}} />
                <RibbonButton icon={FlipHorizontal2} label="Mirror" size="small" disabled onClick={() => {}} />
              </div>
            </RibbonGroup>
          </>
        )}
        {activeTab === "sketch" && (
          <>
            <RibbonGroup label="Draw">
              <RibbonButton icon={Minus} label="Line" shortcut="L" disabled onClick={() => {}} />
              <RibbonButton icon={Circle} label="Circle" disabled onClick={() => {}} />
              <RibbonButton icon={Square} label="Rect" disabled onClick={() => {}} />
            </RibbonGroup>
            <RibbonGroup label="Constrain">
              <div className="flex flex-col gap-0.5">
                <RibbonButton icon={Lock} label="Fix" size="small" disabled onClick={() => {}} />
                <RibbonButton icon={MoveHorizontal} label="Horizontal" size="small" disabled onClick={() => {}} />
                <RibbonButton icon={MoveVertical} label="Vertical" size="small" disabled onClick={() => {}} />
              </div>
              <div className="flex flex-col gap-0.5">
                <RibbonButton icon={Ruler} label="Dimension" size="small" disabled onClick={() => {}} />
              </div>
            </RibbonGroup>
          </>
        )}
        {activeTab === "view" && (
          <>
            <RibbonGroup label="Display">
              <div className="flex flex-col gap-0.5">
                <RibbonButton icon={Eye} label="Shaded" size="small" active={!wireframe} onClick={toggleWireframe} />
                <RibbonButton icon={Layers} label="Wireframe" size="small" shortcut="W" active={wireframe} onClick={toggleWireframe} />
                <RibbonButton icon={Network} label="Edges" size="small" active={showEdges} onClick={toggleEdges} />
              </div>
            </RibbonGroup>
            <RibbonGroup label="Orientation">
              <div className="flex flex-col gap-0.5">
                <RibbonButton icon={Square} label="Front" size="small" onClick={() => {}} />
                <RibbonButton icon={Box} label="Isometric" size="small" onClick={() => {}} />
                <RibbonButton icon={Maximize2} label="Fit All" size="small" onClick={() => {}} />
              </div>
            </RibbonGroup>
          </>
        )}
      </div>
    </div>
  );
}
