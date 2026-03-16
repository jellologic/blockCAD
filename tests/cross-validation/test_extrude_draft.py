"""Cross-validate extrude-with-draft geometry against trimesh.

Geometry: 10x5 rectangle extruded 7mm with a 5-degree draft angle on all sides.
The draft makes the top face smaller than the bottom, so the volume is less than
the non-draft box (350 mm^3).
"""

import numpy as np


def test_extrude_draft_is_watertight(extrude_draft):
    mesh, _ = extrude_draft
    assert mesh.is_watertight, "Extrude-draft mesh should be watertight (closed solid)"


def test_extrude_draft_volume_less_than_box(extrude_draft):
    mesh, _ = extrude_draft
    assert mesh.volume < 350.0, (
        f"Extrude-draft volume ({mesh.volume}) should be less than 350 (non-draft box)"
    )
    assert mesh.volume > 200.0, (
        f"Extrude-draft volume ({mesh.volume}) should be > 200"
    )


def test_extrude_draft_top_smaller_than_bottom(extrude_draft):
    """The top face (at z~7) should be smaller than the bottom face (at z~0)."""
    mesh, _ = extrude_draft
    verts = mesh.vertices

    # Vertices near the bottom (z ~ 0) and top (z ~ 7)
    z_min = verts[:, 2].min()
    z_max = verts[:, 2].max()
    tol = 0.5

    bottom_verts = verts[np.abs(verts[:, 2] - z_min) < tol]
    top_verts = verts[np.abs(verts[:, 2] - z_max) < tol]

    # Bounding box of bottom vs top in XY
    bottom_x_range = bottom_verts[:, 0].max() - bottom_verts[:, 0].min()
    bottom_y_range = bottom_verts[:, 1].max() - bottom_verts[:, 1].min()
    top_x_range = top_verts[:, 0].max() - top_verts[:, 0].min()
    top_y_range = top_verts[:, 1].max() - top_verts[:, 1].min()

    assert top_x_range < bottom_x_range, (
        f"Top X range ({top_x_range}) should be smaller than bottom ({bottom_x_range})"
    )
    assert top_y_range < bottom_y_range, (
        f"Top Y range ({top_y_range}) should be smaller than bottom ({bottom_y_range})"
    )


def test_extrude_draft_matches_kernel_volume(extrude_draft):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = extrude_draft
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
