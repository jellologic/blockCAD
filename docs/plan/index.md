# blockCAD Feature Roadmap

Full feature parity with SolidWorks, organized into implementable phases. Each phase builds on the previous and maps directly to SolidWorks capabilities documented in the *Introducing SOLIDWORKS* manual.

## Current Status

| Phase | Status | Description |
|-------|--------|-------------|
| [Phase 1](./phase-01-vertical-slice.md) | Done | Kernel vertical slice: sketch, extrude, tessellate |
| [Phase 2](./phase-02-parametric-core.md) | Done | Solver bridge, feature tree evaluator, revolve, test infra |
| [Phase 3](./phase-03-foundation.md) | Done | Constraint solver completion, serialization, state mgmt, face selection |
| [Phase 4](./phase-04-solidworks-ui.md) | Done | SolidWorks-style UI: ribbon, feature tree, PropertyManager |
| [Phase 5](./phase-05-sketch-editor.md) | Next | Interactive 2D sketch editor |
| [Phase 6](./phase-06-core-features.md) | Planned | Cut-Extrude, Fillet, Chamfer, Shell |
| [Phase 7](./phase-07-advanced-features.md) | Planned | Sweep, Loft, Patterns, Mirror |
| [Phase 8](./phase-08-assemblies.md) | Planned | Multi-part assemblies with mates |
| [Phase 9](./phase-09-drawings.md) | Planned | 2D drawing generation from 3D models |
| [Phase 10](./phase-10-engineering.md) | Planned | Configurations, import/export, stress analysis |

## Architecture

```
Browser (React + Three.js)
  |
  +-- kernel-js (TypeScript API layer)
  |     |
  |     +-- MockKernelClient (current: JS-side geometry)
  |     +-- KernelClient (future: WASM bridge)
  |
  +-- kernel (Rust)
        |
        +-- sketch/ (2D constraint solver)
        +-- operations/ (extrude, revolve, fillet, etc.)
        +-- feature_tree/ (parametric history with evaluator)
        +-- topology/ (B-Rep: vertices, edges, faces, shells, solids)
        +-- geometry/ (curves, surfaces, NURBS)
        +-- tessellation/ (mesh generation for rendering)
        +-- serialization/ (.blockcad JSON format)
```

## SolidWorks Feature Mapping

Based on the SolidWorks Introduction manual, here's how their core capabilities map to our phases:

### Fundamentals (Ch. 2) → Phases 1-4
- [x] 3D design approach (sketch → feature → part)
- [x] FeatureManager design tree with rollback
- [x] PropertyManager for feature editing
- [x] CommandManager ribbon UI
- [x] Sketches with dimensions and relations
- [x] Feature creation (extrude, revolve)
- [x] Suppress/unsuppress features
- [x] Selection and feedback (face highlighting)

### Parts (Ch. 3) → Phases 5-7
- [ ] Interactive sketch editor (line, circle, arc, spline, rectangle)
- [ ] Sketch relations (coincident, horizontal, vertical, tangent, equal, parallel, perpendicular, midpoint, symmetric)
- [ ] Smart Dimension tool (driving and driven dimensions)
- [ ] Sketch state indicators (fully defined=black, under-defined=blue, over-defined=yellow)
- [ ] Extrude Boss/Base
- [ ] Cut-Extrude (remove material)
- [ ] Mid-Plane extrusion
- [ ] Revolve Boss/Base
- [ ] Sweep (profile along path)
- [ ] Loft (transition between profiles)
- [ ] Shell (hollow out solid)
- [ ] Fillet (round edges)
- [ ] Chamfer (bevel edges)
- [ ] Linear Pattern
- [ ] Circular Pattern
- [ ] Mirror Feature
- [ ] Configurations (part variations)
- [ ] Sheet Metal (base flange, tab, hem)

### Assemblies (Ch. 4) → Phase 8
- [ ] Assembly documents (.sldasm equivalent)
- [ ] Bottom-up and top-down design
- [ ] Mates (coincident, concentric, distance, tangent, perpendicular)
- [ ] SmartMates (automatic mate inference)
- [ ] In-context design (reference other components)
- [ ] Component instances (reuse same part)
- [ ] Hide/show components
- [ ] Exploded views
- [ ] Collision detection
- [ ] Move/rotate components

### Drawings (Ch. 5) → Phase 9
- [ ] Drawing documents with templates
- [ ] Standard 3 views (front, top, right)
- [ ] Isometric and named views
- [ ] Projected views, section views, detail views
- [ ] Dimension insertion from model
- [ ] Reference dimensions
- [ ] Annotations (notes, GD&T, datum symbols, weld symbols)
- [ ] Bill of materials
- [ ] Balloons and stacked balloons
- [ ] Explode lines in assembly drawings

### Engineering Tasks (Ch. 6) → Phase 10
- [ ] Multiple configurations
- [ ] Automatic model updating (parametric propagation)
- [ ] Import/export (STEP, IGES, STL, etc.)
- [ ] Feature recognition in imported parts
- [ ] Stress analysis (SimulationXpress)
- [ ] Assembly animation
- [ ] Standard parts library
- [ ] Geometry examination tools (measure, section)

## Design Principles

From the SolidWorks manual's design philosophy:

1. **Design Intent** — How the model reacts to changes. Dimensions and relations should capture the designer's intent so modifications propagate correctly.
2. **Fully Defined Sketches** — All sketch entities constrained by dimensions or relations. Visual feedback: black=fully defined, blue=under-defined, yellow=over-defined.
3. **Feature Order Matters** — Features build on each other. The feature tree represents the design history and determines rebuild order.
4. **Component-Based** — Changes to a part propagate to all assemblies and drawings that reference it.
5. **Sketch Simplicity** — Simple sketches rebuild faster. Use relations and symmetry to reduce complexity.
