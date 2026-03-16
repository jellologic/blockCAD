"""Cross-validate symmetric extrude (10x5 rect, 7mm total, centered on z=0) against trimesh."""


def test_symmetric_extrude_is_watertight(symmetric_extrude):
    mesh, _ = symmetric_extrude
    assert mesh.is_watertight, "Symmetric extrude mesh should be watertight"


def test_symmetric_extrude_volume(symmetric_extrude):
    mesh, _ = symmetric_extrude
    assert abs(mesh.volume - 350.0) < 2.0, f"Volume should be ~350, got {mesh.volume}"


def test_symmetric_extrude_bounding_box(symmetric_extrude):
    mesh, _ = symmetric_extrude
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert abs(bbox_min[2] - (-3.5)) < 0.5, f"Bbox z-min should be ~-3.5, got {bbox_min[2]}"
    assert abs(bbox_max[2] - 3.5) < 0.5, f"Bbox z-max should be ~3.5, got {bbox_max[2]}"


def test_symmetric_extrude_matches_kernel_volume(symmetric_extrude):
    """trimesh volume should match blockCAD kernel volume."""
    mesh, props = symmetric_extrude
    assert abs(mesh.volume - props["volume"]) < 1.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
