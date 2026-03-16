"""Generate Parasolid XT text format (.x_t) files for basic geometry.

The Parasolid XT text format is a well-documented, human-readable format
that SolidWorks (and most CAD tools) can import directly. It is far simpler
to generate than the binary XT format used inside SLDPRT files.

This module generates valid .x_t files for basic solid bodies:
  - Box (cuboid)
  - Cylinder
  - More shapes can be added

Coordinate system: all values in **meters** (SI units).

Usage::

    from swparse.parasolid.writer import write_box_xt

    xt_data = write_box_xt(
        dx=0.0254, dy=0.0254, dz=0.0254,  # 1x1x1 inch
        origin=(0, 0, 0),
    )
    Path("box.x_t").write_text(xt_data)
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Tuple

Vec3 = Tuple[float, float, float]

PARASOLID_VERSION = "2800174"
SCHEMA = f"SCH_{PARASOLID_VERSION}_28002_13006"


class _IdAllocator:
    """Allocate sequential entity IDs."""

    def __init__(self, start: int = 1):
        self._next = start

    def alloc(self) -> int:
        eid = self._next
        self._next += 1
        return eid

    @property
    def count(self) -> int:
        return self._next - 1


def _fmt(v: float) -> str:
    """Format a float for Parasolid XT text (full precision)."""
    if v == 0.0:
        return "0"
    if v == int(v) and abs(v) < 1e12:
        return str(int(v))
    return f"{v:.17g}"


def _vec(v: Vec3) -> str:
    return f"{_fmt(v[0])} {_fmt(v[1])} {_fmt(v[2])}"


def write_box_xt(
    dx: float,
    dy: float,
    dz: float,
    origin: Vec3 = (0.0, 0.0, 0.0),
) -> str:
    """Generate a Parasolid XT text file for a box (cuboid).

    Args:
        dx, dy, dz: Box dimensions in meters.
        origin: Corner position (min-x, min-y, min-z).

    Returns:
        Complete .x_t file content as a string.
    """
    ox, oy, oz = origin
    # 8 corner vertices
    v = [
        (ox,      oy,      oz),       # 0: min corner
        (ox + dx, oy,      oz),       # 1
        (ox + dx, oy + dy, oz),       # 2
        (ox,      oy + dy, oz),       # 3
        (ox,      oy,      oz + dz),  # 4
        (ox + dx, oy,      oz + dz),  # 5
        (ox + dx, oy + dy, oz + dz),  # 6
        (ox,      oy + dy, oz + dz),  # 7
    ]

    ids = _IdAllocator()

    # Allocate all entity IDs upfront
    body_id = ids.alloc()       # 1
    region_id = ids.alloc()     # 2
    shell_id = ids.alloc()      # 3

    # 6 faces: -Z(bot), +Z(top), -X(left), +X(right), -Y(front), +Y(back)
    face_ids = [ids.alloc() for _ in range(6)]    # 4-9
    loop_ids = [ids.alloc() for _ in range(6)]     # 10-15

    # 12 edges (4 bottom, 4 top, 4 vertical)
    # Bottom: 0-1, 1-2, 2-3, 3-0
    # Top:    4-5, 5-6, 6-7, 7-4
    # Vert:   0-4, 1-5, 2-6, 3-7
    edge_ids = [ids.alloc() for _ in range(12)]    # 16-27

    # 8 vertices
    vert_ids = [ids.alloc() for _ in range(8)]     # 28-35

    # 6 plane surfaces
    surf_ids = [ids.alloc() for _ in range(6)]     # 36-41

    # 12 straight curves (one per edge)
    curve_ids = [ids.alloc() for _ in range(12)]   # 42-53

    # 8 points
    point_ids = [ids.alloc() for _ in range(8)]    # 54-61

    # 24 fins (4 per face, 2 per edge)
    fin_ids = [ids.alloc() for _ in range(24)]     # 62-85

    total = ids.count

    lines: list[str] = []
    W = lines.append

    # ── File header ──────────────────────────────────────────────
    W("**ABCDEFsp")
    W("**PART FILE")
    W(f"        {total}")
    W(f"{SCHEMA} 200")
    W("")

    # ── BODY ─────────────────────────────────────────────────────
    W(f"*{body_id} BODY $-1 $-1 $-1")
    W(f" *{region_id} e1 $-1 $-1 $-1 T @7 unknown T F F F F")
    W("")

    # ── REGION ───────────────────────────────────────────────────
    W(f"*{region_id} REGION $-1 $-1")
    W(f" *{body_id} $ *{shell_id}")
    W("")

    # ── SHELL ────────────────────────────────────────────────────
    W(f"*{shell_id} SHELL $-1 $-1")
    W(f" *{region_id} $ $")
    W(f" *{face_ids[0]} F")
    W("")

    # ── Face definitions ─────────────────────────────────────────
    # Face normals and positions:
    face_defs = [
        # (surface_normal, surface_point, sense)
        ((0, 0, -1), (ox, oy, oz),          "reversed"),  # -Z bottom
        ((0, 0,  1), (ox, oy, oz + dz),     "forward"),   # +Z top
        ((-1, 0, 0), (ox, oy, oz),          "reversed"),  # -X left
        ((1, 0, 0),  (ox + dx, oy, oz),     "forward"),   # +X right
        ((0, -1, 0), (ox, oy, oz),          "reversed"),  # -Y front
        ((0, 1, 0),  (ox, oy + dy, oz),     "forward"),   # +Y back
    ]

    # Edge connectivity per face (indices into edge_ids, and orientation)
    # Bottom face (-Z): edges 0,1,2,3 (bottom square)
    # Top face (+Z): edges 4,5,6,7 (top square)
    # Left face (-X): edges 3,8,7,11 (left side)
    # Right face (+X): edges 1,10,5,9 (right side)
    # Front face (-Y): edges 0,9,4,8 (front side)
    # Back face (+Y): edges 2,11,6,10 (back side)

    # Bottom edges: 0=v0-v1, 1=v1-v2, 2=v2-v3, 3=v3-v0
    # Top edges:    4=v4-v5, 5=v5-v6, 6=v6-v7, 7=v7-v4
    # Vert edges:   8=v0-v4, 9=v1-v5, 10=v2-v6, 11=v3-v7

    face_edges = [
        [0, 1, 2, 3],      # bottom
        [4, 5, 6, 7],      # top
        [3, 11, 7, 8],     # left
        [1, 9, 5, 10],     # right
        [0, 8, 4, 9],      # front
        [2, 10, 6, 11],    # back
    ]

    for fi in range(6):
        fid = face_ids[fi]
        next_fid = face_ids[(fi + 1) % 6] if fi < 5 else face_ids[0]
        prev_fid = face_ids[(fi - 1) % 6] if fi > 0 else face_ids[5]
        lid = loop_ids[fi]
        sid = surf_ids[fi]
        normal, point, sense = face_defs[fi]

        W(f"*{fid} FACE $-1 $-1")
        W(f" *{shell_id} $-1")
        W(f" *{next_fid} *{lid} $-1 {sense} single")
        W(f" $ $ $ $ $")
        W(f" *{sid}")
        W("")

    # ── Loop + Fin definitions ───────────────────────────────────
    for fi in range(6):
        lid = loop_ids[fi]
        fid = face_ids[fi]
        edge_indices = face_edges[fi]
        fin_base = fi * 4

        W(f"*{lid} LOOP $-1 $-1")
        W(f" *{fid} $-1")
        W(f" *{fin_ids[fin_base]} F")
        W("")

        # 4 fins per loop
        for j in range(4):
            fin_id = fin_ids[fin_base + j]
            next_fin = fin_ids[fin_base + (j + 1) % 4]
            prev_fin = fin_ids[fin_base + (j - 1) % 4]
            ei = edge_indices[j]
            eid = edge_ids[ei]

            W(f"*{fin_id} FIN $-1 $-1")
            W(f" *{lid} *{next_fin} *{prev_fin}")
            W(f" *{eid} $-1 forward")
            W("")

    # ── Edge definitions ─────────────────────────────────────────
    # Each edge connects two vertices and has a curve
    edge_verts = [
        (0, 1), (1, 2), (2, 3), (3, 0),  # bottom
        (4, 5), (5, 6), (6, 7), (7, 4),  # top
        (0, 4), (1, 5), (2, 6), (3, 7),  # vertical
    ]

    for ei in range(12):
        eid = edge_ids[ei]
        vi_start, vi_end = edge_verts[ei]
        vid_start = vert_ids[vi_start]
        vid_end = vert_ids[vi_end]
        cid = curve_ids[ei]

        W(f"*{eid} EDGE $-1 $-1")
        W(f" *{vid_start} $ *{vid_end}")
        W(f" $-1 forward")
        W(f" *{cid}")
        W("")

    # ── Vertex definitions ───────────────────────────────────────
    for vi in range(8):
        vid = vert_ids[vi]
        pid = point_ids[vi]

        W(f"*{vid} VERTEX $-1 $-1")
        W(f" $-1")
        W(f" *{pid}")
        W("")

    # ── Plane surface definitions ────────────────────────────────
    for fi in range(6):
        sid = surf_ids[fi]
        normal, point, _ = face_defs[fi]

        # Plane needs: position, normal, and two tangent directions
        # We'll use standard axis-aligned tangents
        nx, ny, nz = normal
        if abs(nz) > 0.5:
            u = (1, 0, 0)
        elif abs(ny) > 0.5:
            u = (0, 0, 1)
        else:
            u = (0, 1, 0)

        W(f"*{sid} PLANE-SURFACE $-1 $-1")
        W(f" {_vec(point)} {_vec(normal)} {_vec(u)}")
        W(f" forward_v I I I I")
        W("")

    # ── Straight curve definitions ───────────────────────────────
    for ei in range(12):
        cid = curve_ids[ei]
        vi_start, vi_end = edge_verts[ei]
        p1 = v[vi_start]
        p2 = v[vi_end]
        # Direction vector
        dx_c = p2[0] - p1[0]
        dy_c = p2[1] - p1[1]
        dz_c = p2[2] - p1[2]
        length = math.sqrt(dx_c**2 + dy_c**2 + dz_c**2)
        if length > 0:
            dx_c, dy_c, dz_c = dx_c/length, dy_c/length, dz_c/length
        else:
            dx_c, dy_c, dz_c = 1, 0, 0

        W(f"*{cid} STRAIGHT-CURVE $-1 $-1")
        W(f" {_vec(p1)} {_fmt(dx_c)} {_fmt(dy_c)} {_fmt(dz_c)}")
        W(f" I I")
        W("")

    # ── Point definitions ────────────────────────────────────────
    for vi in range(8):
        pid = point_ids[vi]
        W(f"*{pid} POINT $-1 $-1")
        W(f" {_vec(v[vi])}")
        W("")

    # ── File trailer ─────────────────────────────────────────────
    W("**END_OF_PART")
    W("")

    return "\n".join(lines)


def write_cylinder_xt(
    radius: float,
    height: float,
    origin: Vec3 = (0.0, 0.0, 0.0),
    axis: Vec3 = (0.0, 0.0, 1.0),
) -> str:
    """Generate a Parasolid XT text file for a cylinder.

    Args:
        radius: Cylinder radius in meters.
        height: Cylinder height in meters.
        origin: Base center position.
        axis: Cylinder axis direction (default: +Z).
    """
    ox, oy, oz = origin
    ax, ay, az = axis

    ids = _IdAllocator()
    body_id = ids.alloc()
    region_id = ids.alloc()
    shell_id = ids.alloc()

    # 3 faces: bottom, top, side
    face_bot = ids.alloc()
    face_top = ids.alloc()
    face_side = ids.alloc()

    # 3 loops (one per face)
    loop_bot = ids.alloc()
    loop_top = ids.alloc()
    loop_side = ids.alloc()

    # 2 edges (bottom circle, top circle)
    edge_bot = ids.alloc()
    edge_top = ids.alloc()

    # 2 vertices (one per circle, at the same angular position)
    vert_bot = ids.alloc()
    vert_top = ids.alloc()

    # Surfaces
    surf_bot = ids.alloc()  # plane
    surf_top = ids.alloc()  # plane
    surf_side = ids.alloc()  # cylinder

    # Curves
    curve_bot = ids.alloc()  # circle
    curve_top = ids.alloc()  # circle

    # Points
    pt_bot = ids.alloc()
    pt_top = ids.alloc()

    # Fins (2 per face for the side, 1 per face for top/bottom)
    fin_bot = ids.alloc()
    fin_top = ids.alloc()
    fin_side_1 = ids.alloc()
    fin_side_2 = ids.alloc()

    total = ids.count

    lines: list[str] = []
    W = lines.append

    top_center = (ox, oy, oz + height)
    vert_bot_pos = (ox + radius, oy, oz)
    vert_top_pos = (ox + radius, oy, oz + height)

    W("**ABCDEFsp")
    W("**PART FILE")
    W(f"        {total}")
    W(f"{SCHEMA} 200")
    W("")

    W(f"*{body_id} BODY $-1 $-1 $-1")
    W(f" *{region_id} e1 $-1 $-1 $-1 T @7 unknown T F F F F")
    W("")

    W(f"*{region_id} REGION $-1 $-1")
    W(f" *{body_id} $ *{shell_id}")
    W("")

    W(f"*{shell_id} SHELL $-1 $-1")
    W(f" *{region_id} $ $")
    W(f" *{face_bot} F")
    W("")

    # Bottom face
    W(f"*{face_bot} FACE $-1 $-1")
    W(f" *{shell_id} $-1 *{face_top} *{loop_bot} $-1 reversed single $ $ $ $ $ *{surf_bot}")
    W("")

    # Top face
    W(f"*{face_top} FACE $-1 $-1")
    W(f" *{shell_id} $-1 *{face_side} *{loop_top} $-1 forward single $ $ $ $ $ *{surf_top}")
    W("")

    # Side face
    W(f"*{face_side} FACE $-1 $-1")
    W(f" *{shell_id} $-1 *{face_bot} *{loop_side} $-1 forward single $ $ $ $ $ *{surf_side}")
    W("")

    # Loops
    W(f"*{loop_bot} LOOP $-1 $-1 *{face_bot} $-1 *{fin_bot} F")
    W("")
    W(f"*{loop_top} LOOP $-1 $-1 *{face_top} $-1 *{fin_top} F")
    W("")
    W(f"*{loop_side} LOOP $-1 $-1 *{face_side} $-1 *{fin_side_1} F")
    W("")

    # Fins
    W(f"*{fin_bot} FIN $-1 $-1 *{loop_bot} *{fin_bot} *{fin_bot} *{edge_bot} $-1 forward")
    W("")
    W(f"*{fin_top} FIN $-1 $-1 *{loop_top} *{fin_top} *{fin_top} *{edge_top} $-1 forward")
    W("")
    W(f"*{fin_side_1} FIN $-1 $-1 *{loop_side} *{fin_side_2} *{fin_side_2} *{edge_bot} $-1 reversed")
    W("")
    W(f"*{fin_side_2} FIN $-1 $-1 *{loop_side} *{fin_side_1} *{fin_side_1} *{edge_top} $-1 reversed")
    W("")

    # Edges
    W(f"*{edge_bot} EDGE $-1 $-1 *{vert_bot} $ *{vert_bot} $-1 forward *{curve_bot}")
    W("")
    W(f"*{edge_top} EDGE $-1 $-1 *{vert_top} $ *{vert_top} $-1 forward *{curve_top}")
    W("")

    # Vertices
    W(f"*{vert_bot} VERTEX $-1 $-1 $-1 *{pt_bot}")
    W("")
    W(f"*{vert_top} VERTEX $-1 $-1 $-1 *{pt_top}")
    W("")

    # Surfaces
    W(f"*{surf_bot} PLANE-SURFACE $-1 $-1 {_vec(origin)} 0 0 -1 1 0 0 forward_v I I I I")
    W("")
    W(f"*{surf_top} PLANE-SURFACE $-1 $-1 {_vec(top_center)} 0 0 1 1 0 0 forward_v I I I I")
    W("")
    W(f"*{surf_side} CONE-SURFACE $-1 $-1 {_vec(origin)} 0 0 1 1 0 0 {_fmt(radius)} 1 I I I I")
    W("")

    # Curves (circles)
    W(f"*{curve_bot} CIRCLE-CURVE $-1 $-1 {_vec(origin)} 0 0 1 {_fmt(radius)} 1 0 0 I I")
    W("")
    W(f"*{curve_top} CIRCLE-CURVE $-1 $-1 {_vec(top_center)} 0 0 1 {_fmt(radius)} 1 0 0 I I")
    W("")

    # Points
    W(f"*{pt_bot} POINT $-1 $-1 {_vec(vert_bot_pos)}")
    W("")
    W(f"*{pt_top} POINT $-1 $-1 {_vec(vert_top_pos)}")
    W("")

    W("**END_OF_PART")
    W("")

    return "\n".join(lines)
