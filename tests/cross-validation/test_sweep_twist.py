"""Cross-validate sweep-with-twist geometry against trimesh.

Geometry: 4x4 square profile swept 10 units along Z with 90-degree twist.
Twisting a constant cross-section does not change volume, so expected ~160.
The bounding box will be slightly larger than the untwisted case because
the square corners trace a diagonal path.
"""

import math


def test_sweep_twisted_is_watertight(sweep_twisted):
    mesh, _ = sweep_twisted
    assert mesh.is_watertight, "Twisted sweep mesh should be watertight (closed solid)"


def test_sweep_twisted_volume(sweep_twisted):
    mesh, _ = sweep_twisted
    # 4x4 square swept 10 units = 160 cubic units (twist preserves volume)
    assert abs(mesh.volume - 160.0) < 8.0, f"Twisted sweep volume should be ~160, got {mesh.volume}"


def test_sweep_twisted_bounding_box(sweep_twisted):
    mesh, _ = sweep_twisted
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Z extents: 0..10 same as untwisted
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"
    # X/Y extents: twist makes corners sweep outward.
    # The 4x4 square has corners at distance sqrt(8) ~ 2.83 from center.
    # With 90-degree twist the max extent is sqrt(8) ~ 2.83 (vs 2.0 untwisted).
    diag = math.sqrt(8.0)
    assert bbox_max[0] >= 1.9, f"Bbox max x should be >= 1.9, got {bbox_max[0]}"
    assert bbox_max[0] <= diag + 0.5, f"Bbox max x should be <= {diag + 0.5}, got {bbox_max[0]}"
    assert bbox_max[1] >= 1.9, f"Bbox max y should be >= 1.9, got {bbox_max[1]}"
    assert bbox_max[1] <= diag + 0.5, f"Bbox max y should be <= {diag + 0.5}, got {bbox_max[1]}"


def test_sweep_twisted_matches_kernel_volume(sweep_twisted):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = sweep_twisted
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
