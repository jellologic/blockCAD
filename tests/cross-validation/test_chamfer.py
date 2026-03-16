"""Cross-validate chamfered box geometry."""


def test_chamfer_is_watertight(chamfer):
    mesh, _ = chamfer
    assert mesh.is_watertight, "Chamfered mesh should be watertight"


def test_chamfer_reduces_volume(chamfer):
    mesh, _ = chamfer
    assert mesh.volume < 350.0, f"Chamfer should reduce volume below 350, got {mesh.volume}"
    assert mesh.volume > 300.0, f"Chamfer shouldn't remove too much, got {mesh.volume}"


def test_chamfer_adds_face(chamfer, box):
    """Chamfer adds a flat bevel face → more triangles."""
    chamfer_mesh, _ = chamfer
    box_mesh, _ = box
    assert len(chamfer_mesh.faces) > len(box_mesh.faces)


def test_chamfer_matches_kernel_volume(chamfer):
    mesh, props = chamfer
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
