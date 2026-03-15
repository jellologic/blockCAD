# Phase 6: Core Part Features

**Status:** Planned

## Goal
Implement the features used in every SolidWorks part: Cut-Extrude, Fillet, Chamfer, Shell. After this phase, users can create real-world parts like the countertop, cabinet door, and moldings from the SolidWorks manual.

## SolidWorks Reference (Manual pp. 36-48)
- Countertop: Extrude + Extrude + Cut-Extrude + Loft + Shell + Fillet (p.37)
- Cabinet door: Extrude + Cut-Extrude + Chamfer (p.46)
- Moldings: Mid-Plane Extrude + Cut-Extrude + Mirror + Configurations (p.47-48)

## Features

### Cut-Extrude
- Same as Boss-Extrude but removes material instead of adding
- Requires: sketch on face or plane, direction, depth
- End conditions: Blind, Through All, Up To Surface
- Kernel: subtract the extruded profile from existing BRep (requires Boolean Subtract)

### Fillet
- Round edges with specified radius (p.41-42)
- Applied feature — no sketch required, select edges
- Constant radius fillet (most common)
- Kernel: offset faces adjacent to selected edges, blend surface between

### Chamfer
- Bevel edges at specified angle/distance (p.46-47)
- Applied feature — select edges
- Distance-distance or distance-angle modes
- Kernel: cut corner geometry, create planar chamfer face

### Shell
- Hollow out a solid, leaving walls of specified thickness (p.41)
- Select faces to remove (open faces)
- Kernel: offset all faces inward, remove selected faces

### Mid-Plane Extrude
- Extrude equally in both directions from sketch plane (p.47)
- Already partially supported via `symmetric` param

## Kernel Requirements
- **Boolean operations**: Union, Subtract, Intersect on BRep (critical for Cut-Extrude)
- **Edge selection**: Edge ID tracking through tessellation (for fillet/chamfer)
- **Face offset**: Move faces inward/outward by distance (for shell)
- **Curved surface tessellation**: Fillet creates cylindrical surfaces

## Testing
- Countertop tutorial walkthrough: create the part step by step
- Each feature: parameter validation, mesh validity, serialization roundtrip
