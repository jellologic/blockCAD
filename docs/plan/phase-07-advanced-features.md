# Phase 7: Advanced Part Features

**Status:** Planned

## Goal
Implement sweep, loft, patterns, and mirror — enabling complex organic shapes and repetitive geometry.

## SolidWorks Reference (Manual pp. 42-52)
- Faucet: Extrude + Sweep (spigot) + Fillets (p.42-43)
- Faucet handle: Revolve + Revolve + Fillets, using Line/Spline/Tangent Arc (p.43-45)
- Hinge: Sheet Metal Base Flange + Tab + Linear Pattern + Hem (p.49-52)

## Features

### Sweep
- Extrude a profile along a path curve (p.42-43)
- Profile and path must intersect at start
- Optional twist angle along path
- Kernel: requires NURBS curve evaluation for path, swept surface generation

### Loft
- Transition between two or more sketch profiles on different planes (p.40)
- Profiles can be different shapes (ellipse → circle)
- Optional guide curves for shape control
- Kernel: requires multi-profile blending, NURBS surface generation

### Linear Pattern
- Copy a feature in one or two directions (p.51)
- Parameters: direction, spacing, count
- Kernel: transform and merge BRep copies

### Circular Pattern
- Copy a feature around an axis
- Parameters: axis, count, total angle

### Mirror Feature
- Mirror a feature about a plane of symmetry (p.48)
- Kernel: reflect BRep geometry across plane

## Kernel Requirements
- **NURBS curve evaluation**: Cox-de Boor algorithm for sweep paths
- **NURBS surface evaluation**: For loft and sweep surfaces
- **Curved surface tessellation**: Adaptive subdivision based on curvature
- **BRep copy + transform**: For pattern operations

## Testing
- Faucet tutorial walkthrough: base extrudes + sweep spigot
- Hinge: linear pattern of tabs
