# Binary Serialization Format

SolidWorks serializes its internal C++ object model into binary streams using
a self-describing format with class markers, string markers, and fixed-width
numeric fields.

## Class Instance Markers

Every serialized C++ class instance is introduced by a 4-byte marker followed
by a length-prefixed ASCII class name:

```
FF FF 01 00  [name_len : uint16 LE]  [class_name : ASCII bytes]
```

### Examples

```
ff ff 01 00 0d 00 6d 6f 45 78 74 72 75 73 69 6f 6e 5f 63
              │           └── "moExtrusion_c" (13 bytes)
              └── name_len = 13

ff ff 01 00 12 00 6d 6f 50 72 6f 66 69 6c 65 46 65 61 74 75 72 65 5f 63
              │           └── "moProfileFeature_c" (18 bytes)
              └── name_len = 18
```

### Class Naming Conventions

| Prefix   | Meaning                          | Example                    |
|----------|----------------------------------|----------------------------|
| `mo*_c`  | Model objects (most common)      | `moExtrusion_c`            |
| `mo*_w`  | Wrapper/reference objects        | `moSketchExtRef_w`         |
| `sg*`    | Sketch geometry entities         | `sgArcHandle`, `sgLineHandle` |
| `gc*_c`  | Graphics/geometry cache          | `gcCurvatureObject_c`      |
| `dm*_c`  | Document management              | `dmConfigHeader_c`         |
| `ui*_c`  | UI-related                       | `uiLineFontMgr_c`          |
| `vis*_c` | Visual state                     | `visStateKeyData_c`        |
| `uo*_c`  | User objects / tessellation      | `uoTempBodyTessData_c`     |
| `*_c`    | Feature classes (no prefix)      | `Fillet_c`, `Chamfer_c`    |

## String Encoding

Strings use a 3-byte marker followed by a 1-byte length and UTF-16LE data:

```
FF FE FF  [char_count : uint8]  [chars : char_count × 2 bytes, UTF-16LE]
```

### Examples

```
ff fe ff 07 53 00 6b 00 65 00 74 00 63 00 68 00 31 00
         │  └── "Sketch1" in UTF-16LE (7 chars × 2 bytes = 14 bytes)
         └── char_count = 7

ff fe ff 00
         └── empty string (0 chars)

ff fe ff 18 41 00 4e 00 53 00 49 00 33 00 31 00 ...
         │  └── "ANSI31..." (24 chars)
         └── char_count = 0x18 = 24
```

Note: `FF FE` is the UTF-16LE byte-order mark, but here it serves as a string
marker prefix. The third `FF` byte is always present.

Maximum string length with uint8 count: 255 characters. Strings longer than
this may use a different encoding (not yet observed in sample files).

## Numeric Values

All numeric values in the serialized object data use **little-endian** byte order.

| Type     | Size   | Python struct | Notes                            |
|----------|--------|---------------|----------------------------------|
| uint8    | 1 byte | `B`           | Flags, small counts              |
| uint16   | 2 bytes| `<H`          | Name lengths, small IDs          |
| uint32   | 4 bytes| `<I`          | Sizes, timestamps, feature IDs   |
| int32    | 4 bytes| `<i`          | Signed integers                  |
| float64  | 8 bytes| `<d`          | **Coordinates in meters (SI)**   |

### Coordinate System

All spatial values (lengths, positions, radii) are stored in **meters**:

```
0.004  = 4 mm   (e.g., circle radius)
0.008  = 8 mm   (e.g., circle diameter)
0.145  = 145 mm (e.g., extrusion depth)
```

### Timestamps

Unix timestamps as uint32 LE:

```
b8 c9 12 5b → 0x5B12C9B8 = 1527957944 → 2018-06-02T21:52:24Z
```

## Object Hierarchy

Classes are nested inline — a parent object's binary data contains child
class markers. The hierarchy can be reconstructed by tracking marker offsets:

```
moPart_c (root)
├── moHeader_c
│   ├── su_CStringArray
│   └── suObList
├── moLogs_c
│   └── moStamp_c (creation/modification timestamps)
├── moNodeName_c (part name as UTF-16 string)
├── moVisualProperties_c
├── moUnitsTable_c
│   ├── moLengthUserUnits_c
│   ├── moAngleUserUnits_c
│   ├── moNumberUserUnits_c
│   └── ... (12+ unit type classes)
├── gcXhatch_c (cross-hatch pattern)
├── moMaterial_c
├── moEnvFolder_c
│   ├── moAmbientLight_c
│   └── moDirectionLight_c (×3)
├── moView_c (viewport orientation)
├── moSketchBlockMgr_c
└── moAnnotationView_c
```

## Bounding Box

The bounding box is stored in `Contents/Config-0` as a sequence of float64 LE
values in meters. For a cylinder with radius 4mm and height 145mm:

```
Offset  Value         Meaning
──────  ────────────  ─────────────────
+0      0.0725        half-height (center Z)
+8      0.004         +X extent (radius)
+16     0.004         +Y extent (radius)
+24     0.145         +Z extent (height)
+32     -0.004        -X extent
+40     -0.004        -Y extent
```

## Feature Tree (ResolvedFeatures stream)

The `Contents/Config-0-ResolvedFeatures` stream contains the full parametric
feature tree. Key classes for a simple extruded-circle part:

```
moCommentsFolder_c         "Comments"
moRefPlane_c               "Front Plane"
moDefaultRefPlnData_c      "Top Plane", "Right Plane"
moOriginProfileFeature_c   "Origin"
moProfileFeature_c         "Sketch1"          ← 2D sketch
  sgArcHandle              (circle entity)
  moSketchChain_c          (entity chain)
moLengthParameter_c        "D1" = 0.008m      ← diameter dimension
moSkDimHandleRadial_c      "<MOD-DIAM>"       ← diameter modifier
moExtrusion_c              "Boss-Extrude1"    ← extrusion feature
moEndSpec_c                (end condition)
ParallelPlaneDistanceDim_c "D1" = 0.145m      ← depth dimension
moFromEndSpec_c            (from-end condition)
```

## Sketch Entity Sub-Records

Within the sgArcHandle data region, sketch entities are delimited by the
sub-record marker `FF FF 1F 00 03`. Despite the class name, this region
contains ALL sketch entity types (points, lines, arcs, circles).

### Sub-record layout

```
Offset  Size  Field
──────  ────  ──────────────────────────────────────────
-4      4     entity_id (uint32 LE)
+0      5     marker: FF FF 1F 00 03
+5      8     sentinel: FF FF FF FF FF FF FF FF
+13     4     float32 = -1.0 (00 00 80 BF)
+17     4     type_field (uint32 LE): entity type discriminator
+21     37    flags, padding, constraint refs
+58     8     annotation_x (float64 LE): display label position
+66     8     annotation_y (float64 LE): display label position
```

### Entity types

| type_field | Entity | Notes |
|------------|--------|-------|
| 0 | Point | Simple point entity |
| 1 | Line/Circle | Line segment or circle (distinguished by context) |
| 2 | Arc | Arc entity (may have extended data) |
| 3 | Constrained point | Point with geometric constraints |
| 4 | Unknown | Observed in complex sketches |
| 5 | Unknown | Observed in complex sketches |

### Important: Where geometry data actually lives

The sub-records contain entity **topology** (IDs, types, connectivity) and
**annotation positions** (where to render dimension labels on screen). They
do NOT contain the actual geometric coordinates (center points, radii,
line endpoints).

Geometric coordinates are in:
- **Parasolid XT** partition: evaluated B-rep with exact positions
- **Dimension classes**: parametric values (D1=0.008m diameter)
- **`sgEntHandle`**: dimension values at relative offset +82
- **`moSkDimHandleRadial_c`**: radius/diameter at offsets +155, +227, +251

### Dimension value locations (confirmed by cross-file correlation)

| Class | Rel Offset | Content |
|-------|-----------|---------|
| `sgEntHandle` | +82 | Dimension value (float64 LE, meters) |
| `moSkDimHandleRadial_c` | +155 | Diameter value |
| `moSkDimHandleRadial_c` | +227 | Radius value |
| `moSkDimHandleRadial_c` | +251 | Radius value (duplicate) |
| `ParallelPlaneDistanceDim_c` | +34 | Distance value |
| `ParallelPlaneDistanceDim_c` | +63 | Related radius |
| `moFavoriteHandle_c` | +28 | Dimension value |
