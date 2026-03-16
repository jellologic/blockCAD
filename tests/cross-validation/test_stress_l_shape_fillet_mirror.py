"""Stress test: L-shaped extrude (10x10 minus 5x5 corner, 5mm tall)
-> Fillet(edge 0, r=0.3) -> Mirror(YZ plane at x=0).

Non-convex profile + finish operation (fillet) + transform (mirror).
The L-shape has area = 10*10 - 5*5 = 75, extruded 5mm => volume = 375.
Fillet removes a small amount; mirror doubles the body.
Expected mirrored volume: ~750 minus small fillet removal.
Bounding box after mirror: x in [-10, 10], y in [0, 10], z in [0, 5].
"""

import pytest


def test_watertight(stress_l_shape_fillet_mirror):
    mesh, _ = stress_l_shape_fillet_mirror
    assert mesh.is_watertight, "L-shape fillet+mirror mesh should be watertight"


def test_volume(stress_l_shape_fillet_mirror):
    mesh, _ = stress_l_shape_fillet_mirror
    # Mirror doubles the filleted L-shape (~375 each, minus tiny fillet removal)
    assert 700.0 < mesh.volume < 760.0, (
        f"L-shape fillet+mirror volume should be ~750, got {mesh.volume}"
    )


def test_bounding_box(stress_l_shape_fillet_mirror):
    mesh, _ = stress_l_shape_fillet_mirror
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # After mirror across YZ at x=0, X spans [-10, 10]
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y spans [0, 10] (L-shape profile)
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 10.0) < 0.5, f"Bbox max y should be ~10, got {bbox_max[1]}"
    # Z spans [0, 5]
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 5.0) < 0.5, f"Bbox max z should be ~5, got {bbox_max[2]}"


def test_kernel_volume_match(stress_l_shape_fillet_mirror):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_l_shape_fillet_mirror
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
