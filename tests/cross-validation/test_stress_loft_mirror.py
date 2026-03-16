"""Cross-validate stress test: loft + mirror geometry against trimesh.

Geometry: Loft between a 4x4 square at z=0 and a 2x2 square at z=10,
then mirrored across the XY plane at z=0.

This produces two frustums: one from z=0 to z=10, one from z=0 to z=-10.
Expected volume: 2 * h/3 * (A1 + A2 + sqrt(A1*A2))
  = 2 * 10/3 * (16 + 4 + sqrt(64)) = 2 * 93.33 = 186.67

Bounding box: [-2, -2, -10] to [2, 2, 10]
"""

import math


def test_stress_loft_mirror_is_watertight(stress_loft_mirror):
    mesh, _ = stress_loft_mirror
    assert mesh.is_watertight, "Loft+mirror mesh should be watertight (closed solid)"


def test_stress_loft_mirror_volume(stress_loft_mirror):
    mesh, _ = stress_loft_mirror
    single_frustum = 10.0 / 3.0 * (16.0 + 4.0 + math.sqrt(16.0 * 4.0))
    expected = 2.0 * single_frustum  # ~186.67
    assert abs(mesh.volume - expected) < 10.0, (
        f"Loft+mirror volume should be ~{expected:.1f}, got {mesh.volume:.1f}"
    )


def test_stress_loft_mirror_bounding_box(stress_loft_mirror):
    mesh, _ = stress_loft_mirror
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X and Y bounded by the larger 4x4 profile: [-2, 2]
    assert abs(bbox_min[0] - (-2.0)) < 0.5, f"Bbox min x should be ~-2, got {bbox_min[0]}"
    assert abs(bbox_min[1] - (-2.0)) < 0.5, f"Bbox min y should be ~-2, got {bbox_min[1]}"
    # Z spans -10 to 10 (mirrored)
    assert abs(bbox_min[2] - (-10.0)) < 0.5, f"Bbox min z should be ~-10, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 2.0) < 0.5, f"Bbox max x should be ~2, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 2.0) < 0.5, f"Bbox max y should be ~2, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_stress_loft_mirror_matches_kernel_volume(stress_loft_mirror):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_loft_mirror
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
