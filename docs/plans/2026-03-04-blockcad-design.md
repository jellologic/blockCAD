# blockCAD Design Document

> Parametric CAD application built with React + Three.js + OpenCascade.js
> Approved: 2026-03-04

---

## Overview

blockCAD is a browser-based parametric CAD application modeled after Onshape. Users create 3D parts by sketching 2D profiles on planes and applying 3D operations (extrude, revolve, fillet, etc.). A server-side OpenCascade.js engine handles all geometry computation, streaming tessellated triangle data to a React Three Fiber frontend.

### Key Decisions

- **Server-side geometry**: OpenCascade.js (WASM) runs in Bun via `opencascade.js/dist/node.js`
- **Client receives triangles only**: positions, normals, indices as Float32Arrays
- **Single-user first**: No real-time collaboration in v1
- **Full feature set**: Sketch, Extrude, Revolve, Sweep, Loft, Fillet, Chamfer, Shell, Boolean, Pattern, Mirror, Draft
- **Builds on existing monorepo**: TanStack Start, oRPC, Drizzle, Better-Auth all stay

---

## 1. System Architecture

```
┌─────────────────────────────────────────────────────┐
│                    BROWSER (React)                   │
│                                                      │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │  Feature  │  │  3D Viewport │  │   Toolbars    │ │
│  │   Tree    │  │  (R3F/Three) │  │  & Panels     │ │
│  │  Panel    │  │   WebGL2     │  │               │ │
│  └────┬─────┘  └──────┬───────┘  └───────┬───────┘ │
│       └───────────────┼───────────────────┘          │
│              ┌────────▼────────┐                     │
│              │  Zustand Store  │                     │
│              └────────┬────────┘                     │
│                       │ oRPC calls                   │
└───────────────────────┼──────────────────────────────┘
                        │
┌───────────────────────┼──────────────────────────────┐
│              SERVER (TanStack Start + Bun)            │
│              ┌────────▼────────┐                     │
│              │   oRPC Router   │                     │
│              └────────┬────────┘                     │
│              ┌────────▼────────┐                     │
│              │ OpenCascade.js  │  ← WASM kernel      │
│              └────────┬────────┘                     │
│     ┌─────────────────┼─────────────────┐           │
│  ┌──▼───┐     ┌──────▼──────┐  ┌──────▼──────┐    │
│  │ B-Rep│     │ Tessellator │  │ STEP/IGES   │    │
│  │Engine│     │ → triangles │  │ Import/Export│    │
│  └──────┘     └─────────────┘  └─────────────┘    │
│              ┌─────────────────┐                     │
│              │   PostgreSQL    │                     │
│              └─────────────────┘                     │
└──────────────────────────────────────────────────────┘
```

---

## 2. Data Model

### Database Tables (Drizzle schema)

**document**: id, name, ownerId, createdAt, updatedAt

**element**: id, documentId, name, type ("partstudio" | "assembly"), index

**feature**: id, elementId, index, type, name, parameters (jsonb), suppressed (boolean), groupId (nullable FK to feature_group)

**feature_group**: id, elementId, name, collapsed (boolean)

**configuration**: id, elementId, name, parameters (jsonb)

**part**: id, elementId, name, color, material, visible (boolean)

### Feature Parameter Examples

```jsonc
// Extrude
{
  "type": "extrude",
  "parameters": {
    "sketchId": "feat_abc123",
    "profiles": ["face_0", "face_1"],
    "direction": "normal",
    "depth": { "value": 25, "unit": "mm" },
    "operation": "new",          // new | add | remove | intersect
    "symmetric": false,
    "draft": { "angle": 0, "inward": false }
  }
}

// Fillet
{
  "type": "fillet",
  "parameters": {
    "edges": ["edge_12", "edge_15"],
    "radius": { "value": 3, "unit": "mm" },
    "tangentPropagation": true
  }
}

// Sketch
{
  "type": "sketch",
  "parameters": {
    "plane": { "type": "face", "ref": "face_top" },
    "entities": [
      { "id": "e1", "type": "line", "start": [0,0], "end": [50,0] },
      { "id": "e2", "type": "arc", "center": [25,30], "radius": 25,
        "startAngle": 0, "endAngle": 180 }
    ],
    "constraints": [
      { "type": "coincident", "entityA": "e1.end", "entityB": "e2.start" },
      { "type": "perpendicular", "entityA": "e1", "entityB": "e2" },
      { "type": "dimension", "entity": "e1", "value": { "value": 50, "unit": "mm" } }
    ]
  }
}
```

Feature evaluation is sequential: the server replays features 1..N through OpenCascade to rebuild the model.

---

## 3. Frontend Component Architecture

### Component Tree

```
<App>
  <TopNavbar>
    <Logo />
    <DocumentName editable />
    <BranchIndicator />
    <ShareButton />
    <UserMenu />
  </TopNavbar>

  <MainLayout>                         {/* allotment resizable panels */}
    <LeftPanelRail />                  {/* Thin icon strip */}
    <LeftPanel>
      <ConfigurationPanel />
      <FeatureTree>
        <FeatureFilter />
        <FeatureList />                {/* Draggable items, groups, rollback */}
      </FeatureTree>
      <PartsList />
    </LeftPanel>

    <ViewportArea>
      <Toolbar />                      {/* Contextual: Part Studio vs Sketch */}
      <Canvas3D>                       {/* React Three Fiber */}
        <CadCamera />
        <CadOrbitControls />           {/* MMB rotate, Shift+MMB pan */}
        <SceneLighting />
        <ModelMesh />                  {/* BufferGeometry from server */}
        <SelectionHighlight />
        <SketchOverlay />              {/* 2D sketch on plane */}
      </Canvas3D>
      <ViewCube />
      <ViewControls />
      <FeatureDialog />                {/* Parameter editing panel */}
    </ViewportArea>

    <RightPanelRail />
  </MainLayout>

  <TabBar />
</App>
```

### Zustand Store

```typescript
interface CadStore {
  document: Document
  activeElementId: string
  features: Feature[]
  selectedFeatureId: string | null
  rollbackIndex: number
  meshData: TessellatedPart[]
  selectedEntities: EntityRef[]
  viewMode: 'shaded' | 'wireframe' | 'hidden-line'
  activeTool: ToolType | null
  toolState: ToolDialogState

  addFeature: (type, params) => Promise<void>
  editFeature: (id, params) => Promise<void>
  deleteFeature: (id) => Promise<void>
  reorderFeature: (id, newIndex) => Promise<void>
  selectEntities: (refs) => void
  setActiveTool: (tool) => void
}
```

### Interaction Loop

1. User clicks tool (e.g., "Extrude") -> `activeTool = 'extrude'`
2. User selects faces in viewport -> `selectedEntities` updates
3. User fills parameters in FeatureDialog
4. User clicks checkmark -> `addFeature('extrude', params)` calls oRPC
5. Server evaluates feature tree through OpenCascade, returns tessellation
6. `meshData` updates -> Three.js re-renders

---

## 4. Server-Side CAD Engine

### oRPC Router

```
cadRouter:
  createDocument    → { documentId }
  getDocument       → { document, elements }
  addFeature        → { feature, tessellation }
  updateFeature     → { feature, tessellation }
  deleteFeature     → { tessellation }
  reorderFeature    → { tessellation }
  suppressFeature   → { tessellation }
  getTessellation   → { parts: TessellatedPart[] }
  getBodyDetails    → { faces, edges, vertices }
  getMassProperties → { volume, area, centerOfMass }
  createSketch      → { sketchId, plane }
  solveSketch       → { solvedGeometry, constraints }
  importSTEP        → { features, tessellation }
  exportSTEP        → { fileUrl }
  exportSTL         → { fileUrl }
```

### Tessellation Response

```typescript
interface TessellatedPart {
  partId: string
  name: string
  color: [r, g, b, a]
  mesh: {
    positions: Float32Array   // [x,y,z, x,y,z, ...]
    normals:   Float32Array   // [nx,ny,nz, ...]
    indices:   Uint32Array    // triangle index buffer
  }
  edges: {
    positions: Float32Array   // line segments for wireframe
  }
}
```

### CadEngine Class

```typescript
class CadEngine {
  private oc: OpenCascadeInstance
  private sessions: Map<string, CadSession>

  async rebuildModel(elementId, features): Promise<TessellatedPart[]>

  // Feature → OCCT mapping:
  extrude()  → BRepPrimAPI_MakePrism
  revolve()  → BRepPrimAPI_MakeRevol
  fillet()   → BRepFilletAPI_MakeFillet
  chamfer()  → BRepFilletAPI_MakeChamfer
  boolean()  → BRepAlgoAPI_Fuse / Cut / Common
  sweep()    → BRepOffsetAPI_MakePipe
  loft()     → BRepOffsetAPI_ThruSections
  shell()    → BRepOffsetAPI_MakeThickSolid
  pattern()  → gp_Trsf transforms + boolean fuse
  mirror()   → BRepBuilderAPI_Transform
  draft()    → BRepOffsetAPI_DraftAngle
}
```

Each active document gets a CadSession holding OCCT shapes in memory. When feature N is edited, engine replays from start (or cached checkpoint) through N, applies the change, continues to end.

---

## 5. Sketch System

Sketches are 2D drawings on a plane that become profiles for 3D operations.

**Rendering**: Drawn in Three.js on the selected plane. Camera locks to face the plane in sketch mode.

**Entities**: Line, Arc, Circle, Rectangle (4 lines), Spline, Construction geometry, Dimensions

**Constraints**: Coincident, Parallel, Perpendicular, Tangent, Equal, Horizontal, Vertical, Midpoint, Symmetric, Fix

**Constraint solving**: Server-side via OCCT's 2D solver. Client sends raw geometry + constraints, server returns solved positions.

**Visual feedback**: Green = fully constrained, Blue = under-constrained, Red = over-constrained

**Sketch toolbar** replaces main toolbar: Line, Arc, Circle, Rect, Spline, Trim, Offset, Mirror, Pattern, Dimension, Constrain, Accept/Cancel

---

## 6. Package Structure

### New files (builds on existing monorepo)

```
apps/web/
  routes/document.$id.tsx                    # CAD editor route
  components/cad/
    viewport/   Canvas3D, CadCamera, ModelMesh, ViewCube, SelectionManager, SketchOverlay
    toolbar/    MainToolbar, SketchToolbar, ToolButton, ToolGroup
    feature-tree/  FeatureTree, FeatureItem, FeatureGroup, PartsList, RollbackBar
    panels/     LeftPanel, RightPanel, PanelRail, ConfigurationPanel
    dialogs/    FeatureDialog, ExtrudeDialog, FilletDialog, ... per feature
    layout/     CadLayout, TabBar
  stores/cad-store.ts

packages/api/src/
  routers/cad.ts
  engine/
    engine.ts, session.ts, tessellator.ts, sketch-solver.ts
    operations/  extrude, revolve, fillet, chamfer, boolean, sweep, loft, shell, pattern, mirror, draft

packages/db/src/schema/
  cad.ts                                     # document, element, feature, part tables
```

### New Dependencies

```
apps/web:       three, @react-three/fiber, @react-three/drei, allotment, zustand
packages/api:   opencascade.js@beta
```

---

## 7. Implementation Phases

1. **Core viewport**: Three.js canvas, orbit controls, lighting, BufferGeometry rendering, view cube
2. **Layout shell**: Resizable split panels, left sidebar, top toolbar, bottom tab bar
3. **Engine foundation**: OpenCascade.js init, basic extrude from hardcoded sketch, tessellation pipeline
4. **Database + documents**: Schema, document CRUD, feature persistence
5. **Feature tree UI**: Feature list, add/delete/reorder, rollback bar
6. **Sketch system**: Basic line/arc/circle drawing, constraint solving, sketch mode
7. **Full feature set**: Revolve, sweep, loft, fillet, chamfer, shell, boolean, pattern, mirror, draft
8. **Import/export**: STEP and STL
9. **Polish**: Selection highlighting, appearances, measure tool, keyboard shortcuts
