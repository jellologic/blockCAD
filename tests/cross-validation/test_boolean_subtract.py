"""Cross-validate boolean subtract geometry against trimesh (independent mesh library).

Geometry: 10x5x7 box (A) minus 4x3x10 box offset to (3,1,-1) (B).
Overlap region: x:[3,7], y:[1,4], z:[0,7] = 4*3*7 = 84.
Expected result volume: 350 - 84 = 266.
Bounding box should still be 10x5x7 (the subtracted notch is interior).
"""

import pytest


@pytest.mark.xfail(reason="Boolean subtract does not yet produce watertight mesh")
def test_boolean_subtract_is_watertight(boolean_subtract):
    mesh, _ = boolean_subtract
    assert mesh.is_watertight, "Boolean subtract mesh should be watertight"


def test_boolean_subtract_volume(boolean_subtract):
    mesh, _ = boolean_subtract
    # 10*5*7 = 350, overlap = 4*3*7 = 84, result = 266
    assert abs(mesh.volume - 266.0) < 5.0, (
        f"Boolean subtract volume should be ~266, got {mesh.volume}"
    )


def test_boolean_subtract_bounding_box(boolean_subtract):
    mesh, _ = boolean_subtract
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_boolean_subtract_matches_kernel_volume(boolean_subtract):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = boolean_subtract
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
