# blockCAD Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a browser-based parametric CAD application with server-side OpenCascade.js geometry engine and React Three Fiber viewport.

**Architecture:** Server-side WASM (OpenCascade.js) evaluates a feature tree and returns tessellated triangles via oRPC. React Three Fiber renders the mesh. Zustand manages client state. PostgreSQL stores documents and features via Drizzle ORM.

**Tech Stack:** React 19, TanStack Start, React Three Fiber, Three.js, OpenCascade.js (WASM), oRPC, Drizzle ORM, Zustand, Allotment, Tailwind CSS

---

## Task 1: Install Dependencies

**Files:**
- Modify: `apps/web/package.json`
- Modify: `packages/api/package.json`

**Step 1: Install frontend 3D and layout deps**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD
cd apps/web && bun add three @react-three/fiber @react-three/drei allotment zustand && bun add -d @types/three
```

**Step 2: Install server-side CAD engine dep**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD
cd packages/api && bun add opencascade.js@beta
```

**Step 3: Verify install**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD && bun install
```

Expected: No errors. `node_modules` updated.

**Step 4: Commit**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD
git add apps/web/package.json packages/api/package.json bun.lock
git commit -m "feat: add Three.js, R3F, OpenCascade.js, Zustand, Allotment deps"
```

---

## Task 2: Database Schema for CAD Documents

**Files:**
- Create: `packages/db/src/schema/cad.ts`
- Modify: `packages/db/src/schema/index.ts`

**Step 1: Create the CAD schema**

Create `packages/db/src/schema/cad.ts`:

```typescript
import { relations } from "drizzle-orm";
import {
  pgTable,
  text,
  timestamp,
  boolean,
  integer,
  jsonb,
  index,
} from "drizzle-orm/pg-core";
import { user } from "./auth";

export const document = pgTable(
  "document",
  {
    id: text("id")
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    name: text("name").notNull().default("Untitled document"),
    ownerId: text("owner_id")
      .notNull()
      .references(() => user.id, { onDelete: "cascade" }),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => new Date())
      .notNull(),
  },
  (table) => [index("document_owner_idx").on(table.ownerId)],
);

export const documentRelations = relations(document, ({ one, many }) => ({
  owner: one(user, { fields: [document.ownerId], references: [user.id] }),
  elements: many(element),
}));

export const element = pgTable(
  "element",
  {
    id: text("id")
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    documentId: text("document_id")
      .notNull()
      .references(() => document.id, { onDelete: "cascade" }),
    name: text("name").notNull().default("Part Studio 1"),
    type: text("type").notNull().default("partstudio"), // partstudio | assembly
    index: integer("index").notNull().default(0),
    createdAt: timestamp("created_at").defaultNow().notNull(),
  },
  (table) => [index("element_document_idx").on(table.documentId)],
);

export const elementRelations = relations(element, ({ one, many }) => ({
  document: one(document, {
    fields: [element.documentId],
    references: [document.id],
  }),
  features: many(feature),
  featureGroups: many(featureGroup),
}));

export const featureGroup = pgTable("feature_group", {
  id: text("id")
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  elementId: text("element_id")
    .notNull()
    .references(() => element.id, { onDelete: "cascade" }),
  name: text("name").notNull(),
  collapsed: boolean("collapsed").notNull().default(false),
});

export const featureGroupRelations = relations(featureGroup, ({ one }) => ({
  element: one(element, {
    fields: [featureGroup.elementId],
    references: [element.id],
  }),
}));

export const feature = pgTable(
  "feature",
  {
    id: text("id")
      .primaryKey()
      .$defaultFn(() => crypto.randomUUID()),
    elementId: text("element_id")
      .notNull()
      .references(() => element.id, { onDelete: "cascade" }),
    index: integer("index").notNull(),
    type: text("type").notNull(), // sketch | extrude | revolve | fillet | chamfer | boolean | sweep | loft | shell | pattern | mirror | draft | plane
    name: text("name").notNull(),
    parameters: jsonb("parameters").notNull().default({}),
    suppressed: boolean("suppressed").notNull().default(false),
    groupId: text("group_id").references(() => featureGroup.id, {
      onDelete: "set null",
    }),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => new Date())
      .notNull(),
  },
  (table) => [
    index("feature_element_idx").on(table.elementId),
    index("feature_index_idx").on(table.elementId, table.index),
  ],
);

export const featureRelations = relations(feature, ({ one }) => ({
  element: one(element, {
    fields: [feature.elementId],
    references: [element.id],
  }),
  group: one(featureGroup, {
    fields: [feature.groupId],
    references: [featureGroup.id],
  }),
}));
```

**Step 2: Export from schema index**

Modify `packages/db/src/schema/index.ts`:

```typescript
export * from "./auth";
export * from "./cad";
```

**Step 3: Push schema to database**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD && bun run db:push
```

Expected: Tables created successfully.

**Step 4: Commit**

```bash
git add packages/db/src/schema/cad.ts packages/db/src/schema/index.ts
git commit -m "feat: add CAD document/element/feature database schema"
```

---

## Task 3: CAD Engine - OpenCascade.js Initialization & Tessellator

**Files:**
- Create: `packages/api/src/engine/init.ts`
- Create: `packages/api/src/engine/tessellator.ts`
- Create: `packages/api/src/engine/types.ts`

**Step 1: Create shared types**

Create `packages/api/src/engine/types.ts`:

```typescript
export interface TessellatedPart {
  partId: string;
  name: string;
  color: [number, number, number, number];
  mesh: {
    positions: number[];
    normals: number[];
    indices: number[];
  };
  edges: {
    positions: number[];
  };
}

export interface FeatureParams {
  type: string;
  name: string;
  parameters: Record<string, unknown>;
  suppressed: boolean;
}

export type OperationType = "new" | "add" | "remove" | "intersect";

export interface ExtrudeParams {
  sketchId: string;
  profiles: string[];
  direction: "normal" | [number, number, number];
  depth: { value: number; unit: string };
  operation: OperationType;
  symmetric: boolean;
  draft: { angle: number; inward: boolean };
}

export interface FilletParams {
  edges: string[];
  radius: { value: number; unit: string };
  tangentPropagation: boolean;
}

export interface RevolveParams {
  sketchId: string;
  profiles: string[];
  axis: string;
  angle: { value: number; unit: string };
  operation: OperationType;
}

export interface BooleanParams {
  operation: "union" | "subtract" | "intersect";
  targetParts: string[];
  toolParts: string[];
  keepTools: boolean;
}

export interface ChamferParams {
  edges: string[];
  distance: { value: number; unit: string };
  symmetric: boolean;
}

export interface SweepParams {
  sketchId: string;
  profiles: string[];
  pathId: string;
  operation: OperationType;
}

export interface LoftParams {
  profiles: string[];
  guides: string[];
  operation: OperationType;
}

export interface ShellParams {
  faces: string[];
  thickness: { value: number; unit: string };
}

export interface LinearPatternParams {
  features: string[];
  direction: string;
  count: number;
  distance: { value: number; unit: string };
}

export interface CircularPatternParams {
  features: string[];
  axis: string;
  count: number;
  angle: { value: number; unit: string };
}

export interface MirrorParams {
  features: string[];
  mirrorPlane: string;
}

export interface DraftParams {
  faces: string[];
  pullDirection: string;
  angle: { value: number; unit: string };
}
```

**Step 2: Create engine initialization**

Create `packages/api/src/engine/init.ts`:

```typescript
let ocInstance: any = null;
let initPromise: Promise<any> | null = null;

export async function getOC(): Promise<any> {
  if (ocInstance) return ocInstance;

  if (!initPromise) {
    initPromise = (async () => {
      // Dynamic import for Node.js WASM loading
      const initOpenCascade = (await import("opencascade.js/dist/node.js"))
        .default;
      ocInstance = await initOpenCascade();
      console.log("[CadEngine] OpenCascade.js initialized");
      return ocInstance;
    })();
  }

  return initPromise;
}
```

**Step 3: Create tessellator**

Create `packages/api/src/engine/tessellator.ts`:

```typescript
import type { TessellatedPart } from "./types";
import { getOC } from "./init";

export async function tessellateShape(
  shape: any,
  partId: string,
  name: string,
  color: [number, number, number, number] = [0.6, 0.7, 0.8, 1.0],
  linearDeflection: number = 0.1,
  angularDeflection: number = 0.5,
): Promise<TessellatedPart> {
  const oc = await getOC();

  // Mesh the shape
  const mesh = new oc.BRepMesh_IncrementalMesh_2(
    shape,
    linearDeflection,
    false,
    angularDeflection,
    false,
  );
  mesh.Perform(new oc.Message_ProgressRange_1());

  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  const edgePositions: number[] = [];

  // Extract face tessellation
  const explorer = new oc.TopExp_Explorer_2(
    shape,
    oc.TopAbs_ShapeEnum.TopAbs_FACE,
    oc.TopAbs_ShapeEnum.TopAbs_SHAPE,
  );

  while (explorer.More()) {
    const face = oc.TopoDS.Face_1(explorer.Current());
    const location = new oc.TopLoc_Location_1();
    const triangulation = oc.BRep_Tool.Triangulation(face, location, 0);

    if (!triangulation.IsNull()) {
      const tri = triangulation.get();
      const nbNodes = tri.NbNodes();
      const nbTriangles = tri.NbTriangles();
      const baseIndex = positions.length / 3;

      // Extract vertices and normals
      for (let i = 1; i <= nbNodes; i++) {
        const node = tri.Node(i);
        const transformedNode = node.Transformed(location.Transformation());
        positions.push(transformedNode.X(), transformedNode.Y(), transformedNode.Z());

        // Compute normal if available
        if (tri.HasNormals()) {
          const normal = tri.Normal(i);
          normals.push(normal.X(), normal.Y(), normal.Z());
        } else {
          normals.push(0, 0, 1);
        }
      }

      // Extract triangle indices
      const orientation = face.Orientation_1();
      for (let i = 1; i <= nbTriangles; i++) {
        const triangle = tri.Triangle(i);
        const n1 = triangle.Value(1) - 1 + baseIndex;
        const n2 = triangle.Value(2) - 1 + baseIndex;
        const n3 = triangle.Value(3) - 1 + baseIndex;

        // Respect face orientation
        if (orientation === oc.TopAbs_Orientation.TopAbs_REVERSED) {
          indices.push(n1, n3, n2);
        } else {
          indices.push(n1, n2, n3);
        }
      }
    }

    explorer.Next();
  }

  // Extract edge tessellation for wireframe
  const edgeExplorer = new oc.TopExp_Explorer_2(
    shape,
    oc.TopAbs_ShapeEnum.TopAbs_EDGE,
    oc.TopAbs_ShapeEnum.TopAbs_SHAPE,
  );

  while (edgeExplorer.More()) {
    const edge = oc.TopoDS.Edge_1(edgeExplorer.Current());
    const location = new oc.TopLoc_Location_1();
    const poly = oc.BRep_Tool.Polygon3D(edge, location);

    if (!poly.IsNull()) {
      const p = poly.get();
      const nbNodes = p.NbNodes();
      for (let i = 1; i <= nbNodes; i++) {
        const node = p.Node(i);
        const transformed = node.Transformed(location.Transformation());
        edgePositions.push(transformed.X(), transformed.Y(), transformed.Z());
      }
    }

    edgeExplorer.Next();
  }

  // Cleanup
  mesh.delete();
  explorer.delete();
  edgeExplorer.delete();

  return {
    partId,
    name,
    color,
    mesh: { positions, normals, indices },
    edges: { positions: edgePositions },
  };
}
```

**Step 4: Commit**

```bash
git add packages/api/src/engine/
git commit -m "feat: add OpenCascade.js engine init and tessellator"
```

---

## Task 4: CAD Engine - Feature Operations

**Files:**
- Create: `packages/api/src/engine/operations/extrude.ts`
- Create: `packages/api/src/engine/operations/revolve.ts`
- Create: `packages/api/src/engine/operations/fillet.ts`
- Create: `packages/api/src/engine/operations/chamfer.ts`
- Create: `packages/api/src/engine/operations/boolean.ts`
- Create: `packages/api/src/engine/operations/sweep.ts`
- Create: `packages/api/src/engine/operations/loft.ts`
- Create: `packages/api/src/engine/operations/shell.ts`
- Create: `packages/api/src/engine/operations/pattern.ts`
- Create: `packages/api/src/engine/operations/mirror.ts`
- Create: `packages/api/src/engine/operations/primitives.ts`
- Create: `packages/api/src/engine/operations/index.ts`

**Step 1: Create primitives (for initial testing)**

Create `packages/api/src/engine/operations/primitives.ts`:

```typescript
import { getOC } from "../init";

export async function makeBox(
  width: number,
  height: number,
  depth: number,
): Promise<any> {
  const oc = await getOC();
  return new oc.BRepPrimAPI_MakeBox_3(width, height, depth).Shape();
}

export async function makeCylinder(
  radius: number,
  height: number,
): Promise<any> {
  const oc = await getOC();
  return new oc.BRepPrimAPI_MakeCylinder_3(radius, height).Shape();
}

export async function makeSphere(radius: number): Promise<any> {
  const oc = await getOC();
  return new oc.BRepPrimAPI_MakeSphere_5(radius).Shape();
}
```

**Step 2: Create extrude operation**

Create `packages/api/src/engine/operations/extrude.ts`:

```typescript
import { getOC } from "../init";

export async function extrude(
  face: any,
  direction: [number, number, number],
  depth: number,
): Promise<any> {
  const oc = await getOC();
  const dir = new oc.gp_Vec_4(direction[0], direction[1], direction[2]);
  dir.Normalize();
  dir.Scale(depth);
  const prism = new oc.BRepPrimAPI_MakePrism_1(face, dir, false, true);
  return prism.Shape();
}
```

**Step 3: Create revolve operation**

Create `packages/api/src/engine/operations/revolve.ts`:

```typescript
import { getOC } from "../init";

export async function revolve(
  face: any,
  axisOrigin: [number, number, number],
  axisDirection: [number, number, number],
  angleDegrees: number,
): Promise<any> {
  const oc = await getOC();
  const origin = new oc.gp_Pnt_3(axisOrigin[0], axisOrigin[1], axisOrigin[2]);
  const dir = new oc.gp_Dir_4(
    axisDirection[0],
    axisDirection[1],
    axisDirection[2],
  );
  const axis = new oc.gp_Ax1_2(origin, dir);
  const angleRad = (angleDegrees * Math.PI) / 180;
  const revol = new oc.BRepPrimAPI_MakeRevol_1(face, axis, angleRad, true);
  return revol.Shape();
}
```

**Step 4: Create fillet operation**

Create `packages/api/src/engine/operations/fillet.ts`:

```typescript
import { getOC } from "../init";

export async function fillet(shape: any, radius: number): Promise<any> {
  const oc = await getOC();
  const filletBuilder = new oc.BRepFilletAPI_MakeFillet(
    shape,
    oc.ChFi3d_FilletShape.ChFi3d_Rational,
  );

  // Add all edges by default (caller can filter)
  const explorer = new oc.TopExp_Explorer_2(
    shape,
    oc.TopAbs_ShapeEnum.TopAbs_EDGE,
    oc.TopAbs_ShapeEnum.TopAbs_SHAPE,
  );

  while (explorer.More()) {
    const edge = oc.TopoDS.Edge_1(explorer.Current());
    filletBuilder.Add_2(radius, edge);
    explorer.Next();
  }

  explorer.delete();
  return filletBuilder.Shape();
}

export async function filletEdges(
  shape: any,
  edges: any[],
  radius: number,
): Promise<any> {
  const oc = await getOC();
  const filletBuilder = new oc.BRepFilletAPI_MakeFillet(
    shape,
    oc.ChFi3d_FilletShape.ChFi3d_Rational,
  );

  for (const edge of edges) {
    filletBuilder.Add_2(radius, edge);
  }

  return filletBuilder.Shape();
}
```

**Step 5: Create chamfer operation**

Create `packages/api/src/engine/operations/chamfer.ts`:

```typescript
import { getOC } from "../init";

export async function chamferEdges(
  shape: any,
  edges: any[],
  distance: number,
): Promise<any> {
  const oc = await getOC();
  const chamferBuilder = new oc.BRepFilletAPI_MakeChamfer(shape);

  for (const edge of edges) {
    chamferBuilder.Add_2(distance, edge);
  }

  return chamferBuilder.Shape();
}
```

**Step 6: Create boolean operation**

Create `packages/api/src/engine/operations/boolean.ts`:

```typescript
import { getOC } from "../init";

export async function booleanUnion(
  shapeA: any,
  shapeB: any,
): Promise<any> {
  const oc = await getOC();
  const fuse = new oc.BRepAlgoAPI_Fuse_3(shapeA, shapeB, new oc.Message_ProgressRange_1());
  return fuse.Shape();
}

export async function booleanCut(
  shapeA: any,
  shapeB: any,
): Promise<any> {
  const oc = await getOC();
  const cut = new oc.BRepAlgoAPI_Cut_3(shapeA, shapeB, new oc.Message_ProgressRange_1());
  return cut.Shape();
}

export async function booleanIntersect(
  shapeA: any,
  shapeB: any,
): Promise<any> {
  const oc = await getOC();
  const common = new oc.BRepAlgoAPI_Common_3(shapeA, shapeB, new oc.Message_ProgressRange_1());
  return common.Shape();
}
```

**Step 7: Create sweep operation**

Create `packages/api/src/engine/operations/sweep.ts`:

```typescript
import { getOC } from "../init";

export async function sweep(
  profile: any,
  spine: any,
): Promise<any> {
  const oc = await getOC();
  const wire = oc.TopoDS.Wire_1(spine);
  const pipe = new oc.BRepOffsetAPI_MakePipe_1(wire, profile);
  return pipe.Shape();
}
```

**Step 8: Create loft operation**

Create `packages/api/src/engine/operations/loft.ts`:

```typescript
import { getOC } from "../init";

export async function loft(
  profiles: any[],
  isSolid: boolean = true,
): Promise<any> {
  const oc = await getOC();
  const thruSections = new oc.BRepOffsetAPI_ThruSections(isSolid, false, 1e-6);

  for (const profile of profiles) {
    const wire = oc.TopoDS.Wire_1(profile);
    thruSections.AddWire(wire);
  }

  thruSections.Build(new oc.Message_ProgressRange_1());
  return thruSections.Shape();
}
```

**Step 9: Create shell operation**

Create `packages/api/src/engine/operations/shell.ts`:

```typescript
import { getOC } from "../init";

export async function shell(
  shape: any,
  facesToRemove: any[],
  thickness: number,
): Promise<any> {
  const oc = await getOC();
  const faceList = new oc.TopTools_ListOfShape_1();

  for (const face of facesToRemove) {
    faceList.Append_1(face);
  }

  const shellBuilder = new oc.BRepOffsetAPI_MakeThickSolid();
  shellBuilder.MakeThickSolidByJoin(
    shape,
    faceList,
    thickness,
    1e-3,
    oc.BRepOffset_Mode.BRepOffset_Skin,
    false,
    false,
    oc.GeomAbs_JoinType.GeomAbs_Arc,
    false,
    new oc.Message_ProgressRange_1(),
  );

  return shellBuilder.Shape();
}
```

**Step 10: Create pattern operation**

Create `packages/api/src/engine/operations/pattern.ts`:

```typescript
import { getOC } from "../init";
import { booleanUnion } from "./boolean";

export async function linearPattern(
  shape: any,
  direction: [number, number, number],
  count: number,
  distance: number,
): Promise<any> {
  const oc = await getOC();
  let result = shape;

  for (let i = 1; i < count; i++) {
    const trsf = new oc.gp_Trsf_1();
    trsf.SetTranslation_1(
      new oc.gp_Vec_4(
        direction[0] * distance * i,
        direction[1] * distance * i,
        direction[2] * distance * i,
      ),
    );
    const transformed = new oc.BRepBuilderAPI_Transform_2(shape, trsf, true);
    result = await booleanUnion(result, transformed.Shape());
  }

  return result;
}

export async function circularPattern(
  shape: any,
  axisOrigin: [number, number, number],
  axisDirection: [number, number, number],
  count: number,
  totalAngleDegrees: number,
): Promise<any> {
  const oc = await getOC();
  let result = shape;
  const stepAngle = (totalAngleDegrees / count) * (Math.PI / 180);

  const origin = new oc.gp_Pnt_3(axisOrigin[0], axisOrigin[1], axisOrigin[2]);
  const dir = new oc.gp_Dir_4(
    axisDirection[0],
    axisDirection[1],
    axisDirection[2],
  );
  const axis = new oc.gp_Ax1_2(origin, dir);

  for (let i = 1; i < count; i++) {
    const trsf = new oc.gp_Trsf_1();
    trsf.SetRotation_1(axis, stepAngle * i);
    const transformed = new oc.BRepBuilderAPI_Transform_2(shape, trsf, true);
    result = await booleanUnion(result, transformed.Shape());
  }

  return result;
}
```

**Step 11: Create mirror operation**

Create `packages/api/src/engine/operations/mirror.ts`:

```typescript
import { getOC } from "../init";
import { booleanUnion } from "./boolean";

export async function mirror(
  shape: any,
  planeOrigin: [number, number, number],
  planeNormal: [number, number, number],
): Promise<any> {
  const oc = await getOC();
  const origin = new oc.gp_Pnt_3(planeOrigin[0], planeOrigin[1], planeOrigin[2]);
  const dir = new oc.gp_Dir_4(planeNormal[0], planeNormal[1], planeNormal[2]);
  const ax2 = new oc.gp_Ax2_3(origin, dir);

  const trsf = new oc.gp_Trsf_1();
  trsf.SetMirror_3(ax2);

  const transformed = new oc.BRepBuilderAPI_Transform_2(shape, trsf, true);
  return booleanUnion(shape, transformed.Shape());
}
```

**Step 12: Create operations index**

Create `packages/api/src/engine/operations/index.ts`:

```typescript
export { makeBox, makeCylinder, makeSphere } from "./primitives";
export { extrude } from "./extrude";
export { revolve } from "./revolve";
export { fillet, filletEdges } from "./fillet";
export { chamferEdges } from "./chamfer";
export { booleanUnion, booleanCut, booleanIntersect } from "./boolean";
export { sweep } from "./sweep";
export { loft } from "./loft";
export { shell } from "./shell";
export { linearPattern, circularPattern } from "./pattern";
export { mirror } from "./mirror";
```

**Step 13: Commit**

```bash
git add packages/api/src/engine/
git commit -m "feat: add all CAD operations (extrude, revolve, fillet, boolean, sweep, loft, shell, pattern, mirror)"
```

---

## Task 5: CAD Engine - Session Manager

**Files:**
- Create: `packages/api/src/engine/session.ts`
- Create: `packages/api/src/engine/index.ts`

**Step 1: Create session manager**

Create `packages/api/src/engine/session.ts`:

```typescript
import type { TessellatedPart, FeatureParams } from "./types";
import { getOC } from "./init";
import { tessellateShape } from "./tessellator";
import { makeBox, makeCylinder, makeSphere } from "./operations";
import { extrude } from "./operations/extrude";
import { fillet } from "./operations/fillet";
import { booleanUnion, booleanCut, booleanIntersect } from "./operations/boolean";

interface SessionState {
  shapes: Map<string, any>; // partId -> OCCT shape
  lastFeatureIndex: number;
}

const sessions = new Map<string, SessionState>();

function getSession(elementId: string): SessionState {
  let session = sessions.get(elementId);
  if (!session) {
    session = { shapes: new Map(), lastFeatureIndex: -1 };
    sessions.set(elementId, session);
  }
  return session;
}

export function clearSession(elementId: string): void {
  sessions.delete(elementId);
}

export async function rebuildModel(
  elementId: string,
  features: FeatureParams[],
): Promise<TessellatedPart[]> {
  const oc = await getOC();
  const session = getSession(elementId);

  // Clear existing shapes
  session.shapes.clear();

  let partCounter = 0;

  for (const feature of features) {
    if (feature.suppressed) continue;

    try {
      switch (feature.type) {
        case "box": {
          const params = feature.parameters as any;
          const shape = await makeBox(
            params.width ?? 50,
            params.height ?? 50,
            params.depth ?? 50,
          );
          const partId = `part_${partCounter++}`;
          session.shapes.set(partId, shape);
          break;
        }
        case "cylinder": {
          const params = feature.parameters as any;
          const shape = await makeCylinder(
            params.radius ?? 25,
            params.height ?? 50,
          );
          const partId = `part_${partCounter++}`;
          session.shapes.set(partId, shape);
          break;
        }
        case "sphere": {
          const params = feature.parameters as any;
          const shape = await makeSphere(params.radius ?? 25);
          const partId = `part_${partCounter++}`;
          session.shapes.set(partId, shape);
          break;
        }
        // Additional operations will be wired up as sketch system matures
      }
    } catch (err) {
      console.error(`[CadEngine] Failed to evaluate feature "${feature.name}":`, err);
    }
  }

  // Tessellate all parts
  const results: TessellatedPart[] = [];
  for (const [partId, shape] of session.shapes) {
    const tessellated = await tessellateShape(shape, partId, `Part ${partId}`);
    results.push(tessellated);
  }

  session.lastFeatureIndex = features.length - 1;
  return results;
}
```

**Step 2: Create engine index**

Create `packages/api/src/engine/index.ts`:

```typescript
export { getOC } from "./init";
export { tessellateShape } from "./tessellator";
export { rebuildModel, clearSession } from "./session";
export * from "./types";
export * from "./operations";
```

**Step 3: Commit**

```bash
git add packages/api/src/engine/
git commit -m "feat: add CAD session manager with model rebuild pipeline"
```

---

## Task 6: oRPC CAD Router

**Files:**
- Create: `packages/api/src/routers/cad.ts`
- Modify: `packages/api/src/routers/index.ts`

**Step 1: Create CAD router**

Create `packages/api/src/routers/cad.ts`:

```typescript
import { z } from "zod";
import { eq, asc } from "drizzle-orm";
import { db } from "@blockCAD-temp/db";
import { document, element, feature } from "@blockCAD-temp/db/schema/cad";
import { protectedProcedure } from "../index";
import { rebuildModel, clearSession } from "../engine";
import type { FeatureParams } from "../engine/types";

export const cadRouter = {
  // Document CRUD
  createDocument: protectedProcedure
    .input(z.object({ name: z.string().optional() }))
    .handler(async ({ context, input }) => {
      const [doc] = await db
        .insert(document)
        .values({
          name: input.name ?? "Untitled document",
          ownerId: context.session.user.id,
        })
        .returning();

      // Create default Part Studio element
      const [elem] = await db
        .insert(element)
        .values({
          documentId: doc.id,
          name: "Part Studio 1",
          type: "partstudio",
          index: 0,
        })
        .returning();

      return { document: doc, element: elem };
    }),

  listDocuments: protectedProcedure.handler(async ({ context }) => {
    return db
      .select()
      .from(document)
      .where(eq(document.ownerId, context.session.user.id))
      .orderBy(asc(document.updatedAt));
  }),

  getDocument: protectedProcedure
    .input(z.object({ documentId: z.string() }))
    .handler(async ({ input }) => {
      const [doc] = await db
        .select()
        .from(document)
        .where(eq(document.id, input.documentId));

      if (!doc) throw new Error("Document not found");

      const elements = await db
        .select()
        .from(element)
        .where(eq(element.documentId, input.documentId))
        .orderBy(asc(element.index));

      return { document: doc, elements };
    }),

  // Feature operations
  getFeatures: protectedProcedure
    .input(z.object({ elementId: z.string() }))
    .handler(async ({ input }) => {
      return db
        .select()
        .from(feature)
        .where(eq(feature.elementId, input.elementId))
        .orderBy(asc(feature.index));
    }),

  addFeature: protectedProcedure
    .input(
      z.object({
        elementId: z.string(),
        type: z.string(),
        name: z.string(),
        parameters: z.record(z.unknown()),
      }),
    )
    .handler(async ({ input }) => {
      // Get current max index
      const existing = await db
        .select()
        .from(feature)
        .where(eq(feature.elementId, input.elementId))
        .orderBy(asc(feature.index));

      const nextIndex = existing.length;

      const [feat] = await db
        .insert(feature)
        .values({
          elementId: input.elementId,
          index: nextIndex,
          type: input.type,
          name: input.name,
          parameters: input.parameters,
        })
        .returning();

      // Rebuild model
      const allFeatures = [...existing, feat].map((f) => ({
        type: f.type,
        name: f.name,
        parameters: f.parameters as Record<string, unknown>,
        suppressed: f.suppressed,
      }));

      const tessellation = await rebuildModel(input.elementId, allFeatures);

      return { feature: feat, tessellation };
    }),

  updateFeature: protectedProcedure
    .input(
      z.object({
        featureId: z.string(),
        elementId: z.string(),
        name: z.string().optional(),
        parameters: z.record(z.unknown()).optional(),
        suppressed: z.boolean().optional(),
      }),
    )
    .handler(async ({ input }) => {
      const updates: Record<string, unknown> = {};
      if (input.name !== undefined) updates.name = input.name;
      if (input.parameters !== undefined) updates.parameters = input.parameters;
      if (input.suppressed !== undefined) updates.suppressed = input.suppressed;

      await db
        .update(feature)
        .set(updates)
        .where(eq(feature.id, input.featureId));

      // Rebuild model
      const allFeatures = await db
        .select()
        .from(feature)
        .where(eq(feature.elementId, input.elementId))
        .orderBy(asc(feature.index));

      const tessellation = await rebuildModel(
        input.elementId,
        allFeatures.map((f) => ({
          type: f.type,
          name: f.name,
          parameters: f.parameters as Record<string, unknown>,
          suppressed: f.suppressed,
        })),
      );

      return { tessellation };
    }),

  deleteFeature: protectedProcedure
    .input(z.object({ featureId: z.string(), elementId: z.string() }))
    .handler(async ({ input }) => {
      await db.delete(feature).where(eq(feature.id, input.featureId));

      // Rebuild model
      const allFeatures = await db
        .select()
        .from(feature)
        .where(eq(feature.elementId, input.elementId))
        .orderBy(asc(feature.index));

      const tessellation = await rebuildModel(
        input.elementId,
        allFeatures.map((f) => ({
          type: f.type,
          name: f.name,
          parameters: f.parameters as Record<string, unknown>,
          suppressed: f.suppressed,
        })),
      );

      return { tessellation };
    }),

  getTessellation: protectedProcedure
    .input(z.object({ elementId: z.string() }))
    .handler(async ({ input }) => {
      const allFeatures = await db
        .select()
        .from(feature)
        .where(eq(feature.elementId, input.elementId))
        .orderBy(asc(feature.index));

      return rebuildModel(
        input.elementId,
        allFeatures.map((f) => ({
          type: f.type,
          name: f.name,
          parameters: f.parameters as Record<string, unknown>,
          suppressed: f.suppressed,
        })),
      );
    }),
};
```

**Step 2: Wire into app router**

Modify `packages/api/src/routers/index.ts`:

```typescript
import type { RouterClient } from "@orpc/server";

import { protectedProcedure, publicProcedure } from "../index";
import { cadRouter } from "./cad";

export const appRouter = {
  healthCheck: publicProcedure.handler(() => {
    return "OK";
  }),
  privateData: protectedProcedure.handler(({ context }) => {
    return {
      message: "This is private",
      user: context.session?.user,
    };
  }),
  cad: cadRouter,
};
export type AppRouter = typeof appRouter;
export type AppRouterClient = RouterClient<typeof appRouter>;
```

**Step 3: Commit**

```bash
git add packages/api/src/routers/
git commit -m "feat: add oRPC CAD router with document/feature CRUD and tessellation"
```

---

## Task 7: Zustand Store

**Files:**
- Create: `apps/web/src/stores/cad-store.ts`

**Step 1: Create the store**

Create `apps/web/src/stores/cad-store.ts`:

```typescript
import { create } from "zustand";

interface Document {
  id: string;
  name: string;
}

interface Element {
  id: string;
  name: string;
  type: string;
}

interface Feature {
  id: string;
  elementId: string;
  index: number;
  type: string;
  name: string;
  parameters: Record<string, unknown>;
  suppressed: boolean;
  groupId: string | null;
}

interface TessellatedPart {
  partId: string;
  name: string;
  color: [number, number, number, number];
  mesh: {
    positions: number[];
    normals: number[];
    indices: number[];
  };
  edges: {
    positions: number[];
  };
}

type ToolType =
  | "sketch"
  | "extrude"
  | "revolve"
  | "sweep"
  | "loft"
  | "fillet"
  | "chamfer"
  | "shell"
  | "boolean"
  | "linearPattern"
  | "circularPattern"
  | "mirror"
  | "draft"
  | "plane"
  | "box"
  | "cylinder"
  | "sphere";

type ViewMode = "shaded" | "wireframe" | "hidden-line";

interface CadState {
  // Document
  document: Document | null;
  elements: Element[];
  activeElementId: string | null;

  // Features
  features: Feature[];
  selectedFeatureId: string | null;

  // 3D viewport
  meshData: TessellatedPart[];
  selectedEntities: string[];
  viewMode: ViewMode;

  // Tool state
  activeTool: ToolType | null;
  toolParams: Record<string, unknown>;

  // UI
  leftPanelOpen: boolean;
  rightPanelOpen: boolean;
  activeRightPanel: string | null;
}

interface CadActions {
  // Document
  setDocument: (doc: Document, elements: Element[]) => void;
  setActiveElement: (elementId: string) => void;

  // Features
  setFeatures: (features: Feature[]) => void;
  selectFeature: (featureId: string | null) => void;

  // Mesh
  setMeshData: (data: TessellatedPart[]) => void;

  // Selection
  setSelectedEntities: (entities: string[]) => void;
  clearSelection: () => void;

  // Tools
  setActiveTool: (tool: ToolType | null) => void;
  setToolParams: (params: Record<string, unknown>) => void;
  resetTool: () => void;

  // View
  setViewMode: (mode: ViewMode) => void;

  // UI
  toggleLeftPanel: () => void;
  toggleRightPanel: (panel?: string) => void;
}

export const useCadStore = create<CadState & CadActions>((set) => ({
  // Initial state
  document: null,
  elements: [],
  activeElementId: null,
  features: [],
  selectedFeatureId: null,
  meshData: [],
  selectedEntities: [],
  viewMode: "shaded",
  activeTool: null,
  toolParams: {},
  leftPanelOpen: true,
  rightPanelOpen: false,
  activeRightPanel: null,

  // Actions
  setDocument: (doc, elements) =>
    set({ document: doc, elements, activeElementId: elements[0]?.id ?? null }),
  setActiveElement: (elementId) => set({ activeElementId: elementId }),
  setFeatures: (features) => set({ features }),
  selectFeature: (featureId) => set({ selectedFeatureId: featureId }),
  setMeshData: (data) => set({ meshData: data }),
  setSelectedEntities: (entities) => set({ selectedEntities: entities }),
  clearSelection: () => set({ selectedEntities: [] }),
  setActiveTool: (tool) => set({ activeTool: tool, toolParams: {} }),
  setToolParams: (params) =>
    set((state) => ({ toolParams: { ...state.toolParams, ...params } })),
  resetTool: () => set({ activeTool: null, toolParams: {} }),
  setViewMode: (mode) => set({ viewMode: mode }),
  toggleLeftPanel: () =>
    set((state) => ({ leftPanelOpen: !state.leftPanelOpen })),
  toggleRightPanel: (panel) =>
    set((state) => {
      if (panel && state.activeRightPanel === panel) {
        return { rightPanelOpen: false, activeRightPanel: null };
      }
      return {
        rightPanelOpen: true,
        activeRightPanel: panel ?? state.activeRightPanel,
      };
    }),
}));
```

**Step 2: Commit**

```bash
git add apps/web/src/stores/
git commit -m "feat: add Zustand CAD store with document, feature, mesh, and tool state"
```

---

## Task 8: 3D Viewport Components

**Files:**
- Create: `apps/web/src/components/cad/viewport/Canvas3D.tsx`
- Create: `apps/web/src/components/cad/viewport/ModelMesh.tsx`
- Create: `apps/web/src/components/cad/viewport/CadCamera.tsx`
- Create: `apps/web/src/components/cad/viewport/ViewCube.tsx`
- Create: `apps/web/src/components/cad/viewport/SceneLighting.tsx`

**Step 1: Create ModelMesh**

Create `apps/web/src/components/cad/viewport/ModelMesh.tsx`:

```tsx
import { useMemo } from "react";
import * as THREE from "three";
import { useCadStore } from "@/stores/cad-store";

export function ModelMesh() {
  const meshData = useCadStore((s) => s.meshData);
  const viewMode = useCadStore((s) => s.viewMode);

  const geometries = useMemo(() => {
    return meshData.map((part) => {
      const geometry = new THREE.BufferGeometry();

      if (part.mesh.positions.length > 0) {
        geometry.setAttribute(
          "position",
          new THREE.Float32BufferAttribute(part.mesh.positions, 3),
        );
        geometry.setAttribute(
          "normal",
          new THREE.Float32BufferAttribute(part.mesh.normals, 3),
        );
        geometry.setIndex(
          new THREE.Uint32BufferAttribute(part.mesh.indices, 1),
        );
      }

      return { geometry, part };
    });
  }, [meshData]);

  return (
    <group>
      {geometries.map(({ geometry, part }) => (
        <group key={part.partId}>
          {/* Solid mesh */}
          {viewMode !== "wireframe" && (
            <mesh geometry={geometry}>
              <meshStandardMaterial
                color={new THREE.Color(part.color[0], part.color[1], part.color[2])}
                metalness={0.2}
                roughness={0.6}
                side={THREE.DoubleSide}
              />
            </mesh>
          )}
          {/* Wireframe edges */}
          {(viewMode === "wireframe" || viewMode === "hidden-line") &&
            part.edges.positions.length > 0 && (
              <lineSegments>
                <bufferGeometry>
                  <bufferAttribute
                    attach="attributes-position"
                    array={new Float32Array(part.edges.positions)}
                    count={part.edges.positions.length / 3}
                    itemSize={3}
                  />
                </bufferGeometry>
                <lineBasicMaterial color="#000000" linewidth={1} />
              </lineSegments>
            )}
        </group>
      ))}
    </group>
  );
}
```

**Step 2: Create SceneLighting**

Create `apps/web/src/components/cad/viewport/SceneLighting.tsx`:

```tsx
export function SceneLighting() {
  return (
    <>
      <ambientLight intensity={0.4} />
      <directionalLight position={[10, 10, 10]} intensity={0.8} castShadow />
      <directionalLight position={[-5, 5, -5]} intensity={0.3} />
      <directionalLight position={[0, -5, 0]} intensity={0.1} />
    </>
  );
}
```

**Step 3: Create CadCamera**

Create `apps/web/src/components/cad/viewport/CadCamera.tsx`:

```tsx
import { OrbitControls } from "@react-three/drei";

export function CadCamera() {
  return (
    <>
      <perspectiveCamera
        makeDefault
        position={[100, 100, 100]}
        fov={45}
        near={0.1}
        far={10000}
      />
      <OrbitControls
        makeDefault
        mouseButtons={{
          LEFT: undefined, // Reserved for selection
          MIDDLE: 0, // Orbit (THREE.MOUSE.ROTATE)
          RIGHT: 2, // Pan (THREE.MOUSE.PAN)
        }}
        enableDamping
        dampingFactor={0.1}
        minDistance={1}
        maxDistance={5000}
      />
    </>
  );
}
```

**Step 4: Create ViewCube**

Create `apps/web/src/components/cad/viewport/ViewCube.tsx`:

```tsx
import { GizmoHelper, GizmoViewcube } from "@react-three/drei";

export function ViewCube() {
  return (
    <GizmoHelper alignment="top-right" margin={[80, 80]}>
      <GizmoViewcube
        color="#ededed"
        textColor="#333"
        strokeColor="#999"
        hoverColor="#4dabf7"
      />
    </GizmoHelper>
  );
}
```

**Step 5: Create Canvas3D**

Create `apps/web/src/components/cad/viewport/Canvas3D.tsx`:

```tsx
import { Canvas } from "@react-three/fiber";
import { Grid } from "@react-three/drei";
import { ModelMesh } from "./ModelMesh";
import { CadCamera } from "./CadCamera";
import { ViewCube } from "./ViewCube";
import { SceneLighting } from "./SceneLighting";

export function Canvas3D() {
  return (
    <div className="relative h-full w-full">
      <Canvas
        gl={{ antialias: true, alpha: false }}
        camera={{ position: [100, 100, 100], fov: 45 }}
      >
        <color attach="background" args={["#1a1a2e"]} />
        <CadCamera />
        <SceneLighting />
        <ModelMesh />
        <Grid
          position={[0, 0, 0]}
          args={[1000, 1000]}
          cellSize={10}
          cellThickness={0.5}
          cellColor="#2a2a4a"
          sectionSize={50}
          sectionThickness={1}
          sectionColor="#3a3a5a"
          fadeDistance={500}
          infiniteGrid
        />
        <ViewCube />
      </Canvas>
    </div>
  );
}
```

**Step 6: Commit**

```bash
git add apps/web/src/components/cad/viewport/
git commit -m "feat: add 3D viewport with Canvas3D, ModelMesh, ViewCube, camera, lighting"
```

---

## Task 9: Layout Shell (Split Panels, Toolbar, Feature Tree, Tab Bar)

**Files:**
- Create: `apps/web/src/components/cad/layout/CadLayout.tsx`
- Create: `apps/web/src/components/cad/layout/TabBar.tsx`
- Create: `apps/web/src/components/cad/toolbar/MainToolbar.tsx`
- Create: `apps/web/src/components/cad/toolbar/ToolButton.tsx`
- Create: `apps/web/src/components/cad/feature-tree/FeatureTree.tsx`
- Create: `apps/web/src/components/cad/feature-tree/FeatureItem.tsx`
- Create: `apps/web/src/components/cad/panels/LeftPanel.tsx`

**Step 1: Create ToolButton**

Create `apps/web/src/components/cad/toolbar/ToolButton.tsx`:

```tsx
import { cn } from "@/lib/utils";
import { useCadStore } from "@/stores/cad-store";

interface ToolButtonProps {
  tool: string;
  label: string;
  shortcut?: string;
  icon: React.ReactNode;
}

export function ToolButton({ tool, label, shortcut, icon }: ToolButtonProps) {
  const activeTool = useCadStore((s) => s.activeTool);
  const setActiveTool = useCadStore((s) => s.setActiveTool);
  const isActive = activeTool === tool;

  return (
    <button
      onClick={() => setActiveTool(isActive ? null : (tool as any))}
      className={cn(
        "flex flex-col items-center gap-0.5 rounded px-2 py-1 text-xs transition-colors",
        isActive
          ? "bg-blue-600 text-white"
          : "text-zinc-300 hover:bg-zinc-700",
      )}
      title={shortcut ? `${label} (${shortcut})` : label}
    >
      {icon}
      <span className="text-[10px] leading-none">{label}</span>
    </button>
  );
}
```

**Step 2: Create MainToolbar**

Create `apps/web/src/components/cad/toolbar/MainToolbar.tsx`:

```tsx
import {
  Box,
  Circle,
  Cylinder,
  Minus,
  Redo,
  Undo,
  Search,
} from "lucide-react";
import { ToolButton } from "./ToolButton";

export function MainToolbar() {
  return (
    <div className="flex items-center gap-1 border-b border-zinc-700 bg-zinc-800 px-2 py-1">
      {/* Undo/Redo */}
      <button className="rounded p-1.5 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-200">
        <Undo size={16} />
      </button>
      <button className="rounded p-1.5 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-200">
        <Redo size={16} />
      </button>

      <div className="mx-1 h-6 w-px bg-zinc-600" />

      {/* Primitives (initial tools) */}
      <ToolButton
        tool="box"
        label="Box"
        icon={<Box size={16} />}
      />
      <ToolButton
        tool="cylinder"
        label="Cylinder"
        icon={<Cylinder size={16} />}
      />
      <ToolButton
        tool="sphere"
        label="Sphere"
        icon={<Circle size={16} />}
      />

      <div className="mx-1 h-6 w-px bg-zinc-600" />

      {/* Future: Sketch, Extrude, Revolve, etc. */}
      <ToolButton
        tool="extrude"
        label="Extrude"
        shortcut="Shift+E"
        icon={<Minus size={16} className="rotate-90" />}
      />

      <div className="flex-1" />

      {/* Search */}
      <button className="flex items-center gap-1 rounded border border-zinc-600 px-2 py-1 text-xs text-zinc-400 hover:bg-zinc-700">
        <Search size={14} />
        <span>Search tools...</span>
        <kbd className="ml-1 rounded bg-zinc-700 px-1 text-[10px]">Alt C</kbd>
      </button>
    </div>
  );
}
```

**Step 3: Create FeatureItem**

Create `apps/web/src/components/cad/feature-tree/FeatureItem.tsx`:

```tsx
import { cn } from "@/lib/utils";
import { useCadStore } from "@/stores/cad-store";
import { Box, Circle, Cylinder, Minus } from "lucide-react";

const featureIcons: Record<string, React.ReactNode> = {
  box: <Box size={14} />,
  cylinder: <Cylinder size={14} />,
  sphere: <Circle size={14} />,
  extrude: <Minus size={14} className="rotate-90" />,
};

interface FeatureItemProps {
  feature: {
    id: string;
    type: string;
    name: string;
    suppressed: boolean;
  };
}

export function FeatureItem({ feature }: FeatureItemProps) {
  const selectedFeatureId = useCadStore((s) => s.selectedFeatureId);
  const selectFeature = useCadStore((s) => s.selectFeature);
  const isSelected = selectedFeatureId === feature.id;

  return (
    <button
      onClick={() => selectFeature(isSelected ? null : feature.id)}
      className={cn(
        "flex w-full items-center gap-2 rounded px-2 py-1 text-left text-sm",
        isSelected ? "bg-blue-600/30 text-blue-300" : "text-zinc-300 hover:bg-zinc-700",
        feature.suppressed && "text-zinc-500 line-through",
      )}
    >
      <span className="text-zinc-400">
        {featureIcons[feature.type] ?? <Box size={14} />}
      </span>
      <span className="truncate">{feature.name}</span>
    </button>
  );
}
```

**Step 4: Create FeatureTree**

Create `apps/web/src/components/cad/feature-tree/FeatureTree.tsx`:

```tsx
import { useCadStore } from "@/stores/cad-store";
import { FeatureItem } from "./FeatureItem";

export function FeatureTree() {
  const features = useCadStore((s) => s.features);

  return (
    <div className="flex flex-col">
      <div className="flex items-center justify-between px-3 py-2">
        <span className="text-xs font-medium text-zinc-400">
          Features ({features.length})
        </span>
      </div>
      <div className="flex flex-col gap-0.5 px-1">
        {features.length === 0 && (
          <p className="px-2 py-4 text-center text-xs text-zinc-500">
            No features yet. Use the toolbar to add geometry.
          </p>
        )}
        {features.map((feature) => (
          <FeatureItem key={feature.id} feature={feature} />
        ))}
      </div>
    </div>
  );
}
```

**Step 5: Create LeftPanel**

Create `apps/web/src/components/cad/panels/LeftPanel.tsx`:

```tsx
import { FeatureTree } from "../feature-tree/FeatureTree";

export function LeftPanel() {
  return (
    <div className="flex h-full flex-col overflow-hidden border-r border-zinc-700 bg-zinc-800">
      <div className="border-b border-zinc-700 px-3 py-2">
        <span className="text-xs font-semibold text-zinc-300">
          Part Studio 1
        </span>
      </div>
      <div className="flex-1 overflow-y-auto">
        <FeatureTree />
      </div>
    </div>
  );
}
```

**Step 6: Create TabBar**

Create `apps/web/src/components/cad/layout/TabBar.tsx`:

```tsx
import { cn } from "@/lib/utils";
import { useCadStore } from "@/stores/cad-store";

export function TabBar() {
  const elements = useCadStore((s) => s.elements);
  const activeElementId = useCadStore((s) => s.activeElementId);
  const setActiveElement = useCadStore((s) => s.setActiveElement);

  return (
    <div className="flex h-8 items-center gap-px border-t border-zinc-700 bg-zinc-900 px-1">
      {elements.map((elem) => (
        <button
          key={elem.id}
          onClick={() => setActiveElement(elem.id)}
          className={cn(
            "flex items-center gap-1 rounded-t px-3 py-1 text-xs",
            activeElementId === elem.id
              ? "bg-zinc-800 text-zinc-200"
              : "text-zinc-500 hover:bg-zinc-800 hover:text-zinc-300",
          )}
        >
          {elem.name}
        </button>
      ))}
    </div>
  );
}
```

**Step 7: Create CadLayout**

Create `apps/web/src/components/cad/layout/CadLayout.tsx`:

```tsx
import { Allotment } from "allotment";
import "allotment/dist/style.css";
import { Canvas3D } from "../viewport/Canvas3D";
import { MainToolbar } from "../toolbar/MainToolbar";
import { LeftPanel } from "../panels/LeftPanel";
import { TabBar } from "./TabBar";

export function CadLayout() {
  return (
    <div className="flex h-full flex-col overflow-hidden bg-zinc-900">
      <Allotment>
        <Allotment.Pane minSize={200} preferredSize={240} maxSize={400}>
          <LeftPanel />
        </Allotment.Pane>
        <Allotment.Pane>
          <div className="flex h-full flex-col">
            <MainToolbar />
            <div className="flex-1">
              <Canvas3D />
            </div>
          </div>
        </Allotment.Pane>
      </Allotment>
      <TabBar />
    </div>
  );
}
```

**Step 8: Commit**

```bash
git add apps/web/src/components/cad/
git commit -m "feat: add CAD layout shell with split panels, toolbar, feature tree, tab bar"
```

---

## Task 10: Document Editor Route

**Files:**
- Create: `apps/web/src/routes/document.$id.tsx`
- Modify: `apps/web/src/routes/dashboard.tsx` (add document list + create button)

**Step 1: Create document editor route**

Create `apps/web/src/routes/document.$id.tsx`:

```tsx
import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { getUser } from "@/functions/get-user";
import { orpc } from "@/utils/orpc";
import { useCadStore } from "@/stores/cad-store";
import { CadLayout } from "@/components/cad/layout/CadLayout";

export const Route = createFileRoute("/document/$id")({
  component: DocumentEditor,
  beforeLoad: async () => {
    const session = await getUser();
    return { session };
  },
  loader: async ({ context }) => {
    if (!context.session) {
      throw redirect({ to: "/login" });
    }
  },
});

function DocumentEditor() {
  const { id } = Route.useParams();
  const setDocument = useCadStore((s) => s.setDocument);
  const setFeatures = useCadStore((s) => s.setFeatures);
  const setMeshData = useCadStore((s) => s.setMeshData);
  const activeElementId = useCadStore((s) => s.activeElementId);

  // Fetch document
  const docQuery = useQuery(
    orpc.cad.getDocument.queryOptions({ input: { documentId: id } }),
  );

  // Set document in store
  useEffect(() => {
    if (docQuery.data) {
      setDocument(docQuery.data.document, docQuery.data.elements);
    }
  }, [docQuery.data, setDocument]);

  // Fetch features for active element
  const featuresQuery = useQuery({
    ...orpc.cad.getFeatures.queryOptions({
      input: { elementId: activeElementId ?? "" },
    }),
    enabled: !!activeElementId,
  });

  useEffect(() => {
    if (featuresQuery.data) {
      setFeatures(featuresQuery.data as any);
    }
  }, [featuresQuery.data, setFeatures]);

  // Fetch tessellation for active element
  const tessQuery = useQuery({
    ...orpc.cad.getTessellation.queryOptions({
      input: { elementId: activeElementId ?? "" },
    }),
    enabled: !!activeElementId,
  });

  useEffect(() => {
    if (tessQuery.data) {
      setMeshData(tessQuery.data as any);
    }
  }, [tessQuery.data, setMeshData]);

  if (docQuery.isLoading) {
    return (
      <div className="flex h-full items-center justify-center bg-zinc-900 text-zinc-400">
        Loading document...
      </div>
    );
  }

  if (docQuery.error) {
    return (
      <div className="flex h-full items-center justify-center bg-zinc-900 text-red-400">
        Error loading document
      </div>
    );
  }

  return <CadLayout />;
}
```

**Step 2: Update dashboard with document list**

Replace `apps/web/src/routes/dashboard.tsx`:

```tsx
import { useMutation, useQuery } from "@tanstack/react-query";
import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import { getUser } from "@/functions/get-user";
import { orpc, queryClient } from "@/utils/orpc";
import { client } from "@/utils/orpc";

export const Route = createFileRoute("/dashboard")({
  component: RouteComponent,
  beforeLoad: async () => {
    const session = await getUser();
    return { session };
  },
  loader: async ({ context }) => {
    if (!context.session) {
      throw redirect({ to: "/login" });
    }
  },
});

function RouteComponent() {
  const { session } = Route.useRouteContext();
  const navigate = useNavigate();

  const documents = useQuery(orpc.cad.listDocuments.queryOptions());

  const createDoc = useMutation({
    mutationFn: () => client.cad.createDocument({ name: "Untitled document" }),
    onSuccess: (data) => {
      queryClient.invalidateQueries();
      navigate({ to: "/document/$id", params: { id: data.document.id } });
    },
  });

  return (
    <div className="mx-auto w-full max-w-4xl p-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-zinc-100">
          Welcome, {session?.user.name}
        </h1>
        <button
          onClick={() => createDoc.mutate()}
          disabled={createDoc.isPending}
          className="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          <Plus size={16} />
          New Document
        </button>
      </div>

      <div className="grid gap-4">
        {documents.data?.map((doc) => (
          <button
            key={doc.id}
            onClick={() =>
              navigate({ to: "/document/$id", params: { id: doc.id } })
            }
            className="flex items-center justify-between rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-3 text-left transition-colors hover:border-zinc-500"
          >
            <span className="font-medium text-zinc-200">{doc.name}</span>
            <span className="text-xs text-zinc-500">
              {new Date(doc.updatedAt).toLocaleDateString()}
            </span>
          </button>
        ))}
        {documents.data?.length === 0 && (
          <p className="py-8 text-center text-zinc-500">
            No documents yet. Create one to get started.
          </p>
        )}
      </div>
    </div>
  );
}
```

**Step 3: Commit**

```bash
git add apps/web/src/routes/document.\$id.tsx apps/web/src/routes/dashboard.tsx
git commit -m "feat: add document editor route and document list dashboard"
```

---

## Task 11: Wire Up Add Feature Flow

**Files:**
- Create: `apps/web/src/components/cad/toolbar/AddFeatureHandler.tsx`

This connects the toolbar buttons to oRPC calls that create features and update the mesh.

**Step 1: Create AddFeatureHandler**

Create `apps/web/src/components/cad/toolbar/AddFeatureHandler.tsx`:

```tsx
import { useEffect } from "react";
import { useMutation } from "@tanstack/react-query";
import { useCadStore } from "@/stores/cad-store";
import { client } from "@/utils/orpc";
import { queryClient } from "@/utils/orpc";

export function AddFeatureHandler() {
  const activeTool = useCadStore((s) => s.activeTool);
  const activeElementId = useCadStore((s) => s.activeElementId);
  const features = useCadStore((s) => s.features);
  const setFeatures = useCadStore((s) => s.setFeatures);
  const setMeshData = useCadStore((s) => s.setMeshData);
  const resetTool = useCadStore((s) => s.resetTool);

  const addFeature = useMutation({
    mutationFn: (input: {
      elementId: string;
      type: string;
      name: string;
      parameters: Record<string, unknown>;
    }) => client.cad.addFeature(input),
    onSuccess: (data) => {
      setFeatures([...features, data.feature as any]);
      setMeshData(data.tessellation as any);
      queryClient.invalidateQueries();
      resetTool();
    },
  });

  // For primitives, add immediately on tool click
  useEffect(() => {
    if (!activeTool || !activeElementId) return;

    const primitiveDefaults: Record<string, Record<string, unknown>> = {
      box: { width: 50, height: 50, depth: 50 },
      cylinder: { radius: 25, height: 50 },
      sphere: { radius: 25 },
    };

    const params = primitiveDefaults[activeTool];
    if (params) {
      addFeature.mutate({
        elementId: activeElementId,
        type: activeTool,
        name: `${activeTool.charAt(0).toUpperCase() + activeTool.slice(1)} ${features.length + 1}`,
        parameters: params,
      });
    }
  }, [activeTool]);

  return null; // Purely logic, no render
}
```

**Step 2: Wire into CadLayout**

Add to `apps/web/src/components/cad/layout/CadLayout.tsx` - import and render `<AddFeatureHandler />` inside the layout:

After the imports, add:
```tsx
import { AddFeatureHandler } from "../toolbar/AddFeatureHandler";
```

Inside the CadLayout return, add `<AddFeatureHandler />` as the first child of the outer div.

**Step 3: Commit**

```bash
git add apps/web/src/components/cad/
git commit -m "feat: wire up add feature flow connecting toolbar to oRPC and mesh updates"
```

---

## Task 12: Smoke Test - Full Pipeline Verification

**Step 1: Push database schema**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD && bun run db:push
```

**Step 2: Start dev server**

```bash
cd /Users/ezrasperodev/Documents/GitHub/blockCAD && bun run dev
```

**Step 3: Manual verification checklist**

1. Navigate to `http://localhost:3001/login` - sign in
2. Navigate to `/dashboard` - see document list
3. Click "New Document" - creates document, redirects to `/document/{id}`
4. See the split-panel layout: left panel (empty feature tree) + 3D viewport (grid + view cube)
5. Click "Box" in toolbar - feature appears in tree, 3D box renders in viewport
6. Click "Cylinder" - second feature appears, cylinder renders
7. Orbit with middle mouse, pan with right mouse
8. View cube responds to camera changes

**Step 4: Commit any fixes needed**

```bash
git add -A
git commit -m "fix: resolve smoke test issues"
```

---

## Future Tasks (post-MVP)

These are not part of the initial implementation but are documented for planning:

- **Task 13**: Sketch system (2D drawing on planes, constraint solving)
- **Task 14**: Extrude from sketch profiles (wiring sketch → extrude)
- **Task 15**: Revolve, Sweep, Loft operations
- **Task 16**: Fillet/Chamfer edge selection in viewport
- **Task 17**: Boolean operations between parts
- **Task 18**: Shell, Pattern, Mirror, Draft
- **Task 19**: STEP/STL import and export
- **Task 20**: Feature reordering, rollback bar, suppress/unsuppress
- **Task 21**: Right panel (appearances, measure, mass properties)
- **Task 22**: Keyboard shortcuts
- **Task 23**: Selection highlighting (face/edge/vertex picking via raycasting)
