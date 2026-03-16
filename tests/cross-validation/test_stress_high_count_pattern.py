"""Stress test: 2x2x2 box patterned 10x along X with spacing 3."""


def test_stress_high_count_pattern_is_watertight(stress_high_count_pattern):
    mesh, _ = stress_high_count_pattern
    assert mesh.is_watertight, "High count pattern mesh should be watertight"


def test_stress_high_count_pattern_volume(stress_high_count_pattern):
    mesh, _ = stress_high_count_pattern
    # 10 copies of 2x2x2 = 80
    assert abs(mesh.volume - 80.0) < 2.0, (
        f"Pattern volume should be ~80, got {mesh.volume}"
    )


def test_stress_high_count_pattern_bounding_box(stress_high_count_pattern):
    mesh, _ = stress_high_count_pattern
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # 10 copies: first at x=0..2, last at x=27..29
    assert abs(bbox_max[0] - 29.0) < 1.0, f"Bbox max x should be ~29, got {bbox_max[0]}"
    assert abs(bbox_max[2] - 2.0) < 0.5, f"Bbox max z should be ~2, got {bbox_max[2]}"


def test_stress_high_count_pattern_matches_kernel_volume(stress_high_count_pattern):
    mesh, props = stress_high_count_pattern
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh ({mesh.volume}) vs kernel ({props['volume']})"
    )
