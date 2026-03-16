# swparse — SolidWorks 3DEXPERIENCE File Toolkit

Reverse-engineered parser, inspector, and modifier for SolidWorks
3DEXPERIENCE `.SLDPRT` and `.SLDASM` files.

> **Note**: This handles the 3DEXPERIENCE/Connected platform format (custom
> binary container), NOT the legacy OLE2-based format used by desktop
> SolidWorks before ~2020.

## Quick Start

```bash
cd tools/solidworks

# Inspect a file
python3 cli/main.py inspect path/to/part.SLDPRT

# List all streams
python3 cli/main.py streams path/to/part.SLDPRT

# List feature dimensions
python3 cli/main.py dims path/to/part.SLDPRT

# List serialized C++ classes
python3 cli/main.py classes path/to/part.SLDPRT

# Extract all streams to a directory
python3 cli/main.py extract path/to/part.SLDPRT -o ./output/

# Extract Parasolid geometry
python3 cli/main.py parasolid path/to/part.SLDPRT -o body.xt
```

## Python API

```python
from swparse import SldFile

sld = SldFile.open("part.SLDPRT")

# File metadata
print(sld.header.version)        # 4
print(sld.header.checksum_hex)   # "f3cdb5cb"
print(len(sld.streams))          # 39

# List streams
for name in sld.stream_names():
    s = sld.streams[name]
    print(f"{name}: {s.size} bytes ({s.content_type.name})")

# Read XML metadata
xml = sld.get_xml("docProps/app.xml")
print(xml)  # <?xml version="1.0" ...

# Extract PNG preview
png = sld.get_preview_png()
Path("preview.png").write_bytes(png)

# Extract Parasolid geometry
from swparse.parasolid.extractor import extract_from_sld
chunks = extract_from_sld(sld)
for chunk in chunks:
    print(f"{chunk.chunk_type}: {chunk.version}, {len(chunk.data)} bytes")

# Modify dimensions and rebuild
from swparse.geometry.dimensions import modify_dimension_in_sld
modified = modify_dimension_in_sld(sld, old_meters=0.004, new_meters=0.005)
new_file = sld.rebuild(modified_streams=modified)
Path("modified.SLDPRT").write_bytes(new_file)
```

## What Can This Parse?

| Layer | What | Status |
|-------|------|--------|
| Container | Record markers, nibble-swapped names, deflate compression | 100% |
| XML Streams | Feature names, dimensions, assembly tree, properties | 100% |
| Preview Images | PNG and BMP thumbnails | 100% |
| Parasolid B-rep | Topology + geometry (double-compressed XT binary) | 90% |
| Binary Serialization | Class markers, string encoding, float64 values | 70% |
| Feature Tree | 317 class types identified, hierarchy known | 50% |
| Individual Fields | Field layouts within each class | 10% |

## File Format Summary

```
┌──────────────────────────────────────────────────┐
│ File Header (14-22 bytes)                        │
│   checksum(4) + version(4) + type_id(variable)   │
├──────────────────────────────────────────────────┤
│ Record 0                                         │
│   marker(6) + tag(4) + fields(16)                │
│   + nibble_swapped_name(N)                       │
│   + raw_deflate_data(M)                          │
├──────────────────────────────────────────────────┤
│ Record 1                                         │
│   ...                                            │
├──────────────────────────────────────────────────┤
│ Record N                                         │
│   ...                                            │
├──────────────────────────────────────────────────┤
│ Table of Contents (~20B entries)                 │
└──────────────────────────────────────────────────┘
```

Stream data encoding:
- **Names**: Nibble-swapped ASCII (`0xAB` ↔ `0xBA`)
- **Compression**: Raw deflate (zlib wbits=-15)
- **Strings**: `FF FE FF [len:u8] [UTF-16LE data]`
- **Classes**: `FF FF 01 00 [len:u16] [ASCII name]`
- **Coordinates**: float64 LE in meters (Parasolid uses BE)

## Documentation

Detailed format specifications in [`docs/`](docs/):

- [Container Format](docs/container-format.md) — File header, record structure, nibble swap
- [Binary Serialization](docs/binary-serialization.md) — Class markers, strings, object hierarchy
- [Parasolid Geometry](docs/parasolid-geometry.md) — Double compression, XT format, entity hierarchy
- [Stream Catalog](docs/stream-catalog.md) — All known stream names and purposes
- [Class Catalog](docs/class-catalog.md) — 317 C++ classes organized by category

## Project Structure

```
tools/solidworks/
├── cli/main.py                        CLI tool
├── src/swparse/
│   ├── container.py                   Top-level SldFile API
│   ├── header.py                      File header parsing
│   ├── records.py                     Record marker + field parsing
│   ├── streams.py                     Decompression + classification
│   ├── nibble.py                      Nibble-swap encoding
│   ├── serialization/
│   │   ├── classes.py                 Class marker parsing (FFFF0100)
│   │   ├── strings.py                 String decoding (FFFEFF + UTF-16)
│   │   └── primitives.py             Numeric type readers
│   ├── parasolid/
│   │   └── extractor.py              Parasolid XT extraction
│   ├── geometry/
│   │   └── dimensions.py             Dimension finding + modification
│   └── catalog/
│       └── stream_registry.py        Known stream database
└── docs/                              Format specifications
```

## Requirements

- Python 3.9+
- No external dependencies (stdlib only: `zlib`, `struct`, `xml`)

## Origin

Reverse-engineered from sample files: 38 SLDPRT parts + 1 SLDASM assembly
comprising a 3-wheel bicycle model. Created with SolidWorks 2015 (AppVersion
23.0), Parasolid kernel version 28.0.0174, by user "YTA" in July-August 2018.
