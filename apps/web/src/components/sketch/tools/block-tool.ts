import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/**
 * Block tool modes:
 * - "create": Select entities to group into a named block.
 * - "insert": Place an existing block definition at a click position.
 * - "explode": Ungroup a block instance back to individual entities.
 *
 * For now, we implement create-block and insert-block via click interactions.
 */

interface BlockDefinition {
  id: string;
  name: string;
  insertionPoint: SketchPoint2D;
  entityIds: string[];
}

// In-memory block definitions registry (shared across clicks in the same session)
const blockDefinitions: BlockDefinition[] = [];

let blockMode: "create" | "insert" | "explode" = "create";
let selectedEntitiesForBlock: string[] = [];
let blockCounter = 0;

export function setBlockMode(mode: "create" | "insert" | "explode"): void {
  blockMode = mode;
  selectedEntitiesForBlock = [];
}

export function getBlockDefinitions(): BlockDefinition[] {
  return blockDefinitions;
}

/**
 * Create a block from the currently selected entities.
 * Call this when the user has selected entities and confirms block creation.
 */
export function createBlockFromSelection(name: string, anchorPos: SketchPoint2D): BlockDefinition | null {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return null;

  // Use currently selected entities (tracked externally by selection system)
  if (selectedEntitiesForBlock.length === 0) return null;

  const blockId = `block-${blockCounter++}`;
  const block: BlockDefinition = {
    id: blockId,
    name: name || `Block ${blockCounter}`,
    insertionPoint: anchorPos,
    entityIds: [...selectedEntitiesForBlock],
  };

  blockDefinitions.push(block);
  selectedEntitiesForBlock = [];
  return block;
}

/**
 * Handle block tool click.
 * In "create" mode: collect clicked entity (point/line/circle) IDs.
 * In "insert" mode: place block at clicked position.
 * In "explode" mode: find and remove block grouping at click location.
 */
export function handleBlockClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  if (blockMode === "create") {
    // Find nearest entity to click and add to selection
    const nearestId = findNearestEntityId(clickPos, session.entities);
    if (nearestId && !selectedEntitiesForBlock.includes(nearestId)) {
      selectedEntitiesForBlock.push(nearestId);
    }
  } else if (blockMode === "insert") {
    // Insert the most recently created block at click position
    if (blockDefinitions.length === 0) return;
    const block = blockDefinitions[blockDefinitions.length - 1];
    insertBlockInstance(block, clickPos);
  }
}

/**
 * Insert a block instance by duplicating its entities at the given position.
 */
function insertBlockInstance(block: BlockDefinition, position: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const dx = position.x - block.insertionPoint.x;
  const dy = position.y - block.insertionPoint.y;

  store.beginUndoBatch();

  // Map from original entity IDs to new entity IDs
  const idMap = new Map<string, string>();

  // First pass: duplicate points with offset
  for (const entityId of block.entityIds) {
    const entity = session.entities.find(e => e.id === entityId);
    if (!entity) continue;

    if (entity.type === "point") {
      const newId = store.genSketchEntityId();
      idMap.set(entityId, newId);
      store.addSketchEntity({
        type: "point",
        id: newId,
        position: { x: entity.position.x + dx, y: entity.position.y + dy },
      });
    }
  }

  // Second pass: duplicate lines/circles/arcs with remapped IDs
  for (const entityId of block.entityIds) {
    const entity = session.entities.find(e => e.id === entityId);
    if (!entity) continue;

    if (entity.type === "line") {
      const newStartId = idMap.get(entity.startId) ?? entity.startId;
      const newEndId = idMap.get(entity.endId) ?? entity.endId;
      const newId = store.genSketchEntityId();
      store.addSketchEntity({ type: "line", id: newId, startId: newStartId, endId: newEndId });
    } else if (entity.type === "circle") {
      const newCenterId = idMap.get(entity.centerId) ?? entity.centerId;
      const newId = store.genSketchEntityId();
      store.addSketchEntity({ type: "circle", id: newId, centerId: newCenterId, radius: entity.radius });
    } else if (entity.type === "arc") {
      const newCenterId = idMap.get(entity.centerId) ?? entity.centerId;
      const newStartId = idMap.get(entity.startId) ?? entity.startId;
      const newEndId = idMap.get(entity.endId) ?? entity.endId;
      const newId = store.genSketchEntityId();
      store.addSketchEntity({ type: "arc", id: newId, centerId: newCenterId, startId: newStartId, endId: newEndId, radius: entity.radius });
    }
  }

  store.endUndoBatch();
}

/**
 * Find the nearest entity ID to a click position.
 */
function findNearestEntityId(
  pos: SketchPoint2D,
  entities: import("@blockCAD/kernel").SketchEntity2D[],
): string | null {
  let nearestId: string | null = null;
  let nearestDist = Infinity;

  for (const entity of entities) {
    if (entity.type === "point") {
      const dist = Math.sqrt(
        (pos.x - entity.position.x) ** 2 + (pos.y - entity.position.y) ** 2
      );
      if (dist < nearestDist) {
        nearestDist = dist;
        nearestId = entity.id;
      }
    }
  }

  return nearestDist < 2.0 ? nearestId : null;
}
