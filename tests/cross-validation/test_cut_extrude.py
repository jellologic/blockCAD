"""Cross-validate cut-extrude (pocket) geometry against trimesh.

Geometry: 10x5x7 box with a 4x2x3 blind pocket cut from the bottom face upward.
Box volume = 350, pocket volume = 24, expected result = 326.
Bounding box should still be 10x5x7 (pocket is internal).
"""

def test_cut_extrude_is_watertight(cut_extrude):
    mesh, _ = cut_extrude
    assert mesh.is_watertight, "Cut-extrude mesh should be watertight (closed solid)"


def test_cut_extrude_volume(cut_extrude):
    mesh, _ = cut_extrude
    # 10*5*7 - 4*2*3 = 350 - 24 = 326
    expected = 326.0
    assert abs(mesh.volume - expected) < 5.0, (
        f"Cut-extrude volume should be ~{expected}, got {mesh.volume}"
    )


def test_cut_extrude_bounding_box(cut_extrude):
    mesh, _ = cut_extrude
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Outer bounding box should still be 10x5x7 (pocket doesn't change it)
    assert all(abs(v) < 0.5 for v in bbox_min), f"Bbox min should be near origin, got {bbox_min}"
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_cut_extrude_matches_kernel_volume(cut_extrude):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = cut_extrude
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )


def test_cut_extrude_volume_less_than_solid(cut_extrude, box):
    mesh, _ = cut_extrude
    box_mesh, _ = box
    assert mesh.volume < box_mesh.volume, (
        f"Cut-extrude volume ({mesh.volume}) should be less than solid box ({box_mesh.volume})"
    )
