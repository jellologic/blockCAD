"""Cross-validate compound operation: Extrude → Fillet → Shell.

Geometry: 10x5x7 box, fillet one edge with r=1, then shell with top face
removed and wall thickness 0.5.

The compound volume should be less than both:
- shell-only (~116): the fillet removes material before shelling
- fillet-only (~349): the shell hollows out the body
"""


def test_compound_is_watertight(compound_fillet_shell):
    mesh, _ = compound_fillet_shell
    assert mesh.is_watertight, "Compound fillet+shell mesh should be watertight"


def test_compound_volume_less_than_shell_only(compound_fillet_shell, shell):
    mesh, _ = compound_fillet_shell
    shell_mesh, _ = shell
    assert mesh.volume < shell_mesh.volume, (
        f"Compound volume ({mesh.volume:.1f}) should be less than "
        f"shell-only volume ({shell_mesh.volume:.1f})"
    )


def test_compound_volume_less_than_fillet_only(compound_fillet_shell, fillet):
    mesh, _ = compound_fillet_shell
    fillet_mesh, _ = fillet
    assert mesh.volume < fillet_mesh.volume, (
        f"Compound volume ({mesh.volume:.1f}) should be less than "
        f"fillet-only volume ({fillet_mesh.volume:.1f})"
    )


def test_compound_bounding_box(compound_fillet_shell):
    """Outer bounding box should still be approximately 10x5x7."""
    mesh, _ = compound_fillet_shell
    bbox_min = mesh.bounds[0]
    bbox_max = mesh.bounds[1]
    assert all(abs(v) < 0.5 for v in bbox_min), (
        f"Bbox min should be near origin, got {bbox_min}"
    )
    assert abs(bbox_max[0] - 10.0) < 0.5, f"Bbox max x should be ~10, got {bbox_max[0]}"
    assert abs(bbox_max[1] - 5.0) < 0.5, f"Bbox max y should be ~5, got {bbox_max[1]}"
    assert abs(bbox_max[2] - 7.0) < 0.5, f"Bbox max z should be ~7, got {bbox_max[2]}"


def test_compound_matches_kernel_volume(compound_fillet_shell):
    """trimesh volume should match blockCAD kernel's divergence-theorem volume."""
    mesh, props = compound_fillet_shell
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume:.2f}) should match kernel ({props['volume']:.2f})"
    )
