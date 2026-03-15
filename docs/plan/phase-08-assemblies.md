# Phase 8: Assemblies

**Status:** Planned

## Goal
Enable multi-part assemblies with mates, following SolidWorks' assembly paradigm.

## SolidWorks Reference (Manual pp. 54-70)
- Assembly definition: collection of parts with mates (p.54)
- Bottom-up design: create parts separately, assemble (p.55)
- Top-down design: create parts in assembly context (p.55)
- Mates: Coincident, Concentric, Distance, Tangent, Perpendicular (p.58-62)
- SmartMates: automatic mate inference from geometry (p.62)
- Component instances: reuse same part multiple times (p.59)
- In-context design: reference other parts' geometry (p.65-66)
- Hide/Show components (p.68)
- Exploded views (p.68)
- Collision detection (p.69-70)

## Implementation Plan

### Data Model
- Assembly document type (.blockcad-asm)
- Component references (path to part document)
- Transform per component instance (position + rotation)
- Mates stored as constraints between component geometry

### Mate System
- Coincident: align two planar faces
- Concentric: align two cylindrical faces
- Distance: maintain fixed distance between entities
- Tangent: face-to-face tangency
- Perpendicular: faces at 90 degrees

### UI
- Assembly tab in CommandManager
- Insert Component dialog (file picker)
- Mate PropertyManager: select two entities, choose mate type
- Component move/rotate in 3D
- Assembly FeatureManager tree showing components and mates

### Kernel Requirements
- Multi-body support in BRep
- Assembly constraint solver (different from sketch solver — operates on rigid body transforms)
- Lightweight component loading (load only visible/needed geometry)

## Testing
- Faucet assembly: faucet + handles with coincident + concentric mates
- Door assembly: door + moldings with coincident mates
