"""Stress test: cross-validate circular pattern + shell against trimesh.

Geometry: 2x2x5 box (at x=5..7, y=-1..1, z=0..5)
  -> CircularPattern(6 copies, 60° apart, axis at origin along Z)
  -> Shell(top face removed, t=0.3)

6 non-overlapping boxes arranged in a ring at radius 5-7 from the Z axis,
each hollowed with 0.3mm wall thickness and top face open.

Per-box shell volume:
  outer = 2 * 2 * 5 = 20
  inner = (2-0.6) * (2-0.6) * (5-0.3) = 1.4 * 1.4 * 4.7 ~= 9.21
  shell_vol = 20 - 9.21 ~= 10.79
Total volume ~= 6 * 10.79 ~= 64.7

Bounding box: boxes at radius 5-7 from origin, 60° apart.
  Extent: roughly -7..7 in X and Y, 0..5 in Z.
"""


def test_watertight(stress_circular_shell):
    mesh, _ = stress_circular_shell
    assert mesh.is_watertight, "Stress circular-shell mesh should be watertight"


def test_volume_bounds(stress_circular_shell):
    mesh, _ = stress_circular_shell
    # 6 shelled boxes, each ~10.8 volume => total ~65
    # Use generous bounds for tessellation error
    assert 30.0 < mesh.volume < 130.0, (
        f"Volume should be roughly 30-130 (expected ~65), got {mesh.volume}"
    )


def test_bounding_box(stress_circular_shell):
    mesh, _ = stress_circular_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Boxes at radius 5-7, pattern fills 360 deg => bbox should extend roughly -7..7 in X and Y
    assert bbox_max[0] > 4.0, f"Bbox max x should be > 4 (boxes at radius 5-7), got {bbox_max[0]}"
    assert bbox_min[0] < -4.0, f"Bbox min x should be < -4, got {bbox_min[0]}"
    assert bbox_max[1] > 4.0, f"Bbox max y should be > 4, got {bbox_max[1]}"
    assert bbox_min[1] < -4.0, f"Bbox min y should be < -4, got {bbox_min[1]}"
    # Z extent: 0..5
    assert abs(bbox_min[2]) < 0.5, f"Bbox min z should be near 0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 5.0) < 0.5, f"Bbox max z should be ~5, got {bbox_max[2]}"


def test_kernel_volume_match(stress_circular_shell):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_circular_shell
    assert abs(mesh.volume - props["volume"]) < 10.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
