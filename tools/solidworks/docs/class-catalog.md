# Serialized Class Catalog

317 unique C++ classes identified across 38 SLDPRT part files and 1 SLDASM
assembly, organized by functional category.

## Sketch Entities (35 classes)

Classes that define 2D sketch geometry and constraints.

| Class | Files | Purpose |
|-------|-------|---------|
| `sgArcHandle` | 37 | Arc or full circle sketch entity |
| `sgLineHandle` | 36 | Line segment sketch entity |
| `sgPointHandle` | 39 | Point sketch entity |
| `sgSplineHandle` | 3 | Spline curve sketch entity |
| `sgCircleDim` | 34 | Circle dimension constraint |
| `sgBlock` | 1 | Sketch block entity |
| `sgSketch` | 39 | Root sketch container |
| `sgSketchBlockHandle` | 2 | Reference to sketch block |
| `sgSlotHandle` | 1 | Slot sketch entity |
| `sgSlot_c` | 1 | Slot definition |
| `sgPGMProfileData_c` | 4 | Profile geometry data |
| `sgPoint3DPlaneDist` | 2 | 3D point-to-plane distance |
| `sgExtEnt_c` | 37 | External entity reference |
| `moProfileFeature_c` | 38 | 2D sketch profile feature |
| `mo3DProfileFeature_c` | 7 | 3D sketch profile feature |
| `moSketchChain_c` | 37 | Chain of connected sketch entities |
| `moSketchExtRef_w` | 37 | External sketch reference |
| `moSketchBlockMgr_c` | 39 | Sketch block manager |
| `moSketchBlockDef_c` | 1 | Sketch block definition |
| `moSketchBlockInst_c` | 1 | Sketch block instance |
| `moSketchBlockExtLinkData_c` | 1 | Block external link |
| `moCompSketchEntHandle_c` | 38 | Component sketch entity handle |
| `moCompProfile_c` | 11 | Component profile reference |
| `moOriginProfileFeature_c` | 39 | Origin point feature |
| `moProfileFtrFolder_c` | 3 | Profile feature folder |
| `moProfileRef_w` | 5 | Profile reference wrapper |
| `moChainComponent_c` | 1 | Chain component |
| `moContextChainComponent_c` | 2 | Context chain component |
| `moReferenceChain_c` | 2 | Reference chain |
| `moFromSktEntSurfIdRep_c` | 33 | Sketch-to-surface ID mapping |
| `moFromSktEnt3IntSurfIdRep_c` | 33 | 3-intersection surface ID |
| `moConstraintCoincLineAtAnglePlaneRefplaneData_c` | 1 | Coincident+angle constraint |
| `moConstraintCoincLinePerpPlaneRefplaneData_c` | 1 | Coincident+perpendicular constraint |
| `moConstraintPerpPlnTanOneCylinderRefplaneData_c` | 1 | Perpendicular+tangent constraint |
| `moConstraintPrllPlnTanOneCylinderRefplaneData_c` | 1 | Parallel+tangent constraint |

## Feature Operations (48 classes)

Classes that define modeling operations (extrude, revolve, fillet, etc.)

| Class | Files | Purpose |
|-------|-------|---------|
| `moExtrusion_c` | 22 | Boss or cut extrusion |
| `moRevolution_c` | 10 | Revolve feature |
| `moSweep_c` | 3 | Sweep feature |
| `moSweepCut_c` | 2 | Cut sweep |
| `moSweepThread_c` | 1 | Thread sweep |
| `Fillet_c` | 15 | Fillet (round) feature |
| `Chamfer_c` | 8 | Chamfer feature |
| `moMirrorPattern_c` | 6 | Mirror pattern |
| `moMirrorSolid_c` | 2 | Mirror solid body |
| `moMirrorStock_c` | 1 | Mirror stock |
| `moCirPattern_c` | 2 | Circular pattern |
| `moCombineBodies_c` | 4 | Combine bodies |
| `moMoveCopyBody_c` | 4 | Move/copy body |
| `moEndCap_c` | 1 | End cap feature |
| `moCut_c` | 1 | Cut feature |
| `moRevCut_c` | 5 | Revolve cut |
| `moLoftSynchRef_c` | 1 | Loft synchronization |
| `moWeldmentFeature_c` | 5 | Weldment structural member |
| `moWeldSegment_c` | 5 | Weld segment |
| `moWeldCornerFeat_c` | 3 | Weld corner |
| `moWeldMemberFeat_c` | 5 | Weld member |
| `moWeldBreakPoint_c` | 3 | Weld break point |
| `moWeldTrimToolData_c` | 3 | Weld trim tool |
| `moWeldmentContours_c` | 5 | Weldment contours |
| `moEndSpec_c` | 35 | Extrusion end condition |
| `moFromEndSpec_c` | 35 | From-end condition |
| `moRevEndSpec_c` | 12 | Revolve end condition |
| `moPerBodyChooserData_c` | 37 | Per-body chooser data |
| `moCompSolidBody_c` | 34 | Component solid body |
| `moSolidBodyFolder_c` | 38 | Solid body folder |
| `moSurfaceBodyFolder_c` | 38 | Surface body folder |
| `moCutListFolder_c` | 5 | Cut list folder |
| `moFilletSurfIdRep_c` | 22 | Fillet surface ID |
| `moMirPatternSurfIdRep_c` | 7 | Mirror pattern surface ID |
| `moCirPatternSurfIdRep_c` | 2 | Circular pattern surface ID |
| `moSweepSideSurfIdRep_c` | 8 | Sweep side surface ID |
| `moEndCapChamferSurfIdRep_c` | 1 | End cap chamfer surface ID |
| `moSegmLeftTrimEdgeIdRep_c` | 8 | Left trim edge ID |
| `moSegmRightTrimEdgeIdRep_c` | 8 | Right trim edge ID |
| `moDisplayRevolveDim_c` | 3 | Revolve dimension display |
| `uoBodyPropInfo_c` | 33 | Body property info |
| `uoTempBodyTessData_c` | 33 | Temporary body tessellation |
| (+ more) | | |

## Dimension & Parameter Classes (40 classes)

| Class | Files | Purpose |
|-------|-------|---------|
| `moLengthParameter_c` | 38 | Length dimension value (float64, meters) |
| `moAngleParameter_c` | 27 | Angle dimension value |
| `moIntegerParameter_c` | 9 | Integer parameter |
| `moScalerParameter_c` | 3 | Scalar parameter |
| `ParallelPlaneDistanceDim_c` | 34 | Plane-to-plane distance |
| `AngleDim_c` | 20 | Angle dimension |
| `ThreeDRadiusDim_c` | 12 | 3D radius dimension |
| `ThreeDdiameterDim_c` | 3 | 3D diameter dimension |
| `moDisplayDistanceDim_c` | 38 | Distance dimension display |
| `moDisplayRadialDim_c` | 34 | Radial dimension display |
| `moDisplayAngularDim_c` | 26 | Angular dimension display |
| `moDisplayDim_c` | 5 | Generic dimension display |
| `moFeatureDimHandle_c` | 37 | Feature dimension handle |
| `moSkDimHandleRadial_c` | 34 | Radial dim handle (\<MOD-DIAM\>) |
| `moSkDimHandleOffset_c` | 10 | Offset dimension handle |
| `moSkDimHandleValG2_c` | 34 | G2 value dimension handle |
| `sgPntPntDist` | 34 | Point-to-point distance constraint |
| `sgPntLineDist` | 22 | Point-to-line distance constraint |
| `sgLLDist` | 17 | Line-to-line distance constraint |
| `sgAnglDim` | 15 | Angle dimension constraint |
| `sgOffsetDim` | 9 | Offset dimension constraint |
| `sgCircularPattCntDim` | 9 | Circular pattern count |
| (+ 18 more) | | |

## Geometry Reference Classes (30 classes)

| Class | Files | Purpose |
|-------|-------|---------|
| `moFaceRef_c` | 39 | Face reference |
| `moEdgeRef_c` | 32 | Edge reference |
| `moVertexRef_c` | 8 | Vertex reference |
| `moCompFace_c` | 35 | Component face |
| `moCompEdge_c` | 28 | Component edge |
| `moEndFaceSurfIdRep_c` | 31 | End face surface ID |
| `moEndFace3IntSurfIdRep_c` | 29 | 3-intersection end face |
| `moSurfaceIdRep_c` | 18 | Surface ID representation |
| `moSimpleSurfIdRep_c` | 10 | Simple surface ID |
| `moBBoxCenterData_c` | 37 | Bounding box center data |
| `moFaceRefPlnData_c` | 32 | Face reference plane data |
| (+ 19 more) | | |

## Assembly Classes (15 classes)

| Class | Files | Purpose |
|-------|-------|---------|
| `moAssembly_c` | 1 | Root assembly object |
| `moMateGroup_c` | 1 | Mate group container |
| `moBCMate_c` | 1 | Boundary condition mate |
| `moMateAnalysisData_c` | 1 | Mate analysis data |
| `moBeltMateFolder_c` | 1 | Belt mate folder |
| `moCompFeature_c` | 39 | Component feature reference |
| `moCompHolder_c` | 38 | Component holder |
| `moCompRefPlane_c` | 39 | Component reference plane |
| `moCompRefAxis_c` | 3 | Component reference axis |
| `moMaterialFolder_c` | 38 | Material folder |
| `moMaterial_c` | 38 | Material definition |
| (+ 4 more) | | |

## Infrastructure Classes (70 classes)

Standard infrastructure present in virtually every file:

- Document management: `moHeader_c`, `moLogs_c`, `moStamp_c`, `moNodeName_c`
- Units: `moUnitsTable_c` + 12 specific unit classes
- Display: `moView_c`, `moAmbientLight_c`, `moDirectionLight_c`
- Visual: `moVisualProperties_c`, `visStateKeyData_c`
- Folders: `moCommentsFolder_c`, `moFavoriteFolder_c`, `moHistoryFolder_c`, etc.
- Configuration: `dmConfigMgrHeader_c`, `dmConfigHeader_c`

See the source code in `catalog/class_registry.py` for the complete list.
