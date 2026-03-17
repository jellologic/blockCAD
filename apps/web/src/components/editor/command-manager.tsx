import { useState, useEffect } from "react";
import {
  Box, RotateCw, Circle, Octagon, Grid3x3, RefreshCw, FlipHorizontal2,
  Minus, Square, Lock, MoveHorizontal, MoveVertical, Ruler, Spline,
  Eye, Layers, Network, Maximize2, BoxSelect, Pencil, Check, X, RulerIcon,
  Download, Plus, Link, FileText, Combine,
  Scissors, ArrowUpRight, Copy, FlipHorizontal,
  Hexagon, Disc, Group, Ungroup,
  CircleDot, Move, Scaling, Umbrella, AlignVerticalSpaceBetween, Scale,
} from "lucide-react";
import { StepExportDialog } from "@/components/export/step-export-dialog";
import { MassPropertiesPanel } from "@/components/analysis/mass-properties-panel";
import { RibbonButton } from "./ribbon-button";
import { useEditorStore } from "@/stores/editor-store";
import { useAssemblyStore } from "@/stores/assembly-store";
import { usePreferencesStore } from "@/stores/preferences-store";

type TabId = "features" | "sketch" | "view" | "assembly";

function RibbonGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center border-r border-[var(--cad-border)] px-2 last:border-r-0">
      <div className="flex items-center gap-0.5 py-1">{children}</div>
      <span className="text-[9px] text-[var(--cad-text-muted)] pb-0.5">{label}</span>
    </div>
  );
}

function InteractionStyleToggle() {
  const style = usePreferencesStore((s) => s.interactionStyle);
  const setStyle = usePreferencesStore((s) => s.setInteractionStyle);
  return (
    <div className="flex flex-col items-center border-r border-[var(--cad-border)] px-2 last:border-r-0">
      <div className="flex items-center gap-0.5 py-1">
        <button
          onClick={() => setStyle("fusion360")}
          className={`rounded px-2 py-1 text-[10px] transition-colors ${
            style === "fusion360"
              ? "bg-[var(--cad-accent)]/20 text-[var(--cad-accent)] font-medium"
              : "text-[var(--cad-text-muted)] hover:text-[var(--cad-text-primary)] hover:bg-white/5"
          }`}
        >
          Fusion 360
        </button>
        <button
          onClick={() => setStyle("solidworks")}
          className={`rounded px-2 py-1 text-[10px] transition-colors ${
            style === "solidworks"
              ? "bg-[var(--cad-accent)]/20 text-[var(--cad-accent)] font-medium"
              : "text-[var(--cad-text-muted)] hover:text-[var(--cad-text-primary)] hover:bg-white/5"
          }`}
        >
          SolidWorks
        </button>
      </div>
      <span className="text-[9px] text-[var(--cad-text-muted)] pb-0.5">Interaction</span>
    </div>
  );
}

export function CommandManager() {
  const [activeTab, setActiveTab] = useState<TabId>("features");
  const startOperation = useEditorStore((s) => s.startOperation);
  const startSketchFlow = useEditorStore((s) => s.startSketchFlow);
  const exitSketchMode = useEditorStore((s) => s.exitSketchMode);
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);
  const toggleWireframe = useEditorStore((s) => s.toggleWireframe);
  const toggleEdges = useEditorStore((s) => s.toggleEdges);
  const mode = useEditorStore((s) => s.mode);
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const setSketchTool = useEditorStore((s) => s.setSketchTool);
  const setCameraTarget = useEditorStore((s) => s.setCameraTarget);
  const fitAll = useEditorStore((s) => s.fitAll);
  const applyConstraint = useEditorStore((s) => s.applyConstraint);
  const exportSTL = useEditorStore((s) => s.exportSTL);
  const exportOBJ = useEditorStore((s) => s.exportOBJ);
  const export3MF = useEditorStore((s) => s.export3MF);
  const exportGLB = useEditorStore((s) => s.exportGLB);
  const hasMesh = useEditorStore((s) => s.meshData !== null);
  const showMassProperties = useEditorStore((s) => s.showMassProperties);
  const setShowMassProperties = useEditorStore((s) => s.setShowMassProperties);

  const [showStepDialog, setShowStepDialog] = useState(false);

  // Assembly store
  const isAssemblyMode = useAssemblyStore((s) => s.isAssemblyMode);
  const initAssembly = useAssemblyStore((s) => s.initAssembly);
  const exitAssemblyMode = useAssemblyStore((s) => s.exitAssemblyMode);
  const startOp = useAssemblyStore((s) => s.startOp);
  const toggleExploded = useAssemblyStore((s) => s.toggleExploded);
  const showBom = useAssemblyStore((s) => s.showBom);
  const exportAssemblyGLB = useAssemblyStore((s) => s.exportGLB);
  const assemblyComponents = useAssemblyStore((s) => s.components);

  // Auto-switch ribbon tab based on mode
  useEffect(() => {
    if (mode === "sketch") {
      setActiveTab("sketch");
    } else if (activeTab === "sketch") {
      setActiveTab("features");
    }
  }, [mode]);

  // Auto-switch to/from assembly tab when assembly mode changes
  useEffect(() => {
    if (isAssemblyMode) {
      setActiveTab("assembly");
    }
  }, [isAssemblyMode]);

  const tabs: { id: TabId; label: string }[] = [
    { id: "features", label: "Features" },
    { id: "sketch", label: "Sketch" },
    { id: "assembly", label: "Assembly" },
    { id: "view", label: "View" },
  ];

  return (
    <>
      <div className="bg-[var(--cad-bg-ribbon)] border-b border-[var(--cad-border)]">
        {/* Tab bar */}
        <div className="flex">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              data-testid={`tab-${tab.id}`}
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
              <RibbonGroup label="Sketch">
                <RibbonButton
                  icon={Pencil}
                  label="Sketch"
                  shortcut="S"
                  testId="ribbon-sketch"
                  onClick={startSketchFlow}
                />
              </RibbonGroup>
              <RibbonGroup label="Extrude">
                <RibbonButton icon={Box} label="Extrude" shortcut="E" testId="ribbon-extrude" onClick={() => startOperation("extrude")} />
                <RibbonButton icon={BoxSelect} label="Cut" shortcut="X" testId="ribbon-cut" onClick={() => startOperation("cut_extrude")} />
              </RibbonGroup>
              <RibbonGroup label="Revolve">
                <RibbonButton icon={RotateCw} label="Revolve" shortcut="V" onClick={() => startOperation("revolve")} />
                <RibbonButton icon={RotateCw} label="Cut" shortcut="" testId="ribbon-cut-revolve" onClick={() => startOperation("cut_revolve")} />
              </RibbonGroup>
              <RibbonGroup label="Sweep/Loft">
                <RibbonButton icon={Spline} label="Sweep" testId="ribbon-sweep" onClick={() => startOperation("sweep")} />
                <RibbonButton icon={Layers} label="Loft" testId="ribbon-loft" onClick={() => startOperation("loft")} />
              </RibbonGroup>
              <RibbonGroup label="Modify">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Circle} label="Fillet" size="small" shortcut="G" onClick={() => startOperation("fillet")} />
                  <RibbonButton icon={Octagon} label="Chamfer" size="small" shortcut="H" onClick={() => startOperation("chamfer")} />
                  <RibbonButton icon={Box} label="Shell" size="small" onClick={() => startOperation("shell")} />
                </div>
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Circle} label="Var Fillet" size="small" testId="ribbon-variable-fillet" onClick={() => startOperation("variable_fillet")} />
                  <RibbonButton icon={Circle} label="Face Fillet" size="small" testId="ribbon-face-fillet" onClick={() => startOperation("face_fillet")} />
                  <RibbonButton icon={CircleDot} label="Hole Wizard" size="small" testId="ribbon-hole-wizard" onClick={() => startOperation("hole_wizard")} />
                </div>
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Umbrella} label="Dome" size="small" onClick={() => startOperation("dome")} />
                  <RibbonButton icon={AlignVerticalSpaceBetween} label="Rib" size="small" onClick={() => startOperation("rib")} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Pattern">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Grid3x3} label="Linear" size="small" onClick={() => startOperation("linear_pattern")} />
                  <RibbonButton icon={RefreshCw} label="Circular" size="small" onClick={() => startOperation("circular_pattern")} />
                  <RibbonButton icon={FlipHorizontal2} label="Mirror" size="small" onClick={() => startOperation("mirror")} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Transform">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Move} label="Move/Copy" size="small" testId="ribbon-move-copy" onClick={() => startOperation("move_copy")} />
                  <RibbonButton icon={Scaling} label="Scale" size="small" testId="ribbon-scale" onClick={() => startOperation("scale")} />
                </div>
              </RibbonGroup>
            </>
          )}
          {activeTab === "sketch" && (
            <>
              <RibbonGroup label="Draw">
                <RibbonButton
                  icon={Minus}
                  label="Line"
                  shortcut="L"
                  testId="tool-line"
                  disabled={mode !== "sketch"}
                  active={sketchSession?.activeTool === "line"}
                  onClick={() => setSketchTool("line")}
                />
                <RibbonButton
                  icon={Circle}
                  label="Circle"
                  testId="tool-circle"
                  disabled={mode !== "sketch"}
                  active={sketchSession?.activeTool === "circle"}
                  onClick={() => setSketchTool("circle")}
                />
                <RibbonButton
                  icon={Spline}
                  label="Arc"
                  shortcut="A"
                  testId="tool-arc"
                  disabled={mode !== "sketch"}
                  active={sketchSession?.activeTool === "arc"}
                  onClick={() => setSketchTool("arc")}
                />
                <RibbonButton
                  icon={Square}
                  label="Rect"
                  testId="tool-rectangle"
                  disabled={mode !== "sketch"}
                  active={sketchSession?.activeTool === "rectangle"}
                  onClick={() => setSketchTool("rectangle")}
                />
              </RibbonGroup>
              <RibbonGroup label="Shapes">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Disc} label="Ellipse" size="small" testId="tool-ellipse" disabled={mode !== "sketch"} active={sketchSession?.activeTool === "ellipse"} onClick={() => setSketchTool("ellipse")} />
                  <RibbonButton icon={Hexagon} label="Polygon" size="small" testId="tool-polygon" disabled={mode !== "sketch"} active={sketchSession?.activeTool === "polygon"} onClick={() => setSketchTool("polygon")} />
                  <RibbonButton icon={Minus} label="Slot" size="small" testId="tool-slot" disabled={mode !== "sketch"} active={sketchSession?.activeTool === "slot"} onClick={() => setSketchTool("slot")} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Constrain">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Lock} label="Fix" size="small" disabled={mode !== "sketch"} onClick={() => applyConstraint("fixed")} />
                  <RibbonButton icon={MoveHorizontal} label="Horizontal" size="small" disabled={mode !== "sketch"} onClick={() => applyConstraint("horizontal")} />
                  <RibbonButton icon={MoveVertical} label="Vertical" size="small" disabled={mode !== "sketch"} onClick={() => applyConstraint("vertical")} />
                </div>
                <div className="flex flex-col gap-0.5">
                  <RibbonButton
                    icon={Ruler}
                    label="Dimension"
                    shortcut="D"
                    size="small"
                    disabled={mode !== "sketch"}
                    active={sketchSession?.activeTool === "dimension"}
                    testId="tool-dimension"
                    onClick={() => setSketchTool("dimension")}
                  />
                  <RibbonButton
                    icon={RulerIcon}
                    label="Measure"
                    shortcut="M"
                    size="small"
                    disabled={mode !== "sketch"}
                    active={sketchSession?.activeTool === "measure"}
                    testId="tool-measure"
                    onClick={() => setSketchTool("measure")}
                  />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Modify">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Scissors} label="Trim" shortcut="T" size="small" disabled={mode !== "sketch"} testId="tool-trim" active={sketchSession?.activeTool === "trim"} onClick={() => setSketchTool("trim")} />
                  <RibbonButton icon={ArrowUpRight} label="Extend" shortcut="E" size="small" disabled={mode !== "sketch"} testId="tool-extend" active={sketchSession?.activeTool === "extend"} onClick={() => setSketchTool("extend")} />
                  <RibbonButton icon={Copy} label="Offset" shortcut="O" size="small" disabled={mode !== "sketch"} testId="tool-offset" active={sketchSession?.activeTool === "offset"} onClick={() => setSketchTool("offset")} />
                </div>
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={FlipHorizontal} label="Mirror" size="small" disabled={mode !== "sketch"} testId="tool-mirror" active={sketchSession?.activeTool === "mirror"} onClick={() => setSketchTool("mirror")} />
                  <RibbonButton icon={Circle} label="Fillet" shortcut="F" size="small" disabled={mode !== "sketch"} testId="tool-sketch-fillet" active={sketchSession?.activeTool === "sketch-fillet"} onClick={() => setSketchTool("sketch-fillet")} />
                  <RibbonButton icon={Octagon} label="Chamfer" shortcut="H" size="small" disabled={mode !== "sketch"} testId="tool-sketch-chamfer" active={sketchSession?.activeTool === "sketch-chamfer"} onClick={() => setSketchTool("sketch-chamfer")} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Block">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Group} label="Create" size="small" disabled={mode !== "sketch"} testId="tool-block-create" active={sketchSession?.activeTool === "block"} onClick={() => setSketchTool("block")} />
                  <RibbonButton icon={Ungroup} label="Explode" size="small" disabled={mode !== "sketch"} testId="tool-block-explode" onClick={() => {/* TODO: explode selected block */}} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Pattern">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Grid3x3} label="Linear" size="small" testId="tool-sketch-linear-pattern" disabled={mode !== "sketch"} active={sketchSession?.activeTool === "sketch-linear-pattern"} onClick={() => setSketchTool("sketch-linear-pattern")} />
                  <RibbonButton icon={RefreshCw} label="Circular" size="small" testId="tool-sketch-circular-pattern" disabled={mode !== "sketch"} active={sketchSession?.activeTool === "sketch-circular-pattern"} onClick={() => setSketchTool("sketch-circular-pattern")} />
                  <RibbonButton icon={BoxSelect} label="Convert" size="small" testId="tool-convert-entities" disabled={mode !== "sketch"} active={sketchSession?.activeTool === "convert-entities"} onClick={() => setSketchTool("convert-entities")} />
                </div>
              </RibbonGroup>
              {/* Confirm / Cancel — always visible in sketch mode */}
              {mode === "sketch" && (
                <RibbonGroup label="Sketch">
                  <button
                    onClick={() => exitSketchMode(true)}
                    data-testid="ribbon-confirm-sketch"
                    className="flex flex-col items-center gap-0.5 rounded px-3 py-1 hover:bg-[#22cc44]/20 transition-colors"
                    title="Confirm Sketch (Enter)"
                  >
                    <Check size={20} color="#22cc44" />
                    <span className="text-[9px] text-[#22cc44] font-medium">Confirm</span>
                  </button>
                  <button
                    onClick={() => exitSketchMode(false)}
                    data-testid="ribbon-cancel-sketch"
                    className="flex flex-col items-center gap-0.5 rounded px-3 py-1 hover:bg-[#cc3333]/20 transition-colors"
                    title="Cancel Sketch (Escape)"
                  >
                    <X size={20} color="#cc3333" />
                    <span className="text-[9px] text-[#cc3333] font-medium">Cancel</span>
                  </button>
                </RibbonGroup>
              )}
            </>
          )}
          {activeTab === "assembly" && (
            <>
              <RibbonGroup label="Mode">
                {!isAssemblyMode ? (
                  <RibbonButton icon={Combine} label="Start" testId="assembly-start" onClick={initAssembly} />
                ) : (
                  <RibbonButton icon={X} label="Exit" testId="assembly-exit" onClick={exitAssemblyMode} />
                )}
              </RibbonGroup>
              {isAssemblyMode && (
                <>
                  <RibbonGroup label="Component">
                    <RibbonButton
                      icon={Plus}
                      label="Insert"
                      testId="assembly-insert"
                      onClick={() => startOp({ type: "insert-component", partId: "", name: "Component", x: 0, y: 0, z: 0 })}
                    />
                  </RibbonGroup>
                  <RibbonGroup label="Mate">
                    <div className="flex flex-col gap-0.5">
                      <RibbonButton
                        icon={Link}
                        label="Add Mate"
                        size="small"
                        testId="assembly-mate"
                        disabled={assemblyComponents.length < 2}
                        onClick={() => startOp({ type: "add-mate", kind: "coincident", compA: "", compB: "", faceA: 0, faceB: 0 })}
                      />
                    </div>
                  </RibbonGroup>
                  <RibbonGroup label="Assembly">
                    <div className="flex flex-col gap-0.5">
                      <RibbonButton icon={Maximize2} label="Explode" size="small" testId="assembly-explode" onClick={toggleExploded} />
                      <RibbonButton icon={FileText} label="BOM" size="small" testId="assembly-bom" onClick={showBom} />
                      <RibbonButton icon={Download} label="Export" size="small" testId="assembly-export" onClick={exportAssemblyGLB} />
                    </div>
                  </RibbonGroup>
                </>
              )}
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
                  <RibbonButton icon={Square} label="Front" size="small" onClick={() => setCameraTarget([0, 0, 30])} />
                  <RibbonButton icon={Box} label="Isometric" size="small" onClick={() => setCameraTarget([20, 15, 20])} />
                  <RibbonButton icon={Maximize2} label="Fit All" size="small" onClick={fitAll} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Export">
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Download} label="STL" size="small" testId="export-stl" disabled={!hasMesh} onClick={() => exportSTL(true)} />
                  <RibbonButton icon={Download} label="OBJ" size="small" testId="export-obj" disabled={!hasMesh} onClick={exportOBJ} />
                  <RibbonButton icon={Download} label="3MF" size="small" testId="export-3mf" disabled={!hasMesh} onClick={export3MF} />
                </div>
                <div className="flex flex-col gap-0.5">
                  <RibbonButton icon={Download} label="GLB" size="small" testId="export-glb" disabled={!hasMesh} onClick={exportGLB} />
                  <RibbonButton icon={Download} label="STEP" size="small" testId="export-step" disabled={!hasMesh} onClick={() => setShowStepDialog(true)} />
                </div>
              </RibbonGroup>
              <RibbonGroup label="Analysis">
                <RibbonButton icon={Scale} label="Mass Props" size="small" testId="mass-properties" disabled={!hasMesh} onClick={() => setShowMassProperties(true)} />
              </RibbonGroup>
              <InteractionStyleToggle />
            </>
          )}
        </div>
      </div>
      {showStepDialog && <StepExportDialog onClose={() => setShowStepDialog(false)} />}
      {showMassProperties && <MassPropertiesPanel onClose={() => setShowMassProperties(false)} />}
    </>
  );
}
