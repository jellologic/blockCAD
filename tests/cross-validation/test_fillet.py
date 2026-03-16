"""Cross-validate filleted box geometry."""


def test_fillet_is_watertight(fillet):
    mesh, _ = fillet
    assert mesh.is_watertight, "Filleted mesh should be watertight"


def test_fillet_reduces_volume(fillet):
    mesh, _ = fillet
    assert mesh.volume < 350.0, f"Fillet should reduce volume below 350, got {mesh.volume}"
    assert mesh.volume > 300.0, f"Fillet shouldn't remove too much, got {mesh.volume}"


def test_fillet_bounding_box_unchanged(fillet):
    """Fillet on an interior edge shouldn't change the overall bounding box."""
    mesh, _ = fillet
    bbox_max = mesh.bounds[1]
    # Bounding box should still be approximately 10x5x7
    assert bbox_max[0] < 11.0, f"Bbox max x should be <=10, got {bbox_max[0]}"
    assert bbox_max[1] < 6.0, f"Bbox max y should be <=5, got {bbox_max[1]}"
    assert bbox_max[2] < 8.0, f"Bbox max z should be <=7, got {bbox_max[2]}"


def test_fillet_more_triangles_than_box(fillet, box):
    """Fillet adds arc faces → more triangles."""
    fillet_mesh, _ = fillet
    box_mesh, _ = box
    assert len(fillet_mesh.faces) > len(box_mesh.faces), (
        f"Fillet should have more faces ({len(fillet_mesh.faces)}) than box ({len(box_mesh.faces)})"
    )


def test_fillet_matches_kernel_volume(fillet):
    mesh, props = fillet
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
