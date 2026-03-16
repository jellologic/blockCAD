"""Cross-validate L-shaped (non-convex) profile extrude against trimesh."""


def test_l_shape_is_watertight(l_shape):
    mesh, _ = l_shape
    assert mesh.is_watertight, "L-shape mesh should be watertight (closed solid)"


def test_l_shape_volume(l_shape):
    mesh, _ = l_shape
    # L-shape area = 10*10 - 5*5 = 75, extruded 5mm => volume = 375
    assert abs(mesh.volume - 375.0) < 2.0, f"L-shape volume should be ~375, got {mesh.volume}"


def test_l_shape_bounding_box(l_shape):
    mesh, _ = l_shape
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 10.0) < 0.5, f"Bbox max y should be ~10, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 5.0) < 0.5, f"Bbox max z should be ~5, got {bbox_max[2]}"


def test_l_shape_matches_kernel_volume(l_shape):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = l_shape
    assert abs(mesh.volume - props["volume"]) < 1.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
