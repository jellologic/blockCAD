"""Cross-validate 2D linear pattern geometry against trimesh (independent mesh library).

Geometry: 10x5x7 box patterned in a 2x3 grid.
  - 2 copies in X with spacing 15
  - 3 copies in Y with spacing 8
Total copies = 2 * 3 = 6, each 10x5x7 = 350, total volume = 2100.
Bounding box: X: 0..25, Y: 0..21, Z: 0..7.
"""


def test_linear_pattern_2d_is_watertight(linear_pattern_2d):
    mesh, _ = linear_pattern_2d
    assert mesh.is_watertight, "2D linear pattern mesh should be watertight (closed solid)"


def test_linear_pattern_2d_volume(linear_pattern_2d):
    mesh, _ = linear_pattern_2d
    # 6 copies of 10x5x7 = 2100
    assert abs(mesh.volume - 2100.0) < 5.0, f"2D linear pattern volume should be ~2100, got {mesh.volume}"


def test_linear_pattern_2d_bounding_box(linear_pattern_2d):
    mesh, _ = linear_pattern_2d
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Origin should be near (0, 0, 0)
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    # X: 2 copies at x=0, x=15; each box is 10 wide, so max x = 15+10 = 25
    assert abs(bbox_max[0] - 25.0) < 0.5, f"Bbox max x should be ~25, got {bbox_max[0]}"
    # Y: 3 copies at y=0, y=8, y=16; each box is 5 tall, so max y = 16+5 = 21
    assert abs(bbox_max[1] - 21.0) < 0.5, f"Bbox max y should be ~21, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_linear_pattern_2d_matches_kernel_volume(linear_pattern_2d):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = linear_pattern_2d
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
