"""Cross-validate stress test: regular hexagon (6 vertices, circumradius=5) extruded 5mm."""

import math


def test_watertight(stress_hexagon):
    mesh, _ = stress_hexagon
    assert mesh.is_watertight, "Hexagon extrude mesh should be watertight"


def test_volume_approx_324(stress_hexagon):
    mesh, _ = stress_hexagon
    # Regular hexagon area = (3*sqrt(3)/2) * r^2 = ~64.95, volume = 64.95 * 5 = ~324.76
    expected = 3.0 * math.sqrt(3) / 2.0 * 25.0 * 5.0
    assert abs(mesh.volume - expected) < 5.0, (
        f"Hexagon volume should be ~{expected:.1f}, got {mesh.volume}"
    )


def test_bounding_box(stress_hexagon):
    mesh, _ = stress_hexagon
    extents = mesh.bounds[1] - mesh.bounds[0]
    # Hexagon with circumradius 5: X extent = 10, Y extent = 2 * 5*sin(60) = ~8.66, Z = 5
    assert abs(extents[0] - 10.0) < 1.0, f"X extent should be ~10, got {extents[0]}"
    assert abs(extents[1] - 8.66) < 1.0, f"Y extent should be ~8.66, got {extents[1]}"
    assert abs(extents[2] - 5.0) < 1.0, f"Z extent should be ~5, got {extents[2]}"


def test_kernel_volume_match(stress_hexagon):
    mesh, props = stress_hexagon
    assert abs(mesh.volume - props["volume"]) < 2.0, (
        f"trimesh volume ({mesh.volume}) should match kernel ({props['volume']})"
    )
