"""Cross-validate boolean union geometry against trimesh (independent mesh library).

Geometry: union of two 10x5x7 boxes, second offset by 5 in X.
Box A spans (0..10, 0..5, 0..7), Box B spans (5..15, 0..5, 0..7).
Overlap region: (5..10, 0..5, 0..7) = 5*5*7 = 175.
Expected union volume: 350 + 350 - 175 = 525.
Expected bounding box: (0..15, 0..5, 0..7).
"""


def test_boolean_union_is_watertight(boolean_union):
    mesh, _ = boolean_union
    assert mesh.is_watertight, "Boolean union mesh should be watertight (closed solid)"


def test_boolean_union_volume(boolean_union):
    mesh, _ = boolean_union
    assert abs(mesh.volume - 525.0) < 10.0, (
        f"Boolean union volume should be ~525, got {mesh.volume}"
    )


def test_boolean_union_bounding_box(boolean_union):
    mesh, _ = boolean_union
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min), (
        f"Bbox min should be near origin, got {bbox_min}"
    )
    assert abs(bbox_max[0] - 15.0) < 0.5, f"Bbox max x should be ~15, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_boolean_union_matches_kernel_volume(boolean_union):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = boolean_union
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
