"""Cross-validate drafted box geometry against trimesh (independent mesh library).

Geometry: 10x5x7 box with 5-degree draft applied to two side faces.
Pull direction is +Z, so the side faces taper outward as Z increases.
The volume should differ slightly from the original 350 (10*5*7).
"""


def test_draft_is_watertight(draft):
    mesh, _ = draft
    assert mesh.is_watertight, "Draft mesh should be watertight (closed solid)"


def test_draft_volume(draft):
    mesh, _ = draft
    # Draft tapers side faces, so volume should differ from 350 but stay close
    assert abs(mesh.volume - 350.0) > 0.1, (
        f"Draft should change volume from 350, got {mesh.volume}"
    )
    assert abs(mesh.volume - 350.0) < 50.0, (
        f"Draft volume should be within 50 of 350, got {mesh.volume}"
    )


def test_draft_bounding_box(draft):
    mesh, _ = draft
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # The base (z=0) should remain at origin; top may extend slightly due to taper
    assert all(v < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    # X extent: base is 10, top may be wider due to draft
    assert bbox_max[0] > 9.5, f"Bbox max x should be >= ~10, got {bbox_max[0]}"
    assert bbox_max[0] < 12.0, f"Bbox max x should be < 12, got {bbox_max[0]}"
    # Y extent: base is 5, top may be wider
    assert bbox_max[1] > 4.5, f"Bbox max y should be >= ~5, got {bbox_max[1]}"
    assert bbox_max[1] < 7.0, f"Bbox max y should be < 7, got {bbox_max[1]}"
    # Z extent should remain 7
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_draft_matches_kernel_volume(draft):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = draft
    assert abs(mesh.volume - props["volume"]) < 5.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
