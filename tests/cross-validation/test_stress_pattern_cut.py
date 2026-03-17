"""Stress test: cross-validate box + linear pattern + cut extrude against trimesh."""


def test_watertight(stress_pattern_cut):
    mesh, _ = stress_pattern_cut
    assert mesh.is_watertight, "Stress pattern-cut mesh should be watertight"


def test_volume_bounds(stress_pattern_cut):
    mesh, _ = stress_pattern_cut
    assert 620.0 < mesh.volume < 720.0, f"Volume should be ~676, got {mesh.volume}"


def test_bounding_box(stress_pattern_cut):
    mesh, _ = stress_pattern_cut
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min)
    assert abs(bbox_max[0] - 25.0) < 0.5
    assert abs(bbox_max[1] - 5.0) < 0.5
    assert abs(bbox_max[2] - 7.0) < 0.5


def test_kernel_volume_match(stress_pattern_cut):
    mesh, props = stress_pattern_cut
    kv = props["volume"]
    assert abs(mesh.volume - kv) < 5.0, f"trimesh={mesh.volume} vs kernel={kv}"
