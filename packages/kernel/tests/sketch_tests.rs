use blockcad_kernel::geometry::Pt2;
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;

#[test]
fn create_sketch_with_line() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(10.0, 0.0),
    });
    let _line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

    assert_eq!(sketch.entity_count(), 3);
    assert_eq!(sketch.constraint_count(), 0);
}

#[test]
fn add_constraints_to_sketch() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p1 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(0.0, 0.0),
    });
    let p2 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(5.0, 5.0),
    });
    let p3 = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(10.0, 0.0),
    });

    // Fix first point
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
    // Distance between p1 and p2
    sketch.add_constraint(Constraint::new(
        ConstraintKind::Distance { value: 7.07 },
        vec![p1, p2],
    ));
    // Horizontal constraint on p2-p3
    let _l = sketch.add_entity(SketchEntity::Line {
        start: p2,
        end: p3,
    });

    assert_eq!(sketch.constraint_count(), 2);
}

#[test]
fn sketch_circle_entity() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let center = sketch.add_entity(SketchEntity::Point {
        position: Pt2::new(5.0, 5.0),
    });
    let _circle = sketch.add_entity(SketchEntity::Circle {
        center,
        radius: 3.0,
    });
    assert_eq!(sketch.entity_count(), 2);
}
