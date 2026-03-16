#!/usr/bin/env python3
"""swparse – SolidWorks 3DEXPERIENCE file inspection and manipulation CLI.

Usage:
    python -m cli.main inspect  <file>     Overview of file structure
    python -m cli.main streams  <file>     List all streams with sizes
    python -m cli.main classes  <file>     List serialized C++ classes
    python -m cli.main extract  <file>     Extract streams to directory
    python -m cli.main dims     <file>     List dimension values from XML
    python -m cli.main parasolid <file>    Extract Parasolid XT data
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

# Add src/ to path so we can import swparse
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from swparse import SldFile
from swparse.serialization.classes import find_all as find_classes, count_by_name
from swparse.serialization.strings import find_all as find_strings
from swparse.parasolid.extractor import extract_from_sld
from swparse.catalog.stream_registry import KNOWN_STREAMS


def cmd_inspect(args):
    """Print an overview of the file structure."""
    sld = SldFile.open(args.file)
    h = sld.header

    print(f"File:     {args.file}")
    print(f"Size:     {len(sld.raw):,} bytes")
    print(f"Version:  {h.version}")
    print(f"Checksum: {h.checksum_hex}")
    print(f"Type ID:  {h.type_id_hex}")
    print(f"Header:   {h.size} bytes")
    print(f"Records:  {len(sld.records)}")
    print(f"Streams:  {len(sld.streams)} (unique, decompressed)")
    print()

    # App version from XML
    xml = sld.get_xml("docProps/app.xml")
    if xml and "AppVersion" in xml:
        import re
        m = re.search(r"<AppVersion>(.*?)</AppVersion>", xml)
        if m:
            print(f"SolidWorks version: {m.group(1)}")

    # Author from core.xml
    xml = sld.get_xml("docProps/core.xml")
    if xml:
        import re
        m = re.search(r"<dc:lastModifiedBy>(.*?)</dc:lastModifiedBy>", xml)
        if m:
            print(f"Last modified by:   {m.group(1)}")
        m = re.search(r"<dcterms:created.*?>(.*?)</dcterms:created>", xml)
        if m:
            print(f"Created:            {m.group(1)}")
        m = re.search(r"<dcterms:modified.*?>(.*?)</dcterms:modified>", xml)
        if m:
            print(f"Modified:           {m.group(1)}")

    # Parasolid version
    chunks = extract_from_sld(sld)
    if chunks:
        print(f"Parasolid version:  {chunks[0].version}")
        print(f"Parasolid schema:   {chunks[0].schema}")

    # Preview
    png = sld.get_preview_png()
    if png:
        print(f"Preview PNG:        {len(png):,} bytes")

    print()

    # Stream summary by category
    xml_count = sum(1 for s in sld.streams.values() if s.content_type.name == "XML")
    bin_count = sum(1 for s in sld.streams.values() if s.content_type.name == "BINARY")
    print(f"XML streams:    {xml_count}")
    print(f"Binary streams: {bin_count}")


def cmd_streams(args):
    """List all streams with their sizes and types."""
    sld = SldFile.open(args.file)

    print(f"{'Stream Name':<50s} {'Size':>10s} {'Type':<8s} {'Known'}")
    print("-" * 80)

    for name in sorted(sld.streams.keys()):
        s = sld.streams[name]
        known = "yes" if name in KNOWN_STREAMS else ""
        print(f"  {name:<48s} {s.size:>8,}B  {s.content_type.name:<8s} {known}")

    total = sum(s.size for s in sld.streams.values())
    print(f"\n  Total decompressed: {total:,} bytes across {len(sld.streams)} streams")


def cmd_classes(args):
    """List all serialized C++ class instances."""
    sld = SldFile.open(args.file)

    all_instances = []
    for name, stream in sld.streams.items():
        if stream.data:
            instances = find_classes(stream.data)
            for inst in instances:
                inst_dict = {"stream": name, "class": inst.name, "offset": inst.offset}
                all_instances.append(inst_dict)

    # Count by name
    counts: dict[str, int] = {}
    for inst in all_instances:
        cn = inst["class"]
        counts[cn] = counts.get(cn, 0) + 1

    print(f"{'Class Name':<55s} {'Count':>5s}")
    print("-" * 62)
    for cn in sorted(counts.keys()):
        print(f"  {cn:<53s} {counts[cn]:>5d}")

    print(f"\n  {len(counts)} unique classes, {len(all_instances)} total instances")


def cmd_extract(args):
    """Extract streams to a directory."""
    sld = SldFile.open(args.file)
    out_dir = Path(args.output or f"{Path(args.file).stem}_streams")
    out_dir.mkdir(parents=True, exist_ok=True)

    for name, stream in sld.streams.items():
        if not stream.data:
            continue

        # Sanitize name for filesystem
        safe_name = name.replace("/", "__").replace("\\", "__")
        if stream.content_type.name == "XML":
            safe_name += ".xml" if not safe_name.endswith(".xml") else ""
        elif stream.content_type.name == "PNG":
            safe_name += ".png"
        elif stream.content_type.name == "BMP":
            safe_name += ".bmp"
        else:
            safe_name += ".bin"

        out_path = out_dir / safe_name
        out_path.write_bytes(stream.data)
        print(f"  {name} -> {out_path} ({stream.size:,} bytes)")

    # Also extract Parasolid chunks
    chunks = extract_from_sld(sld)
    for i, chunk in enumerate(chunks):
        out_path = out_dir / f"parasolid_{chunk.chunk_type}_{i}.xt"
        out_path.write_bytes(chunk.data)
        print(f"  Parasolid {chunk.chunk_type} -> {out_path} ({len(chunk.data):,} bytes)")

    print(f"\n  Extracted to {out_dir}/")


def cmd_dims(args):
    """List dimension values from the KeyWords XML."""
    sld = SldFile.open(args.file)

    for name, stream in sld.streams.items():
        if "KeyWords" not in name:
            continue
        # KeyWords can be classified as BINARY due to a leading byte before <?xml
        raw_text = stream.data.decode("utf-8", errors="replace")
        xml_start = raw_text.find("<?xml")
        if xml_start < 0:
            continue
        text = raw_text[xml_start:]

        import xml.etree.ElementTree as ET
        try:
            root = ET.fromstring(text)
        except ET.ParseError:
            continue

        part_name = root.get("Name", "?")
        print(f"Part: {part_name}\n")
        print(f"{'Feature':<30s} {'Type':<20s} {'Dimensions'}")
        print("-" * 80)

        for child in root:
            tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag
            attrs = dict(child.attrib)
            feat_name = attrs.get("Name", "")
            feat_type = attrs.get("Type", tag)

            dims = []
            for dim in child:
                dim_tag = dim.tag.split("}")[-1] if "}" in dim.tag else dim.tag
                if dim_tag == "Dimension":
                    dims.append(f"{dim.get('Name', '?')}={dim.text}")

            if dims or feat_type not in ("ConfigurationManager",):
                dim_str = ", ".join(dims) if dims else ""
                print(f"  {feat_name:<28s} {feat_type:<20s} {dim_str}")
        break


def cmd_parasolid(args):
    """Extract and display Parasolid XT data."""
    sld = SldFile.open(args.file)
    chunks = extract_from_sld(sld)

    if not chunks:
        print("No Parasolid data found.")
        return

    for i, chunk in enumerate(chunks):
        print(f"Chunk {i}: {chunk.chunk_type}")
        print(f"  Version: {chunk.version}")
        print(f"  Schema:  {chunk.schema}")
        print(f"  Size:    {len(chunk.data):,} bytes")
        print(f"  Header:  {chunk.data[:80].decode('ascii', errors='replace')}")
        print()

    if args.output:
        out = Path(args.output)
        out.write_bytes(chunks[0].data)
        print(f"Written main partition to {out}")


def cmd_validate(args):
    """Validate CRC32 checksums for all records."""
    import zlib as _zlib
    from swparse.streams import decompress as _decompress

    sld = SldFile.open(args.file)
    total = 0
    matched = 0
    failed = 0
    skipped = 0

    for r in sld.records:
        total += 1
        decompressed = _decompress(r)
        if decompressed is None:
            skipped += 1
            continue

        if r.uncompressed_size == 0:
            if r.field1 == 0:
                matched += 1
            else:
                failed += 1
                print(f"  FAIL (empty): {r.name} field1={r.field1:#x} expected 0")
            continue

        expected_crc = _zlib.crc32(decompressed) & 0xFFFFFFFF
        if r.field1 == expected_crc:
            matched += 1
        elif r.field1 == 2 * r.uncompressed_size:
            matched += 1  # small-record size sentinel
        else:
            failed += 1
            if args.verbose:
                print(f"  MISMATCH: {r.name} field1={r.field1:#010x} crc32={expected_crc:#010x} 2*uncomp={2*r.uncompressed_size:#x}")

    print(f"Records: {total} total, {matched} OK, {failed} failed, {skipped} skipped")
    if failed == 0:
        print("All checksums valid.")


def cmd_field_scan(args):
    """Run cross-file field entropy analysis for a class."""
    from swparse.analysis.class_extractor import extract_from_directory
    from swparse.analysis.field_differ import analyze, format_field_map

    instances = extract_from_directory(
        args.directory, args.class_name, suffix=args.suffix
    )
    if not instances:
        print(f"No instances of {args.class_name} found.")
        return

    fm = analyze(instances)
    print(format_field_map(fm))


def cmd_correlate(args):
    """Correlate XML dimension values with binary class offsets."""
    from swparse.analysis.dim_correlator import correlate, format_correlations

    sld = SldFile.open(args.file)
    corrs = correlate(sld)
    if not corrs:
        print("No correlations found.")
        return

    print(format_correlations(corrs))


def cmd_sketch(args):
    """Extract sketch topology and dimensions."""
    from swparse.geometry.sketch_entities import extract_sketch

    sld = SldFile.open(args.file)
    sketch = extract_sketch(sld)

    # Entity topology
    print(f"Sketch entities: {len(sketch.entities)} ({sketch.point_count} points, {sketch.line_count} lines/circles, {sketch.arc_count} arcs)")
    for e in sketch.entities:
        print(f"  {e.type_name:12s}  id={e.entity_id}")

    # Dimensions (the parametric values)
    if sketch.dimensions:
        print(f"\nDimensions: {len(sketch.dimensions)}")
        for d in sketch.dimensions:
            prefix = "diam " if d.is_diameter else ""
            print(f"  {d.name:<6s} = {prefix}{d.value_mm:.4f} mm  ({d.class_name})")

    # Chains
    if sketch.chains:
        print(f"\nChains: {len(sketch.chains)}")
        for c in sketch.chains:
            print(f"  {c.entity_ids}")


def cmd_generate(args):
    """Generate a Parasolid .x_t file for a basic shape."""
    from swparse.parasolid.writer import write_box_xt, write_cylinder_xt

    INCH = 0.0254
    shape = args.shape
    size = float(args.size) / 1000.0  # mm to meters

    if shape == "box":
        xt = write_box_xt(dx=size, dy=size, dz=size)
    elif shape == "cylinder":
        xt = write_cylinder_xt(radius=size / 2, height=size)
    else:
        print(f"Unknown shape: {shape}")
        return

    out = Path(args.output)
    out.write_text(xt)
    print(f"Generated {shape} ({args.size}mm) → {out} ({len(xt):,} chars)")
    print(f"Import this .x_t file into SolidWorks to get a valid part.")


def main():
    parser = argparse.ArgumentParser(
        prog="swparse",
        description="SolidWorks 3DEXPERIENCE file parser and inspector",
    )
    sub = parser.add_subparsers(dest="command")

    p = sub.add_parser("inspect", help="Overview of file structure")
    p.add_argument("file")

    p = sub.add_parser("streams", help="List all streams")
    p.add_argument("file")

    p = sub.add_parser("classes", help="List serialized C++ classes")
    p.add_argument("file")

    p = sub.add_parser("extract", help="Extract streams to directory")
    p.add_argument("file")
    p.add_argument("-o", "--output", help="Output directory")

    p = sub.add_parser("dims", help="List dimensions from XML")
    p.add_argument("file")

    p = sub.add_parser("parasolid", help="Extract Parasolid XT data")
    p.add_argument("file")
    p.add_argument("-o", "--output", help="Output file for Parasolid data")

    p = sub.add_parser("validate", help="Validate record CRC32 checksums")
    p.add_argument("file")
    p.add_argument("-v", "--verbose", action="store_true", help="Show individual mismatches")

    p = sub.add_parser("field-scan", help="Cross-file field entropy analysis")
    p.add_argument("class_name", help="Class name to analyze (e.g., moExtrusion_c)")
    p.add_argument("directory", help="Directory containing SLDPRT files")
    p.add_argument("--suffix", default=".SLDPRT", help="File suffix filter")

    p = sub.add_parser("correlate", help="Correlate XML dims with binary offsets")
    p.add_argument("file")

    p = sub.add_parser("sketch", help="Extract sketch geometry entities")
    p.add_argument("file")

    p = sub.add_parser("generate", help="Generate a Parasolid .x_t file")
    p.add_argument("shape", choices=["box", "cylinder"], help="Shape type")
    p.add_argument("size", help="Size in mm (edge length or diameter)")
    p.add_argument("-o", "--output", required=True, help="Output .x_t file")

    args = parser.parse_args()

    commands = {
        "inspect": cmd_inspect,
        "streams": cmd_streams,
        "classes": cmd_classes,
        "extract": cmd_extract,
        "dims": cmd_dims,
        "parasolid": cmd_parasolid,
        "validate": cmd_validate,
        "field-scan": cmd_field_scan,
        "correlate": cmd_correlate,
        "sketch": cmd_sketch,
        "generate": cmd_generate,
    }

    if args.command in commands:
        commands[args.command](args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
