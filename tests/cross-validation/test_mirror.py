"""Cross-validate mirrored box geometry against trimesh (independent mesh library).

Geometry: 10x5x7 box mirrored across the YZ plane (X=0).
Original box spans [0,10] x [0,5] x [0,7].
Mirrored copy spans [-10,0] x [0,5] x [0,7].
Combined solid: [-10,10] x [0,5] x [0,7], volume = 20*5*7 = 700.
"""


def test_mirror_is_watertight(mirror):
    mesh, _ = mirror
    assert mesh.is_watertight, "Mirrored mesh should be watertight (closed solid)"


def test_mirror_volume(mirror):
    mesh, _ = mirror
    # 2x original box: 2 * 10*5*7 = 700
    assert abs(mesh.volume - 700.0) < 10.0, (
        f"Mirrored box volume should be ~700, got {mesh.volume}"
    )


def test_mirror_bounding_box(mirror):
    mesh, _ = mirror
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # X should span [-10, 10] (doubled in mirror direction)
    assert abs(bbox_min[0] - (-10.0)) < 0.5, f"Bbox min x should be ~-10, got {bbox_min[0]}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    # Y and Z unchanged
    assert abs(bbox_min[1] - 0.0) < 0.5, f"Bbox min y should be ~0, got {bbox_min[1]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_min[2] - 0.0) < 0.5, f"Bbox min z should be ~0, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_mirror_matches_kernel_volume(mirror):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = mirror
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
