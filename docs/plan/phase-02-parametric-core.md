# Phase 2: Parametric Core

**Status:** Complete

## Goal
Implement the two critical missing pieces: sketch solver bridge and feature tree evaluator, enabling real parametric modeling.

## Delivered
- **Solver bridge** (`solver_bridge.rs`): Maps sketch entities to solver variables, constraints to equations (6 types: Fixed, Coincident, Horizontal, Vertical, Distance, Perpendicular)
- **Feature tree evaluator** (`evaluator.rs`): Replays Sketch → Extrude/Revolve operations, producing BRep output
- **Profile extraction** (`profile.rs`): Extracts solved 2D positions into 3D ExtrudeProfile
- **Revolve operation**: Full/partial revolution with Rodrigues rotation, cap faces
- **Vitest setup**: Test infrastructure for kernel-js and web app
- **Integration tests**: `parametric_modeling_test.rs` with 8 end-to-end tests (including serialization roundtrip and revolve pipeline)
- 189 Rust tests + 14 frontend tests passing (at Phase 2 completion; grew to 209+25 through Phases 3-4)

## Key Insight
The evaluator stores Sketch objects in a side-channel HashMap on FeatureTree (not in FeatureParams) because EntityStore didn't serialize at this point. This was fixed in Phase 3.
