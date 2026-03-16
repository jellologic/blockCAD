"""Cross-validate stress test: Extrude 10x5x7 box -> Shell(remove top AND front, t=0.5).

Two faces removed creates two openings. The shell hollows out the box with 0.5mm
walls, leaving the top (+Z) and front (-Y) faces open. This tests multi-face
removal in the shell operation.

Outer box: 10x5x7 = 350. Inner cavity extends to both openings, so the shell
volume is the outer minus inner with two open sides. Approximately:
  outer = 350
  inner ~= 9 x 4 x 6.5 = 234 (rough, depends on which walls exist)
  shell volume ~= 350 - 234 = 116 (approximate)
"""


def test_watertight(stress_multi_face_shell):
    mesh, _ = stress_multi_face_shell
    assert mesh.is_watertight, "Multi-face shell mesh should be watertight"


def test_volume_bounds(stress_multi_face_shell):
    mesh, _ = stress_multi_face_shell
    # Shell with two openings: volume should be well below solid 350
    # but still substantial with 0.5mm walls.
    assert mesh.volume > 30.0, f"Volume should be > 30, got {mesh.volume}"
    assert mesh.volume < 300.0, f"Volume should be < 300, got {mesh.volume}"


def test_bounding_box(stress_multi_face_shell):
    mesh, _ = stress_multi_face_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer dimensions should still be close to 10x5x7.
    x_extent = bbox_max[0] - bbox_min[0]
    y_extent = bbox_max[1] - bbox_min[1]
    z_extent = bbox_max[2] - bbox_min[2]
    assert 9.0 < x_extent < 11.0, f"X extent should be ~10, got {x_extent}"
    assert 4.0 < y_extent < 6.0, f"Y extent should be ~5, got {y_extent}"
    assert 6.0 < z_extent < 8.0, f"Z extent should be ~7, got {z_extent}"


def test_kernel_volume_match(stress_multi_face_shell):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = stress_multi_face_shell
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
