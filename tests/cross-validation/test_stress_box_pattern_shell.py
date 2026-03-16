"""Stress test: cross-validate box + linear pattern + shell against trimesh.

Geometry: 10x5x7 box -> LinearPattern(3x, spacing 15, along X) -> Shell(top face removed, t=0.3).

Three separate 10x5x7 boxes at x=0, x=15, x=30 (spacing > width so no overlap),
each shelled with wall thickness 0.3 and top face removed.

Per-box shell volume:
  outer = 10 * 5 * 7 = 350
  inner = (10-0.6) * (5-0.6) * (7-0.3) = 9.4 * 4.4 * 6.7 ~= 277.1
  shell = 350 - 277.1 ~= 72.9
Total volume ~= 3 * 72.9 ~= 218.7

Bounding box: X: 0..40, Y: 0..5, Z: 0..7 (same as the unshelled pattern).
"""


def test_watertight(stress_box_pattern_shell):
    mesh, _ = stress_box_pattern_shell
    assert mesh.is_watertight, "Stress box-pattern-shell mesh should be watertight"


def test_volume_bounds(stress_box_pattern_shell):
    mesh, _ = stress_box_pattern_shell
    # Each shelled box ~72.9, total ~218.7; use generous tolerance for tessellation error
    assert 150.0 < mesh.volume < 300.0, (
        f"Volume should be roughly 150-300 (expected ~219), got {mesh.volume}"
    )


def test_bounding_box(stress_box_pattern_shell):
    mesh, _ = stress_box_pattern_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Origin near (0, 0, 0)
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    # 3 copies at x=0, 15, 30; each 10 wide -> max x = 40
    assert abs(bbox_max[0] - 40.0) < 0.5, f"Bbox max x should be ~40, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_kernel_volume_match(stress_box_pattern_shell):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_box_pattern_shell
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
