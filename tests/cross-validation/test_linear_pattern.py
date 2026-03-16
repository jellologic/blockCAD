"""Cross-validate linear pattern geometry against trimesh (independent mesh library).

Geometry: 10x5x7 box patterned 3 times along X with spacing 15.
Each copy is a full 10x5x7 box, so total volume = 3 * 350 = 1050.
Bounding box: X spans 0..40 (last copy starts at 30, ends at 40), Y: 0..5, Z: 0..7.
"""


def test_linear_pattern_is_watertight(linear_pattern):
    mesh, _ = linear_pattern
    assert mesh.is_watertight, "Linear pattern mesh should be watertight (closed solid)"


def test_linear_pattern_volume(linear_pattern):
    mesh, _ = linear_pattern
    # 3 copies of 10x5x7 = 1050
    assert abs(mesh.volume - 1050.0) < 5.0, f"Linear pattern volume should be ~1050, got {mesh.volume}"


def test_linear_pattern_bounding_box(linear_pattern):
    mesh, _ = linear_pattern
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Origin should be near (0, 0, 0)
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    # 3 copies: at x=0, x=15, x=30; each box is 10 wide, so max x = 30+10 = 40
    assert abs(bbox_max[0] - 40.0) < 0.5, f"Bbox max x should be ~40, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_linear_pattern_matches_kernel_volume(linear_pattern):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = linear_pattern
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
