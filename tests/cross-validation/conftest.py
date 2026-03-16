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
