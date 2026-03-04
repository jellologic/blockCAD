# Onshape CAD Interface Research

> Research conducted via Playwright browser automation on 2026-03-03
> Model explored: MKCad Pulleys (configurable VexPro Hex Bore Pulley) + Bench Vise (owned, editable)

---

## 1. Architecture Overview

### Framework & Rendering Stack

| Layer | Technology |
|-------|-----------|
| **UI Framework** | AngularJS 1.8.11 (legacy) + custom Web Components |
| **3D Rendering** | Custom WebGL2 renderer (NOT Three.js) |
| **WASM Module** | `GraphicsWebAssemblyUtils.wasm` for GPU buffer processing |
| **Bundler** | Webpack (runtime, vendor-bundle, serialize, webgl chunks) |
| **Geometry Kernel** | Server-side Parasolid (Siemens PLM) |
| **Constraint Solver** | Server-side D-Cubed |
| **Communication** | HTTPS/REST + WebSocket (proprietary wire protocol) |

### Key Scripts Loaded
```
runtime.6f19dde503b7bb75ed92.js        # Webpack runtime
vendor-bundle.df05cb6593d86ee41a7a.js   # Third-party libs
webpack-vendor.95e8f057d9376b1c29c9.js  # More vendor code
serialize.a504118dcd4d8e71021c.js       # Data serialization
webgl.939f99c208f98755047f.js           # WebGL renderer
woolsthorpe.8175a31b57288c896eae.js     # Main app (AngularJS)
woolsthorpe.f03cae891c8817da9958.js     # App continued
GraphicsWebAssemblyUtils.js             # WASM loader
GraphicsWebAssemblyUtils.wasm           # GPU buffer processing
cacheworker-worker.*.chunk.js           # Cache web worker
```

### Client-Server Model
- **Server reconstructs geometry** from feature lists using Parasolid kernel
- **Server computes tessellations** (triangles) and sends them to client
- **Client receives triangulated mesh data**, NOT full B-rep geometry
- **Progressive tessellation**: coarse mesh first, then finer detail after view manipulation stops
- Data is stored **incrementally in a database**, not as files

---

## 2. UI Layout Architecture

### Overall DOM Structure
```
body.os-hover-enabled
  div.os-app-container
    osx-root                          # Root web component
      navigation (top navbar)
        logo + document name + branch + stats
        share/help/user buttons
      div.element-split-container
        div.os-panel-select-bar       # Left icon rail (5 buttons)
        div.os-column-splitter-container
          div.left-panel              # Feature tree + config
          div.os-splitter             # Draggable divider
          div.os-split-content        # Main canvas area
            div.canvas-container
              canvas#canvas           # WebGL2 canvas (954x723)
              os-lui-toolbar          # Bottom floating toolbar
              div.element-banner      # "View only" banner
              div.element-measurement # Measurement overlay
              div.os-graphics-dropdown
                os-view-cube-bounds   # 3D orientation cube (150x150px)
                div.cube-options      # View cube menu
        div.right-panel               # Right sidebar panels
      div.os-tab-bar-content          # Bottom tab bar
        tab-list                      # Document tabs
```

### Layout Strategy
- **Splitter-based layout**: `os-column-splitter-container` and `os-row-splitter-container` with draggable dividers
- **Panels are togglable**: Left panel can collapse/expand
- **Canvas fills remaining space**: After left panel and right panel are accounted for
- **Floating toolbars**: Bottom toolbar and view cube overlay on canvas
- **Tab bar at bottom**: Horizontal scrollable tab list for document elements

---

## 3. Left Sidebar Panel

### Panel Structure
```
Configurations Panel
  - Dropdown selector (os-select-single component)
  - Shows current config: "18t 15mm Wide 1/2" Hex Bore..."

Feature Tree
  - Filter input: "Filter by name or type"
  - Header: "Features (34)" or "Features (47)"
  - Controls: rollback slider, rebuild, settings buttons
  - Tree items:
    - Default geometry group (Origin, Top, Front, Right planes)
    - Variables: #teeth = 18, #width = 18.5mm, etc.
    - Sketches
    - Features (Extrude, Revolve, Fillet, etc.)
    - Feature groups (collapsible folders)

Parts List
  - Header: "Parts (5)"
  - Individual part items with icons
  - Surfaces section
```

### Configuration System
When a config is changed:
1. Dropdown selection triggers API call
2. Server recomputes geometry with new parameters
3. Feature tree variables update (#teeth, #flange_inner_dia, etc.)
4. New tessellation data streams to client
5. WebGL canvas re-renders with new mesh

Config dropdown options are a flat list with descriptive names:
```
"18t 15mm Wide 1/2" Hex Bore Double Flange 5mm HTD Aluminum Pulley (217-4100)"
"24t 15mm Wide 1/2" Hex Bore Double Flange 5mm HTD Aluminum Pulley (217-4101)"
...
```

---

## 4. Top Toolbar (Edit Mode)

### Part Studio Tools
| Tool | Shortcut | SVG Icon |
|------|----------|----------|
| Undo | Cmd+Z | `#svg-icon-undo-button` |
| Redo | Ctrl+Y | `#svg-icon-redo-button` |
| **Sketch** | Shift+S | `#svg-icon-new-sketch-button` |
| **Extrude** | Shift+E | `#svg-icon-extrude-button` |
| **Revolve** | Shift+W | `#svg-icon-revolve-button` |
| Sweep | - | `#svg-icon-sweep-button` |
| Loft | - | `#svg-icon-loft-button` |
| Thicken | - | `#svg-icon-thicken-button` |
| **Fillet** | Shift+F | `#svg-icon-fillet-button` |
| Linear Pattern | - | `#svg-icon-linear-pattern-button` |
| Boolean | - | `#svg-icon-boolean-bodies-button` |
| Modify Fillet | - | `#svg-icon-modify-fillet-button` |
| Plane | - | `#svg-icon-c-plane-button` |
| Frame | - | `#svg-icon-frame-button` |
| Sheet Metal | - | `#svg-icon-sheet-metal-start-button` |
| Custom Features | - | `#svg-icon-add-feature-type` |

### Toolbar Layout
- Toolbar uses `os-element-toolbar` class
- Tools are grouped with visual separators
- Some tools have dropdown arrows for sub-tools
- "Search tools..." button at end (Alt+C)
- Groups are separated by thin vertical dividers

---

## 5. Bottom Toolbar (View-Only Mode)

| Tool | SVG Icon |
|------|----------|
| Default View (home) | `#svg-icon-lui-home` |
| Inspection Table | `#svg-icon-lui-model-based-definition` |
| Properties | `#svg-icon-lui-properties` |
| Export Tab | `#svg-icon-lui-export` |
| Print Image | `#svg-icon-lui-print` |
| Measure | `#svg-icon-lui-measure` |

---

## 6. Right Side Panels

### Panel Icon Rail (Left Edge)
| Panel | SVG Icon |
|-------|----------|
| Versions and History | `#svg-icon-versions-history-panel` |
| Create Version | `#svg-icon-create-version-button` |
| Comments | `#svg-icon-comments-panel` |
| Document Notes | `#svg-icon-notes-panel` |
| Where Used | `#svg-icon-where-used-upgrade` |
| Performance | `#svg-icon-performance-panel` |
| AI Advisor | `#svg-icon-onshape-ai-advisor-button` |
| Explore Onshape | `#svg-icon-explore-onshape-circle` |
| Tab Manager | `#svg-icon-tab-manager` |

### Right-Side View Controls (on canvas)
| Control | SVG Icon |
|---------|----------|
| Features and Parts toggle | `#svg-icon-treelist-button` |
| Show Measure Details | (shortcut: `[`) |
| Show Analysis Tools | - |
| Mass/Section Properties | - |
| Camera and Render Options | - |
| Appearance Panel | `#svg-icon-appearances-button` |
| Configuration Panel | `#svg-icon-configuration-table-button` |
| Custom Tables | `#svg-icon-featurescript-table-button` |
| Hole Table | `#svg-icon-hole-table` |
| Inspection Table | `#svg-icon-model-based-definition` |
| Variable Table | `#svg-icon-variable-table-button` |

---

## 7. 3D Viewport & View Cube

### Canvas
- Single `<canvas id="canvas">` element
- WebGL2 rendering context
- Dimensions: fills available space (observed 954x723)
- Parent: `div.canvas-container.dialog-drag-container`

### View Cube
- Located in `div.os-view-cube-bounds` (150x150px, absolutely positioned)
- Rendered as part of the WebGL canvas (empty DOM = rendered in canvas)
- Shows axes: X (red), Y (green), Z (blue)
- Labels: Front, Top, Right
- Clickable faces/edges for standard views
- Interactive rotation by dragging

### WebGL2 Details
```javascript
{
  renderer: "WebGL2",
  maxTextureSize: 16384,
  vendor: "WebKit",
  extensions: [
    "EXT_color_buffer_float",
    "EXT_texture_filter_anisotropic",
    "KHR_parallel_shader_compile",
    "OES_texture_float_linear",
    // ... 20+ extensions
  ]
}
```

---

## 8. Tab Bar (Bottom)

### Structure
- Horizontal scrollable tab bar
- Each tab: icon + name text
- Active tab highlighted
- Tab types:
  - **Part Studio** (sketch icon) - individual part modeling
  - **Assembly** (assembly icon) - multi-part assemblies
  - **Drawing** (drawing icon)
  - **Tab Groups** (folder-like grouping with `os-tab-bar-tab-group` class)
- Left/right scroll buttons for overflow
- "+" button for adding new tabs

### Example Tabs (Pulley Document - 22 tabs)
```
5mm HTD Timing Pulleys (group)
Versapulleys (group)
Configurable VexPro Hex Bore Pulley (active)
GT2 Timing Pulleys (group)
Configurable 3mm GT2 Timing Pulley
Configurable HTD Timing Pulley
Polycord Pulleys (group)
CAD Imports (group)
Assembly 1
... etc
```

---

## 9. API Patterns

### URL Structure
```
/api/v13/{resource}/{documentId}/w/{workspaceId}/e/{elementId}
```
- **DWVME pattern**: Document / Workspace|Version|Microversion / Element
- IDs are 24-character hex strings

### Key API Endpoints Observed
```
GET  /api/v13/documents/{docId}                    # Document metadata
GET  /api/v13/documents/{docId}/permissionset      # Permissions
GET  /api/v13/users/session                        # Current user session
GET  /api/v13/users/settings                       # User preferences
GET  /api/v13/toolbar/tools                        # Available tools (19KB!)
GET  /api/v13/toolbar/toolbars                     # Toolbar layout (21KB!)
GET  /api/v13/keyboardshortcuts/users/{userId}     # Keyboard shortcuts
GET  /api/v13/capabilities/allcurrent/             # Feature flags (15KB)
GET  /api/v13/documents/d/{docId}/w/{wsId}/elements # Document elements
GET  /api/v13/elements/translatorFormats/{d}/w/{w}/{e} # Export formats
POST /api/v13/documents/d/{docId}/workspaces/{wsId}/createIfNecessary # Session init
GET  /api/v13/documents/{docId}:{wsId}/modelingServiceRequest # Model session
GET  /api/v13/elementLibrary/standardlibrarydefinitions # Standard features
GET  /api/v13/notifications/summary                # Polling for notifications
```

### Key REST API Endpoints for 3D Data (from documentation)
```
GET /api/v6/partstudios/d/{did}/w/{wid}/e/{eid}/tessellatedfaces  # Face mesh data
GET /api/v6/partstudios/d/{did}/w/{wid}/e/{eid}/tessellatededges  # Edge wireframes
GET /api/v6/partstudios/d/{did}/w/{wid}/e/{eid}/features          # Feature tree
GET /api/v6/partstudios/d/{did}/w/{wid}/e/{eid}/bodydetails       # Body info
```

### Communication Channels
1. **REST/HTTPS**: Standard API calls for data fetching
2. **WebSocket**: Real-time updates, proprietary wire protocol
3. **Cache Worker**: `cacheworker-worker.*.chunk.js` for client-side caching

---

## 10. Custom Web Components

Onshape defines many custom elements:
```html
<osx-root>               <!-- App root -->
<os-woolsthorpe-navbar>   <!-- Top navigation -->
<os-lui-toolbar>          <!-- Bottom toolbar -->
<os-vue-lui-toolbar>      <!-- Vue-based toolbar variant -->
<os-select-single>        <!-- Custom dropdown -->
<os-enum-parameter>       <!-- Parameter editor -->
<os-parameter-list-view>  <!-- Parameter list -->
<os-flyout>               <!-- Popup panels -->
<os-notification-flyout>  <!-- Notifications -->
<selection-filter>        <!-- Entity selection filter -->
<tab-list>                <!-- Tab bar -->
<tab-list-item>           <!-- Individual tab -->
<element-name>            <!-- Editable element name -->
<document-spinners>       <!-- Loading indicators -->
<network-monitor>         <!-- Connection status -->
```

---

## 11. React Replication Strategy

### Recommended Tech Stack

| Onshape Component | React Equivalent |
|-------------------|-----------------|
| AngularJS 1.8 | React 18+ with hooks |
| Custom WebGL renderer | **Three.js** (React Three Fiber) |
| WASM GPU buffers | Three.js `BufferGeometry` |
| Parasolid kernel | **OpenCascade.js** (WASM) or server-side BREP engine |
| WebSocket protocol | Standard WebSocket + custom protocol |
| AngularJS services | React Context + Zustand/Jotai |
| ng-controller directives | React functional components |
| Splitter layout | `react-resizable-panels` or `allotment` |
| SVG icon system | SVG sprite sheet or `lucide-react` |
| Custom elements | React components |

### Component Hierarchy (React)

```
<App>
  <TopNavbar>
    <Logo />
    <DocumentInfo name="..." branch="Main" />
    <DocumentStats views={0} likes={0} copies={0} />
    <ShareButton />
    <UserMenu />
  </TopNavbar>

  <MainLayout>  {/* Resizable split panels */}
    <LeftPanelRail>
      <PanelButton icon="history" tooltip="Versions" />
      <PanelButton icon="notes" tooltip="Notes" />
      <PanelButton icon="performance" tooltip="Performance" />
      <PanelButton icon="tabs" tooltip="Tab Manager" />
    </LeftPanelRail>

    <LeftPanel>
      <ConfigurationPanel>
        <ConfigDropdown options={configurations} />
      </ConfigurationPanel>
      <FeatureTree>
        <FeatureFilter />
        <FeatureTreeHeader count={47} />
        <FeatureGroup name="Default geometry">
          <FeatureItem type="origin" name="Origin" />
          <FeatureItem type="plane" name="Top" />
          <FeatureItem type="plane" name="Front" />
          <FeatureItem type="plane" name="Right" />
        </FeatureGroup>
        <FeatureItem type="variable" name="#teeth = 18" />
        <FeatureItem type="sketch" name="Sketch 1" />
        <FeatureItem type="extrude" name="5mm HTD pulley" />
        {/* ... */}
      </FeatureTree>
      <PartsList>
        <PartItem name="BASE" />
        <PartItem name="LOCKING COLLAR" />
      </PartsList>
    </LeftPanel>

    <ResizableDivider />

    <ViewportArea>
      <Toolbar>  {/* Top toolbar - edit mode */}
        <UndoRedo />
        <ToolGroup>
          <ToolButton icon="sketch" label="Sketch" shortcut="Shift+S" />
          <ToolButton icon="extrude" label="Extrude" shortcut="Shift+E" />
          <ToolButton icon="revolve" label="Revolve" shortcut="Shift+W" />
          <ToolButton icon="sweep" label="Sweep" />
          <ToolButton icon="loft" label="Loft" />
        </ToolGroup>
        <ToolGroup>
          <ToolButton icon="fillet" label="Fillet" shortcut="Shift+F" />
          <ToolButton icon="pattern" label="Linear Pattern" />
          <ToolButton icon="boolean" label="Boolean" />
        </ToolGroup>
        <SearchTools />
      </Toolbar>

      <Canvas3D>  {/* React Three Fiber */}
        <PerspectiveCamera />
        <OrbitControls />
        <ambientLight />
        <directionalLight />
        <ModelMesh geometry={tessellatedData} />
        <GridHelper />
      </Canvas3D>

      <ViewCube position="top-right" />  {/* Three.js overlay */}

      <ViewControls position="right">
        <ControlButton tooltip="Camera options" />
        <ControlButton tooltip="Appearances" />
        <ControlButton tooltip="Measure" />
      </ViewControls>

      <BottomToolbar>  {/* Floating bottom bar */}
        <ToolButton icon="home" label="Default view" />
        <ToolButton icon="export" label="Export" />
        <ToolButton icon="measure" label="Measure" />
      </BottomToolbar>
    </ViewportArea>

    <RightPanelRail>
      <PanelButton icon="history" tooltip="Versions" />
      <PanelButton icon="comments" tooltip="Comments" />
      <PanelButton icon="notes" tooltip="Notes" />
    </RightPanelRail>
  </MainLayout>

  <TabBar>
    <TabScrollButton direction="left" />
    <TabList>
      <Tab name="Part Studio 1" type="partstudio" active />
      <Tab name="Assembly 1" type="assembly" />
      <TabGroup name="Imported Parts" />
    </TabList>
    <TabScrollButton direction="right" />
    <AddTabButton />
  </TabBar>
</App>
```

### Key Implementation Priorities

#### Phase 1: Core Viewport
1. **Three.js Canvas** with React Three Fiber (`@react-three/fiber`)
2. **OrbitControls** for camera manipulation (`@react-three/drei`)
3. **BufferGeometry** renderer for tessellated mesh data
4. **View Cube** overlay (use `@react-three/drei` ViewCube or custom)
5. **Grid and axis helpers**

#### Phase 2: Layout Shell
1. **Resizable split panels** (`allotment` or `react-resizable-panels`)
2. **Left sidebar** with feature tree (tree view component)
3. **Top toolbar** with icon buttons and dropdowns
4. **Bottom tab bar** with horizontal scroll
5. **Right panel rail** with togglable panels

#### Phase 3: Feature Tree & Parametric Modeling
1. **Feature tree** with collapsible groups, drag reorder
2. **Configuration system** with dropdown selector
3. **Parameter editing** in feature dialogs
4. **Sketch editing mode** (2D canvas overlay)

#### Phase 4: Geometry Engine
1. **OpenCascade.js** for client-side B-rep operations (or server-side engine)
2. **Tessellation pipeline**: B-rep -> triangles -> BufferGeometry
3. **Feature evaluation**: sequential feature application
4. **Progressive LOD**: coarse mesh first, refine on idle

### Key Libraries

```json
{
  "@react-three/fiber": "^8.x",
  "@react-three/drei": "^9.x",
  "three": "^0.160+",
  "allotment": "^1.x",
  "zustand": "^4.x",
  "opencascade.js": "^2.x",
  "lucide-react": "^0.x"
}
```

---

## 12. Screenshots Reference

| Screenshot | Description |
|-----------|-------------|
| `onshape-signin.png` | Sign-in page |
| `onshape-dashboard.png` | Document dashboard (owned by me) |
| `onshape-cad-model.png` | 3D pulley model (view-only mode) |
| `onshape-config-dropdown.png` | Configuration dropdown expanded |
| `onshape-config-changed.png` | Model after config change (36t pulley) |
| `onshape-edit-mode.png` | Full edit mode with toolbar (Bench Vise) |

---

## Sources

- [Onshape REST API Introduction](https://onshape-public.github.io/docs/api-intro/)
- [How Does Onshape Really Work?](https://www.onshape.com/en/blog/how-does-onshape-really-work)
- [Onshape Forum: HTML/JS 3D Viewer](https://forum.onshape.com/discussion/13716/html-javascript-3d-web-viewer-for-onshape)
- [Onshape Forum: Three.js Shaders](https://forum.onshape.com/discussion/22002/shaders-for-three-js-model-viewer)
- [Onshape WebGL Requirements](https://cad.onshape.com/help/Content/webgl.htm)
