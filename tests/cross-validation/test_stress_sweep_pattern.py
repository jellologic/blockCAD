"""Cross-validate stress_sweep_pattern geometry against trimesh.

Geometry: 4x4 square swept 10 units along Z (volume=160), then circular
pattern with 3 copies at 120 degrees apart around Z axis offset at x=-15.
Total expected volume: 3 * 160 = 480.

Bounding box reasoning:
- Copy 0 (0 deg): sweep center at (0,0). Box x=[-2,2], y=[-2,2], z=[0,10].
- Copy 1 (120 deg): center rotated to (-22.5, 12.99). Box x~[-24.5,-20.5], y~[11.0,15.0].
- Copy 2 (240 deg): center rotated to (-22.5, -12.99). Box x~[-24.5,-20.5], y~[-15.0,-11.0].
Overall: x~[-24.5, 2], y~[-15, 15], z=[0, 10].
"""


def test_stress_sweep_pattern_is_watertight(stress_sweep_pattern):
    mesh, _ = stress_sweep_pattern
    assert mesh.is_watertight, "Stress sweep pattern mesh should be watertight (closed solid)"


def test_stress_sweep_pattern_volume(stress_sweep_pattern):
    mesh, _ = stress_sweep_pattern
    # 3 copies of 4*4*10 = 160 each => 480 total
    assert abs(mesh.volume - 480.0) < 20.0, (
        f"Stress sweep pattern volume should be ~480, got {mesh.volume}"
    )


def test_stress_sweep_pattern_bounding_box(stress_sweep_pattern):
    mesh, _ = stress_sweep_pattern
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Copy 0 extends to x=2; copies 1 & 2 extend to x~-24.5
    assert abs(bbox_min[0] - (-24.5)) < 1.0, f"Bbox min x should be ~-24.5, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 2.0) < 1.0, f"Bbox max x should be ~2, got {bbox_max[0]}"
    # Copies 1 & 2 extend to y~+/-15
    assert abs(bbox_min[1] - (-15.0)) < 1.0, f"Bbox min y should be ~-15, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 15.0) < 1.0, f"Bbox max y should be ~15, got {bbox_max[1]}"
    # Z range: [0, 10]
    assert abs(bbox_min[2]) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_stress_sweep_pattern_matches_kernel_volume(stress_sweep_pattern):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_sweep_pattern
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
