"""Stress test: 10x5x7 box with tiny fillet r=0.1 on edge 0."""


def test_stress_tiny_fillet_is_watertight(stress_tiny_fillet):
    mesh, _ = stress_tiny_fillet
    assert mesh.is_watertight, "Tiny fillet mesh should be watertight"


def test_stress_tiny_fillet_volume_close_to_350(stress_tiny_fillet):
    mesh, _ = stress_tiny_fillet
    assert abs(mesh.volume - 350.0) < 1.0, (
        f"Tiny fillet volume should be ~350, got {mesh.volume}"
    )


def test_stress_tiny_fillet_bounding_box(stress_tiny_fillet):
    mesh, _ = stress_tiny_fillet
    bbox_max = mesh.bounds[1]
    assert abs(bbox_max[0] - 10.0) < 0.5
    assert abs(bbox_max[1] - 5.0) < 0.5
    assert abs(bbox_max[2] - 7.0) < 0.5


def test_stress_tiny_fillet_matches_kernel_volume(stress_tiny_fillet):
    mesh, props = stress_tiny_fillet
    assert abs(mesh.volume - props["volume"]) < 1.0, (
        f"trimesh ({mesh.volume}) vs kernel ({props['volume']})"
    )
