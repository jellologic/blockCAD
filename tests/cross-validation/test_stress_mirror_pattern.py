"""Cross-validate stress_mirror_pattern geometry against trimesh.

Geometry: 10x5x7 box extruded from sketch, mirrored across YZ plane at x=0,
then linear pattern 2x along X with spacing 25.

The mirror doubles the box across x=0, producing a body spanning [-10,10] x [0,5] x [0,7].
The linear pattern creates 2 copies along X with spacing 25, so the second mirrored
body is shifted +25 in X, spanning [15,35] x [0,5] x [0,7].

4 copies total (original + mirror, then 2x pattern). No overlap since gap = 25 - 20 = 5.
Expected volume: 4 * 350 = 1400.

Bounding box: x in [-10, 35], y in [0, 5], z in [0, 7].
"""


def test_stress_mirror_pattern_is_watertight(stress_mirror_pattern):
    mesh, _ = stress_mirror_pattern
    assert mesh.is_watertight, "Mirror+pattern mesh should be watertight (closed solid)"


def test_stress_mirror_pattern_volume(stress_mirror_pattern):
    mesh, _ = stress_mirror_pattern
    # 4 copies of 10*5*7 = 350 each => 1400 total
    assert abs(mesh.volume - 1400.0) < 30.0, (
        f"Mirror+pattern volume should be ~1400, got {mesh.volume}"
    )


def test_stress_mirror_pattern_bounding_box(stress_mirror_pattern):
    mesh, _ = stress_mirror_pattern
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X spans [-10, 35]: original mirror [-10,10], patterned copy [15,35]
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 35.0) < 0.5, f"Bbox max x should be ~35, got {bbox_max[0]}"
    # Y spans [0, 5]
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    # Z spans [0, 7]
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_stress_mirror_pattern_matches_kernel_volume(stress_mirror_pattern):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_mirror_pattern
    assert abs(mesh.volume - props["volume"]) < 10.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
