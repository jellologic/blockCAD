# Phase 1: Vertical Slice

**Status:** Complete

## Goal
Prove the full stack end-to-end: Rust kernel → Three.js viewport rendering an extruded box.

## Delivered
- Rust kernel with sketch entities, constraint types, Newton-Raphson solver
- Extrude operation creating BRep from planar profiles
- Ear-clip tessellation for planar faces
- Feature tree with rollback cursor, suppress/unsuppress
- JSON serialization (.blockcad format) with schema versioning
- Three.js viewport with OrbitControls, grid, gizmo
- Mock kernel returning hardcoded 10x5x7 box mesh
- Feature tree sidebar and display toggles
- 156 Rust tests passing

## Key Files
- `packages/kernel/` — Rust CAD kernel
- `packages/kernel-js/` — TypeScript wrapper + mock kernel
- `apps/web/src/routes/editor.tsx` — Editor page
- `apps/web/src/components/viewport/` — 3D rendering
