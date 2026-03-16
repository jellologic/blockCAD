"""Cross-validate stress test: tall thin extrude against trimesh.

Geometry: 2x2 rectangle sketch extruded 50mm along Z.
High aspect ratio (25:1) stresses tessellation and mass property computation.
Expected volume = 2 * 2 * 50 = 200 mm^3.
"""


def test_stress_tall_thin_extrude_is_watertight(stress_tall_thin_extrude):
    mesh, _ = stress_tall_thin_extrude
    assert mesh.is_watertight, "Tall thin extrude mesh should be watertight"


def test_stress_tall_thin_extrude_volume(stress_tall_thin_extrude):
    mesh, _ = stress_tall_thin_extrude
    vol = abs(mesh.volume)
    assert abs(vol - 200.0) < 2.0, (
        f"Tall thin extrude volume should be ~200, got {vol}"
    )


def test_stress_tall_thin_extrude_bounding_box(stress_tall_thin_extrude):
    mesh, _ = stress_tall_thin_extrude
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    # Bounding box should be approximately 2x2x50
    assert abs(bbox_max[0] - bbox_min[0] - 2.0) < 0.5, (
        f"Bbox X extent should be ~2, got {bbox_max[0] - bbox_min[0]}"
    )
    assert abs(bbox_max[1] - bbox_min[1] - 2.0) < 0.5, (
        f"Bbox Y extent should be ~2, got {bbox_max[1] - bbox_min[1]}"
    )
    assert abs(bbox_max[2] - bbox_min[2] - 50.0) < 0.5, (
        f"Bbox Z extent should be ~50, got {bbox_max[2] - bbox_min[2]}"
    )


def test_stress_tall_thin_extrude_matches_kernel_volume(stress_tall_thin_extrude):
    """trimesh volume should match blockCAD's divergence-theorem volume."""
    mesh, props = stress_tall_thin_extrude
    assert abs(abs(mesh.volume) - abs(props["volume"])) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
