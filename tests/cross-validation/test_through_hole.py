"""Cross-validate CutExtrude through-hole geometry against trimesh."""


def test_through_hole_is_watertight(through_hole):
    mesh, _ = through_hole
    assert mesh.is_watertight, "Through-hole mesh should be watertight"


def test_through_hole_volume(through_hole):
    mesh, _ = through_hole
    expected = 294.0
    assert abs(mesh.volume - expected) < 5.0, f"Volume should be ~{expected}, got {mesh.volume}"


def test_through_hole_bounding_box(through_hole):
    mesh, _ = through_hole
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min)
    assert abs(bbox_max[0] - 10.0) < 0.5
    assert abs(bbox_max[1] - 5.0) < 0.5
    assert abs(bbox_max[2] - 7.0) < 0.5


def test_through_hole_matches_kernel_volume(through_hole):
    mesh, props = through_hole
    kv = props["volume"]
    assert abs(mesh.volume - kv) < 2.0, f"trimesh={mesh.volume} vs kernel={kv}"
