# Stream Catalog

Complete catalog of known stream names found across 38 SLDPRT + 1 SLDASM files.

## XML Metadata Streams (OPC Format)

These streams use the Open Packaging Convention (OPC), the same packaging
standard used by `.docx`, `.xlsx`, and other Office Open XML formats.

| Stream | Purpose |
|--------|---------|
| `[Content_Types].xml` | Maps file extensions to MIME types |
| `_rels/.rels` | Relationships between parts (links to app.xml, core.xml, etc.) |
| `docProps/app.xml` | Application: SolidWorks version, company, editing time |
| `docProps/core.xml` | Dublin Core: author, creation/modification dates |
| `docProps/custom.xml` | Custom properties: unit system settings |
| `docProps/Config-N-Properties.xml` | Per-configuration custom properties |
| `docProps/ISolidWorksInformation.xml` | SW-specific: file name, folder path, configuration |
| `swXmlContents/Features` | Minimal feature tree: file refs, configurations, model list |
| `swXmlContents/KeyWords` | **Feature names, types, and dimensions** (richest XML source) |
| `swXmlContents/COMPINSTANCETREE` | Assembly component hierarchy with 4×4 transforms |
| `swXmlContents/Tables` | Design tables (if present) |

### KeyWords XML Example

```xml
<Keywords id="1532617502" Name="Part12">
  <Configuration id="0" Name="Default" Type="ConfigurationManager"
                 Material="Material &lt;not specified&gt;"/>
  <Extrusion id="31" Name="Boss-Extrude1" Type="Boss-Extrude">
    <Dimension Name="D1">145</Dimension>
  </Extrusion>
  <Sketch id="23" Name="Sketch1" Dissectable="true">
    <Dimension Name="D1">&lt;MOD-DIAM&gt;8</Dimension>
  </Sketch>
</Keywords>
```

### COMPINSTANCETREE XML Example (Assembly)

```xml
<swSolidWorks swObjCount="150" swVersion="9000">
  <swHeader swObjCount="41">
    <swFile id="6" swDocType="PART" swCreationTime="1532003760"
            swPath="D:\...\1.SLDPRT"/>
    <swFile id="9" swDocType="PART" swCreationTime="1533005351"
            swPath="D:\...\2.SLDPRT"/>
    <!-- 38 more part references -->
  </swHeader>
  <swModelList>
    <!-- Component instances with transforms -->
  </swModelList>
</swSolidWorks>
```

## Binary Configuration Streams

| Stream | Purpose | Size Range |
|--------|---------|------------|
| `Contents/Config-0` | Root configuration: units, views, materials, bounding box | 25-80 KB |
| `Contents/Config-0-Partition` | Parasolid XT B-rep geometry (double-compressed) | 0.5-500 KB |
| `Contents/Config-0-ResolvedFeatures` | Full parametric feature tree (serialized classes) | 10-500 KB |
| `Contents/Config-0-GhostPartition` | Suppressed/hidden geometry | 0.5-2 KB |
| `Contents/Config-0-LWDATA` | Lightweight display data, cross-hatch | 1-3 KB |
| `Contents/Config-0-MatesList` | Assembly constraints (mates) | 0-270 KB |
| `Contents/Config-0-Attachment` | Assembly attachment info | ~2 B |
| `Contents/CMgr` | Configuration manager (moConfigurationMgr_c) | 1.5-30 KB |
| `Contents/CMgrHdr2` | Config names (e.g., "Default\<As Machined\>") | ~120 B |
| `Contents/CnfgObjs` | Configuration objects, per-body chooser data | 8 B - 12 KB |
| `Contents/Definition` | Part definition, coordinate transforms | 3.7 KB |
| `Contents/DisplayLists` | Tessellated triangle mesh for display | 11 KB - 15 MB |
| `Contents/MBLinkData` | Model-based link data | ~6 B |
| `Contents/OleItems` | Embedded OLE items | ~4 B |
| `Contents/User Units Table` | User unit system | ~0-1.5 KB |
| `Contents/View Orientation Data` | Saved view orientations | ~0 B |
| `Contents/eModelLic` | Model licensing | ~4 B |

## Preview Streams

| Stream | Format | Typical Size |
|--------|--------|-------------|
| `PreviewPNG` | PNG image | 6-46 KB |
| `Preview` | BMP image | 10-25 KB |

Both contain thumbnail previews of the part. PreviewPNG is directly
extractable as a `.png` file.

## Header & Version Streams

| Stream | Purpose |
|--------|---------|
| `Header2` | Document header (moHeader_c), bounding box |
| `ModelStamps` | Model timestamps (12 bytes) |
| `_MO_VERSION_9000/Biography` | File modification biography |
| `_MO_VERSION_9000/History` | Version history |
| `_MO_VERSION_9000/AssyVisualData` | Assembly visual data version |
| `_DL_VERSION_9000/DLUpdateStamp` | Display list update timestamp |

## Third-Party Streams

| Stream | Purpose |
|--------|---------|
| `ThirdPty/Animator` | Motion study (moAnimationManager_c) |
| `ThirdPty/CMMotionLoadMapU` | Motion loads |
| `ThirdPty/CM_MOTION_LOAD_1` | Motion load data |
| `ThirdPty/SWA_Schedules` | Schedule data |
| `ThirdPtyStore/VisualStates` | Display states (visStateKeyData_c) |
| `SWIFT/Config0-Schema1` | SWIFT schema |
| `SwDocContentMgr/SwDocContentMgrInfo` | Doc content manager |

## Virtual Component Streams (Assembly Only)

| Stream | Purpose |
|--------|---------|
| `VirtualComp/Belt3-2` | Virtual belt component |
| `VirtualComp/Part3` | Virtual part component |

These appear in assemblies and contain embedded part data for in-context
components.
