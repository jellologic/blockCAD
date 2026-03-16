# Parasolid XT Geometry Format

The B-rep (boundary representation) solid geometry in SolidWorks files is
stored using the **Parasolid XT (Transmit) binary format**, a well-documented
standard by Siemens PLM Software.

## Location

Geometry lives in the `Contents/Config-N-Partition` stream, where N is the
configuration index (typically 0).

## Double Compression

The partition data is **double-compressed**:

1. **Outer layer**: Raw deflate (handled automatically by the container parser)
2. **Inner layer**: Standard zlib with header (`78 xx`)

After the outer decompression, the stream contains:

```
[chunk_header : 16-20 bytes]
[zlib-compressed Parasolid data]
[optional: more chunks with their own headers + zlib data]
```

### Chunk Header

```
Offset  Size  Field
──────  ────  ────────────────────
0       4     Chunk size (uint32 LE, approximate)
4       4     Same bytes as file header bytes 4-11 (identity marker)
8+      4     Compressed size info
...           More metadata
```

Each chunk's zlib data starts with the standard `78 01` (or `78 5E`, `78 9C`,
`78 DA`) header.

## Parasolid XT Binary Format

After decompressing the inner zlib layer, the data is standard Parasolid XT:

```
PS   ?: TRANSMIT FILE (partition) created by modeller version 2800174
SCH_2800174_28002_13006
```

### Header Fields

| Field | Meaning |
|-------|---------|
| `PS`  | Parasolid magic bytes |
| `partition` or `deltas` | Chunk type |
| `2800174` | Parasolid kernel version (28.0.0174) |
| `SCH_2800174_28002_13006` | Schema version |

### Multiple Chunks

A single partition stream typically contains two chunks:

1. **Partition**: Main B-rep topology and geometry
2. **Deltas**: Incremental modifications to the topology

## Entity Hierarchy

Parasolid models solid geometry as a hierarchical tree:

```
BODY
└── REGION
    └── SHELL
        ├── FACE (with surface geometry)
        │   └── LOOP
        │       └── FIN
        │           └── EDGE (with curve geometry)
        │               └── VERTEX (with point geometry)
        ├── FACE ...
        └── FACE ...
```

### For a Simple Cylinder (8mm diameter × 145mm height)

```
BODY
└── REGION
    └── SHELL
        ├── FACE → cylindrical surface (r=0.004m, axis=Z)
        │   ├── LOOP (top) → EDGE → circle (z=0.145m, r=0.004m)
        │   └── LOOP (bot) → EDGE → circle (z=0, r=0.004m)
        ├── FACE → planar surface (z=0.145m, normal=+Z)
        │   └── LOOP → EDGE → circle (shared with cylindrical face)
        └── FACE → planar surface (z=0, normal=-Z)
            └── LOOP → EDGE → circle (shared with cylindrical face)
```

## Coordinate System

- All coordinates in **meters** (SI units)
- Float64 values are **BIG-ENDIAN** (unlike the rest of the file which is LE)
- Unit vectors use standard IEEE 754 representations:
  - `3F F0 00 00 00 00 00 00` = 1.0
  - `BF F0 00 00 00 00 00 00` = -1.0
  - `80 00 00 00 00 00 00 00` = -0.0

### Verified Dimension Values

For the test cylinder (26.SLDPRT):

| Hex (big-endian)       | Float64 | Meaning      |
|------------------------|---------|--------------|
| `3F 70 62 4D D2 F1 A9 FC` | 0.004   | Radius 4mm   |
| `3F C2 8F 5C 28 F5 C2 8F` | 0.145   | Height 145mm |

## Attributes

The Parasolid data includes entity attributes:

| Attribute Name | Purpose |
|---------------|---------|
| `BODY_IN_LIGHTWEIGHT_PERM` | Lightweight mode flag |
| `SDL/TYSA_COLOUR` | Face/body color (3× float64 RGB, 0-1 range) |
| `BODY_RECIPE_2001` | Body creation recipe |
| `BODY_MATCH` | Body matching identifier |
| `SWEntUnchanged` | Modification tracking |
| `LAST_BODY_MODIFYING_FEATURE_ID` | Last modifying feature |
| `ATOM_ID_2001` | Unique atom identifier |
| `ENT_TIME_STAMP_2001` | Entity timestamp |
| `FACE_ID_2001` | Face identifier |

### Default Color

The default SolidWorks part color:

```
R = 0.7922 (3F E9 59 59 59 59 59 59)
G = 0.8196 (3F EA 3A 3A 3A 3A 3A 3A)
B = 0.9333 (3F ED DD DD DD DD DD DE)

RGB(202, 209, 238) — light steel blue/grey
```

## Modification

Dimension values can be replaced by finding their exact byte patterns:

```python
import struct

# Little-endian (in Config-0, ResolvedFeatures, etc.)
old_le = struct.pack('<d', 0.004)  # 4mm radius
new_le = struct.pack('<d', 0.005)  # 5mm radius

# Big-endian (in Parasolid XT data)
old_be = struct.pack('>d', 0.004)
new_be = struct.pack('>d', 0.005)
```

When modifying dimensions, ALL streams containing the value must be updated:
- `Contents/Config-0` (bounding box, main config)
- `Contents/Config-0-ResolvedFeatures` (feature parameters)
- `Contents/Config-0-LWDATA` (lightweight data)
- `Contents/DisplayLists` (tessellated mesh)
- `Header2` (document header)
- `Contents/Config-0-Partition` (Parasolid XT, big-endian!)

Also update derived values:
- Negative values (bounding box min coordinates)
- Half values (bounding box center)
