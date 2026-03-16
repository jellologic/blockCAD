# SolidWorks 3DEXPERIENCE Container Format

> Reverse-engineered from 38 SLDPRT part files and 1 SLDASM assembly file
> from a 3-wheel bicycle model (SolidWorks 2015 / AppVersion 23.0 / Parasolid 28.0).
> Files were in `.snapshot.3` directory format from the 3DEXPERIENCE platform.

## Overview

SolidWorks 3DEXPERIENCE files (`.SLDPRT`, `.SLDASM`) do **NOT** use the legacy
Microsoft OLE2/Structured Storage (Compound Binary File) format. Instead they
use a custom binary container with the following properties:

- Stream names are **nibble-swapped** ASCII
- Stream data is **raw deflate** compressed (zlib with wbits=-15, no header)
- Records are delimited by a fixed 6-byte **marker**
- The file has a variable-length **header** (14-22 bytes)
- Multiple records can share the same stream name (versioned snapshots)
- An **index/TOC** section appears near the end of the file

This is distinct from the older OLE2-based format which can be identified by
the `D0 CF 11 E0` magic bytes. The 3DEXPERIENCE format has no standard magic;
instead the version field at bytes 4-7 is always `00 00 00 04` (big-endian 4).

## File Header

The header has **variable length** (observed range: 14-22 bytes). Its end is
determined by finding the first record marker.

```
Offset  Size  Field
──────  ────  ─────────────────────────────────────────
0       4     Checksum/hash (unique per file, algorithm unknown)
4       4     Version = 0x00000004 (big-endian, always 4)
8       N     Type identifier (variable length, shared between similar files)
```

### Header Size Observations

| File         | Header Size | Type ID bytes |
|--------------|-------------|---------------|
| 1.SLDPRT     | 17          | 9 bytes       |
| 10.SLDPRT    | 13          | 5 bytes       |
| 11.SLDPRT    | 22          | 14 bytes      |
| 26.SLDPRT    | 14          | 6 bytes       |
| Assem1.SLDASM| 17          | 9 bytes       |

Files that share the same Type ID bytes often have the same internal structure
(e.g., same template or copy of the same part).

## Record Structure

Every record begins with the 6-byte marker, followed by a 20-byte fixed header:

```
Offset  Size  Field
──────  ────  ──────────────────────────────────────────────────────────
0       6     Marker: always 14 00 06 00 08 00
6       4     Tag: starts with 3B 78, last 2 bytes vary by version
10      4     Field1 (uint32 LE): checksum/hash for large records
14      4     Compressed size (uint32 LE): declared compressed payload size
18      4     Uncompressed size (uint32 LE): declared uncompressed size
22      4     Name length (uint32 LE): byte count of nibble-swapped name
26      N     Stream name (nibble-swapped ASCII, N = name_length)
26+N    M     Compressed payload (raw deflate, M = compressed_size approx)
```

Total fixed header: **26 bytes** (marker + fields + name_length).

### Tag Field

The 4-byte tag always starts with `3B 78`. Known tag variants:

```
3B 78 CF 3B    3B 78 4F 3A    3B 78 6F 3E    3B 78 6F 3B
3B 78 CF 3A    3B 78 EF 3B    3B 78 EF 3A    3B 78 AE 3B
3B 78 8E 3B    3B 78 8E 3F    3B 78 CE 3B    3B 78 2E 3B
3B 78 2E 3F    3B 78 0E 3F    3B F8 E4 FD
```

Different tags for the same stream name indicate different version snapshots.
The file keeps multiple versions; readers should use the version with the
largest decompressed size.

### Nibble Swap Encoding

Stream names use a byte-level transformation where the high and low 4-bit
nibbles of each byte are swapped:

```
Original byte:  0xAB
Swapped byte:   0xBA
```

Example:
```
Raw bytes: 34 f6 e6 47 56 e6 47 37
Swapped:   43 6f 6e 74 65 6e 74 73 = "Contents"
```

The transformation is its own inverse: applying it twice returns the original.

### Field1 (Checksum) — SOLVED

Field1 uses **standard CRC32** (ISO 3309, same as `zlib.crc32`) of the
**decompressed** payload data. Verified across 1313 large records with zero
mismatches.

Three cases:

| Record Type | Condition | Field1 Value |
|-------------|-----------|--------------|
| **Empty** | `uncompressed_size == 0` | `0` |
| **Small** | Size sentinel pattern | `2 × uncompressed_size` |
| **Large** | All other records | `CRC32(decompressed_data) & 0xFFFFFFFF` |

For small records, Field1 is not a checksum but a size-based sentinel. These
records also follow the pattern `compressed_size == uncompressed_size / 2`.

```python
import zlib
field1 = zlib.crc32(decompressed_data) & 0xFFFFFFFF
```

### File Header Checksum — Still Unknown

The 4-byte value at file offset 0-3 has NOT been identified. It is:
- Not CRC32 of any obvious byte range
- Not Adler32, FNV-1a, DJB2, or simple XOR-fold
- Possibly a platform-assigned identifier rather than a content hash
- Two different files (11.SLDPRT and 33.SLDPRT) share the same checksum
  despite having different content, suggesting it may not be content-derived

For write-back, keeping the original header checksum bytes works for
template-based modifications.

## Compression

All stream payloads use **raw deflate** compression (RFC 1951) without any
header or trailer. This corresponds to `zlib.decompress(data, wbits=-15)` or
`zlib.compressobj(level, DEFLATED, wbits=-15)` in Python.

This is different from:
- Standard zlib (has `78 xx` header)
- Gzip (has `1F 8B` header)

When recompressing, any deflate compression level produces valid output. The
original files appear to use level 6 or similar. The recompressed size will
differ from the original, so the `compressed_size` field in the record header
must be updated accordingly.

## Table of Contents

Records near the end of the file (typically 20-byte records ending with the
bytes `93 49 7E EC 00 00`) appear to serve as a directory/index. Their exact
structure and whether SolidWorks validates them is unknown.

## Round-Trip Capability

The container can be perfectly reconstructed:

```python
header = file_data[:first_marker_position]
records = file_data[first_marker_position:]
reconstructed = header + records  # byte-identical to original
```

Streams can be decompressed, modified, recompressed, and repacked. The
recompressed file will differ in size (due to different compression settings)
but contains identical decompressed content.
