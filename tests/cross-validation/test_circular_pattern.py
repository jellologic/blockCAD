"""Cross-validate circular pattern geometry against trimesh (independent mesh library).

Geometry: 2x2x5 box offset from origin, patterned 4x around Z axis at 90 degree intervals.
Single box volume: 2*2*5 = 20. Total expected volume: 80.
"""


def test_circular_pattern_is_watertight(circular_pattern):
    mesh, _ = circular_pattern
    assert mesh.is_watertight, "Circular pattern mesh should be watertight (closed solid)"


def test_circular_pattern_volume(circular_pattern):
    mesh, _ = circular_pattern
    # 4 copies of 2x2x5 box = 80
    assert abs(mesh.volume - 80.0) < 5.0, f"Circular pattern volume should be ~80, got {mesh.volume}"


def test_circular_pattern_bounding_box(circular_pattern):
    mesh, _ = circular_pattern
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # The pattern places boxes at 0, 90, 180, 270 degrees around Z.
    # Original box at x=[5,7], y=[-1,1]. After 90 deg rotation: x=[-1,1], y=[5,7].
    # After 180 deg: x=[-7,-5], y=[-1,1]. After 270 deg: x=[-1,1], y=[-7,-5].
    # Overall bbox: x=[-7,7], y=[-7,7], z=[0,5]
    assert abs(bbox_min[0] - (-7.0)) < 0.5, f"Bbox min x should be ~-7, got {bbox_min[0]}"
    assert abs(bbox_min[1] - (-7.0)) < 0.5, f"Bbox min y should be ~-7, got {bbox_min[1]}"
    assert abs(bbox_min[2]) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 7.0) < 0.5, f"Bbox max x should be ~7, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 7.0) < 0.5, f"Bbox max y should be ~7, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 5.0) < 0.5, f"Bbox max z should be ~5, got {bbox_max[2]}"


def test_circular_pattern_matches_kernel_volume(circular_pattern):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = circular_pattern
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
