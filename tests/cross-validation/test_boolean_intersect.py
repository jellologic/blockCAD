"""Cross-validate BooleanIntersect geometry against trimesh (independent mesh library).

Geometry: intersection of two 10x10x10 boxes.
  Box A: (0,0,0) to (10,10,10)
  Box B: (5,5,0) to (15,15,10)
  Result: 5x5x10 box from (5,5,0) to (10,10,10), volume = 250.
"""


def test_boolean_intersect_is_watertight(boolean_intersect):
    mesh, _ = boolean_intersect
    assert mesh.is_watertight, "Boolean intersect mesh should be watertight (closed solid)"


def test_boolean_intersect_volume(boolean_intersect):
    mesh, _ = boolean_intersect
    assert abs(mesh.volume - 250.0) < 5.0, f"Intersect volume should be ~250, got {mesh.volume}"


def test_boolean_intersect_bounding_box(boolean_intersect):
    mesh, _ = boolean_intersect
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Intersection region: (5,5,0) to (10,10,10)
    assert abs(bbox_min[0] - 5.0) < 0.5, f"Bbox min x should be ~5, got {bbox_min[0]}"
    assert abs(bbox_min[1] - 5.0) < 0.5, f"Bbox min y should be ~5, got {bbox_min[1]}"
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 10.0) < 0.5, f"Bbox max y should be ~10, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 10.0) < 0.5, f"Bbox max z should be ~10, got {bbox_max[2]}"


def test_boolean_intersect_matches_kernel_volume(boolean_intersect):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = boolean_intersect
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
