use fj_math::{Scalar, Triangle, Winding};

use itertools::Itertools;
use spade::HasPosition;

use crate::geometry;

/// Create a Delaunay triangulation of all points
pub fn triangulate(
    points: Vec<geometry::Point<2, 3>>,
) -> Vec<[geometry::Point<2, 3>; 3]> {
    use spade::Triangulation as _;

    let triangulation = spade::DelaunayTriangulation::<_>::bulk_load(points)
        .expect("Inserted invalid values into triangulation");

    let mut triangles = Vec::new();
    for triangle in triangulation.inner_faces() {
        let [v0, v1, v2] = triangle.vertices().map(|vertex| *vertex.data());
        let orientation =
            Triangle::<2>::from_points([v0.local(), v1.local(), v2.local()])
                .winding_direction();

        let triangle = match orientation {
            Winding::Ccw => [v0, v1, v2],
            Winding::Cw => [v0, v2, v1],
        };

        triangles.push(triangle);
    }

    triangles
}

/// Create a constrained Delaunay triangulation of all points
pub fn triangulate_constrained(
    points: Vec<geometry::Point<2, 3>>,
    exterior: Vec<geometry::Point<2, 3>>,
    interiors: Vec<Vec<geometry::Point<2, 3>>>,
) -> Vec<[geometry::Point<2, 3>; 3]> {
    use spade::Triangulation as _;

    let mut triangulation =
        spade::ConstrainedDelaunayTriangulation::<_>::bulk_load(points)
            .expect("Inserted invalid values into triangulation");
    for (a, b) in exterior.into_iter().tuple_windows() {
        let from_handle = triangulation.insert(a).unwrap();
        let to_handle = triangulation.insert(b).unwrap();
        if !triangulation.can_add_constraint(from_handle, to_handle) {
            triangulation.add_constraint_edge(a, b).unwrap();
        }
    }
    for interior in interiors {
        for (a, b) in interior.into_iter().tuple_windows() {
            let from_handle = triangulation.insert(a).unwrap();
            let to_handle = triangulation.insert(b).unwrap();
            if !triangulation.can_add_constraint(from_handle, to_handle) {
                triangulation.add_constraint_edge(a, b).unwrap();
            }
        }
    }
    let mut vertices_removed = false;
    let temp_triangulation = triangulation.clone();
    let faces_to_remove = temp_triangulation
        .inner_faces()
        .into_iter()
        .filter(|tris| tris.area() < Scalar::from(0.0001_f64));
    for face in faces_to_remove.rev() {
        triangulation.remove(face.vertices()[1].fix());
        vertices_removed = true;
    }

    if vertices_removed {}

    let mut triangles = Vec::new();

    for triangle in triangulation.inner_faces() {
        let [v0, v1, v2] = triangle.vertices().map(|vertex| *vertex.data());
        let orientation =
            Triangle::<2>::from_points([v0.local(), v1.local(), v2.local()])
                .winding_direction();

        let triangle = match orientation {
            Winding::Ccw => [v0, v1, v2],
            Winding::Cw => [v0, v2, v1],
        };

        triangles.push(triangle);
    }

    triangles
}

// Enables the use of `geometry::Point` in the triangulation.
impl HasPosition for geometry::Point<2, 3> {
    type Scalar = Scalar;

    fn position(&self) -> spade::Point2<Self::Scalar> {
        spade::Point2 {
            x: self.local().u,
            y: self.local().v,
        }
    }
}
