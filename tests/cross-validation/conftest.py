"""Shared fixtures for cross-validation tests."""

import json
import os
import trimesh
import pytest

FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "fixtures")


def load_stl(name: str) -> trimesh.Trimesh:
    """Load an STL fixture file as a trimesh."""
    path = os.path.join(FIXTURES_DIR, f"{name}.stl")
    return trimesh.load(path)


def load_kernel_props(name: str) -> dict:
    """Load blockCAD kernel mass properties JSON."""
    path = os.path.join(FIXTURES_DIR, f"{name}.json")
    with open(path) as f:
        return json.load(f)


@pytest.fixture
def box():
    """10x5x7 extruded box."""
    return load_stl("box_10x5x7"), load_kernel_props("box_10x5x7")


@pytest.fixture
def fillet():
    """10x5x7 box with r=1 fillet on one edge."""
    return load_stl("box_fillet_r1"), load_kernel_props("box_fillet_r1")


@pytest.fixture
def chamfer():
    """10x5x7 box with d=1 chamfer on one edge."""
    return load_stl("box_chamfer_d1"), load_kernel_props("box_chamfer_d1")


@pytest.fixture
def cylinder():
    """Cylinder r=5, h=10."""
    return load_stl("cylinder_r5_h10"), load_kernel_props("cylinder_r5_h10")


@pytest.fixture
def sweep():
    """4x4 square swept 10 units along Z axis."""
    return load_stl("sweep_straight"), load_kernel_props("sweep_straight")


@pytest.fixture
def sweep_twisted():
    """4x4 square swept 10 units along Z with 90-degree twist."""
    return load_stl("sweep_twisted"), load_kernel_props("sweep_twisted")


@pytest.fixture
def cut_extrude():
    """10x5x7 box with a 4x2x3 blind pocket cut from the bottom."""
    return load_stl("box_cut_pocket"), load_kernel_props("box_cut_pocket")


@pytest.fixture
def compound_cut_chamfer():
    """10x5x7 box with d=0.5 chamfer on one edge and 4x2x3 pocket."""
    return load_stl("compound_cut_chamfer"), load_kernel_props("compound_cut_chamfer")


@pytest.fixture
def shell():
    """10x5x7 box shelled with t=0.5, top face removed."""
    return load_stl("box_shell_t05"), load_kernel_props("box_shell_t05")


@pytest.fixture
def draft():
    """10x5x7 box with 5-degree draft on two side faces."""
    return load_stl("box_draft_5deg"), load_kernel_props("box_draft_5deg")


@pytest.fixture
def revolve():
    """Full 360-degree revolve of rectangle around Y-axis (annular solid)."""
    return load_stl("revolve_full"), load_kernel_props("revolve_full")


@pytest.fixture
def revolve_half():
    """180-degree revolve of rectangle around Y-axis (half-annulus)."""
    return load_stl("revolve_half"), load_kernel_props("revolve_half")


@pytest.fixture
def mirror():
    """10x5x7 box mirrored across YZ plane (X=0)."""
    return load_stl("mirror_box"), load_kernel_props("mirror_box")


@pytest.fixture
def linear_pattern():
    """10x5x7 box patterned 3x along X with spacing 15."""
    return load_stl("linear_pattern_3x"), load_kernel_props("linear_pattern_3x")


@pytest.fixture
def linear_pattern_2d():
    """10x5x7 box patterned in 2x3 grid (X spacing 15, Y spacing 8)."""
    return load_stl("linear_pattern_2d"), load_kernel_props("linear_pattern_2d")


@pytest.fixture
def circular_pattern():
    """2x2x5 box patterned 4x around Z axis at 90 degree intervals."""
    return load_stl("circular_pattern_4x"), load_kernel_props("circular_pattern_4x")


@pytest.fixture
def loft():
    """Loft between 4x4 square at z=0 and 2x2 square at z=10 (tapered prism)."""
    return load_stl("loft_taper"), load_kernel_props("loft_taper")


@pytest.fixture
def loft_3section():
    """Loft between 4x4, 3x3, 2x2 squares at z=0, 5, 10."""
    return load_stl("loft_3section"), load_kernel_props("loft_3section")


@pytest.fixture
def boolean_intersect():
    """Intersection of two 10x10x10 boxes offset by (5,5,0). Result: 5x5x10."""
    return load_stl("boolean_intersect"), load_kernel_props("boolean_intersect")


@pytest.fixture
def boolean_subtract():
    """10x5x7 box with a 4x3x10 box subtracted at offset (3,1,-1)."""
    return load_stl("boolean_subtract"), load_kernel_props("boolean_subtract")


@pytest.fixture
def boolean_union():
    """Union of two 10x5x7 boxes, second offset by 5 in X."""
    return load_stl("boolean_union"), load_kernel_props("boolean_union")


@pytest.fixture
def compound_fillet_shell():
    """10x5x7 box with r=1 fillet on one edge, then shelled (t=0.5, top removed)."""
    return load_stl("compound_fillet_shell"), load_kernel_props("compound_fillet_shell")


@pytest.fixture
def fillet_multi():
    """10x5x7 box with r=1 fillet on three edges."""
    return load_stl("box_fillet_multi"), load_kernel_props("box_fillet_multi")


@pytest.fixture
def extrude_draft():
    """10x5 rectangle extruded 7mm with 5-degree draft angle."""
    return load_stl("box_extrude_draft"), load_kernel_props("box_extrude_draft")


@pytest.fixture
def symmetric_extrude():
    """10x5 rectangle extruded 7mm symmetrically (centered on z=0)."""
    return load_stl("box_symmetric"), load_kernel_props("box_symmetric")


@pytest.fixture
def through_hole():
    """10x5x7 box with 4x2 through-hole along Z."""
    return load_stl("box_through_hole"), load_kernel_props("box_through_hole")


@pytest.fixture
def l_shape():
    """L-shaped profile (10x10 minus 5x5 corner) extruded 5mm."""
    return load_stl("l_shape_extrude"), load_kernel_props("l_shape_extrude")
