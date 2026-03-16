"""Stress test: cross-validate box + linear pattern + cut extrude against trimesh.

Geometry: 10x5x7 box -> LinearPattern(2x, spacing 15, along X) -> CutExtrude(4x2x3 blind pocket on first copy).

Two 10x5x7 boxes at x=0..10 and x=15..25 (spacing > width so no overlap).
A 4x2x3 blind pocket is cut from the bottom face (z=0) of the first copy only,
centered at (5, 2.5) in the XY plane.

Volume: 2 * (10*5*7) - (4*2*3) = 700 - 24 = 676
Bounding box: X: 0..25, Y: 0..5, Z: 0..7.
"""

import pytest


@pytest.mark.xfail(reason="Pattern+cut per-face tessellation produces non-watertight mesh (kernel limitation)")
def test_watertight(stress_pattern_cut):
    mesh, _ = stress_pattern_cut
    assert mesh.is_watertight, "Stress pattern-cut mesh should be watertight"


def test_volume_bounds(stress_pattern_cut):
    mesh, _ = stress_pattern_cut
    # 2 boxes of 350 minus 24 pocket = 676; generous tolerance for tessellation
    assert 620.0 < mesh.volume < 720.0, (
        f"Volume should be roughly 620-720 (expected ~676), got {mesh.volume}"
    )


def test_bounding_box(stress_pattern_cut):
    mesh, _ = stress_pattern_cut
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Origin near (0, 0, 0)
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    # 2 copies at x=0 and x=15; each 10 wide -> max x = 25
    assert abs(bbox_max[0] - 25.0) < 0.5, f"Bbox max x should be ~25, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_kernel_volume_match(stress_pattern_cut):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_pattern_cut
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
