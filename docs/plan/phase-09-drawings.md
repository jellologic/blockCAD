# Phase 9: 2D Drawings

**Status:** Planned

## Goal
Generate 2D engineering drawings from 3D models with dimensions, annotations, and views.

## SolidWorks Reference (Manual pp. 71-88)
- Drawing documents with templates and sheet formats (p.71-73)
- Standard 3 views: front, top, right (p.74-75)
- Isometric and named views (p.75-76)
- Projected, section, and detail views (p.76)
- View display modes: Hidden Lines Visible, Shaded With Edges, Hidden Lines Removed (p.76-77)
- Dimensions imported from model (p.77-78)
- Reference dimensions, ordinate dimensions (p.79)
- Hole callouts (p.79)
- Annotations: notes, GD&T, datum symbols, weld symbols, center marks (p.80)
- Bill of Materials for assemblies (p.87)
- Balloons and stacked balloons (p.88)
- Explode lines in assembly drawings (p.82)

## Implementation Plan

### Drawing Engine
- Orthographic projection: project 3D BRep edges onto 2D plane
- Hidden line removal algorithm
- Section view: cut model with plane, show cross-section
- Detail view: magnified circular region

### View Layout
- Sheet with border and title block template
- Drag to place views, automatic alignment (front↔top, front↔right)
- Scale per view

### Dimensioning
- Import driving dimensions from model
- Add reference dimensions
- Ordinate and baseline dimensioning
- Auto-jog to avoid overlap

### Annotations
- Leader notes, GD&T symbols, surface finish
- Bill of materials (auto-generated from assembly)
- Balloons pointing to BOM entries

### Export
- PDF output
- DXF/DWG export
- SVG for web viewing

## Kernel Requirements
- Edge projection onto arbitrary planes
- Hidden line computation (ray casting or geometric)
- Section plane intersection with BRep
