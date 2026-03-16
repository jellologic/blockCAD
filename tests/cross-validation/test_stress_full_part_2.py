"""Cross-validate stress test: Extrude 10x5x7 box -> Chamfer(edge 0, d=0.5) -> Mirror(YZ at x=0) -> Draft(2 faces, 5 deg).

Geometry: Four operations covering finish (chamfer), transform (mirror), and taper (draft).
- Base box: 10x5x7 = volume 350
- Chamfer removes a small amount (~0.06)
- Mirror across YZ doubles the body: X spans [-10, 10], volume ~700
- Draft tilts 2 side faces by 5 degrees, slightly modifying volume
"""

def test_watertight(stress_full_part_2):
    mesh, _ = stress_full_part_2
    assert mesh.is_watertight, "stress_full_part_2 mesh should be watertight (closed solid)"


def test_volume_bounds(stress_full_part_2):
    mesh, _ = stress_full_part_2
    # Mirrored chamfered box ~700, draft modifies slightly
    assert 600.0 < mesh.volume < 750.0, (
        f"Volume should be ~700 (mirrored box minus chamfer, plus draft adjustment), got {mesh.volume}"
    )


def test_bounding_box(stress_full_part_2):
    mesh, _ = stress_full_part_2
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X spans [-10, 10] after mirror (draft may widen slightly)
    assert abs(bbox_min[0] - (-10.0)) < 1.0, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 1.0, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y spans [0, 5] (draft may widen slightly)
    assert abs(bbox_min[1] - 0.0) < 1.0, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 5.0) < 1.0, f"Bbox max y should be ~5, got {bbox_max[1]}"
    # Z spans [0, 7]
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_kernel_volume_match(stress_full_part_2):
    """trimesh volume should match blockCAD kernel divergence-theorem volume."""
    mesh, props = stress_full_part_2
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
