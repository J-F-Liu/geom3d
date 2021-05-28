use super::Model;
use crate::surface::{Surface, SurfacePatch};
use crate::{Float, Grid, KnotVector, Point3, Point4};
use iso_10303::step::{EntityRef, Real, StepReader};
use iso_10303_parts::ap214::*;
use std::any::Any;

fn point3(coordinates: &Vec<Real>) -> Point3 {
    Point3::new(
        coordinates[0].0 as Float,
        coordinates[1].0 as Float,
        coordinates[2].0 as Float,
    )
}

fn point4(coordinates: &Vec<Real>, weight: &Real) -> Point4 {
    let weight = weight.0 as Float;
    Point4::new(
        (coordinates[0].0 as Float) * weight,
        (coordinates[1].0 as Float) * weight,
        (coordinates[2].0 as Float) * weight,
        weight,
    )
}

fn extract_control_points(
    reader: &Ap214Reader,
    control_points_list: &Vec<Vec<EntityRef>>,
) -> Vec<Point3> {
    control_points_list
        .iter()
        .map(|row| {
            row.iter().map(|point| {
                reader
                    .get_entity::<CartesianPoint>(point)
                    .map(|point| point3(point.coordinates()))
                    .unwrap()
            })
        })
        .flatten()
        .collect()
}

fn extract_weighted_control_points(
    reader: &Ap214Reader,
    control_points_list: &Vec<Vec<EntityRef>>,
    weights_data: &Vec<Vec<Real>>,
) -> Vec<Point4> {
    control_points_list
        .iter()
        .zip(weights_data.iter())
        .map(|(points, weights)| {
            points.iter().zip(weights.iter()).map(|(point, weight)| {
                reader
                    .get_entity::<CartesianPoint>(point)
                    .map(|point| point4(point.coordinates(), weight))
                    .unwrap()
            })
        })
        .flatten()
        .collect()
}

fn extract_knot_vector(knots: &Vec<Real>, multiplicities: &Vec<i64>) -> KnotVector {
    KnotVector::from_values_and_multiplicities(
        knots.iter().map(|value| value.0 as Float).collect(),
        multiplicities.iter().map(|&value| value as usize).collect(),
    )
    .normalize()
}

fn extract_surface(
    reader: &Ap214Reader,
    face: &AdvancedFace,
) -> Option<SurfacePatch<Box<dyn Surface>>> {
    if let Some(bezier_surface) = reader.get_entity::<BezierSurface>(face.face_geometry()) {
        let control_points = extract_control_points(reader, bezier_surface.control_points_list());
        let surface = crate::surface::BezierSurface::new(Grid::from_vec(
            control_points,
            bezier_surface.control_points_list()[0].len(),
        ));
        return Some(SurfacePatch {
            surface: Box::new(surface) as Box<dyn Surface>,
            parameter_range: ((0.0, 1.0), (0.0, 1.0)),
            parameter_division: (16, 16),
        });
    }
    if let Some(bspline_surface) = reader.get_entity::<BSplineSurface>(face.face_geometry()) {
        let control_points = extract_control_points(reader, bspline_surface.control_points_list());
        let degree = (
            bspline_surface.u_degree() as usize,
            bspline_surface.v_degree() as usize,
        );
        let surface = crate::surface::BSplineSurface::uniform_clamped(
            Grid::from_vec(
                control_points,
                bspline_surface.control_points_list()[0].len(),
            ),
            degree,
        );
        return Some(SurfacePatch {
            surface: Box::new(surface) as Box<dyn Surface>,
            parameter_range: ((0.0, 1.0), (0.0, 1.0)),
            parameter_division: (16, 16),
        });
    }
    if let Some(bspline_surface) = reader.get_entity::<RationalBSplineSurface>(face.face_geometry())
    {
        let control_points = extract_weighted_control_points(
            reader,
            bspline_surface.control_points_list(),
            bspline_surface.weights_data(),
        );
        let degree = (
            bspline_surface.u_degree() as usize,
            bspline_surface.v_degree() as usize,
        );
        let surface = crate::surface::BSplineSurface::uniform_clamped(
            Grid::from_vec(
                control_points,
                bspline_surface.control_points_list()[0].len(),
            ),
            degree,
        );
        return Some(SurfacePatch {
            surface: Box::new(surface) as Box<dyn Surface>,
            parameter_range: ((0.0, 1.0), (0.0, 1.0)),
            parameter_division: (16, 16),
        });
    }
    if let Some(bspline_surface) =
        reader.get_entity::<BSplineSurfaceWithKnots>(face.face_geometry())
    {
        let control_points = extract_control_points(reader, bspline_surface.control_points_list());
        let u_knots = extract_knot_vector(
            bspline_surface.u_knots(),
            bspline_surface.u_multiplicities(),
        );
        let v_knots = extract_knot_vector(
            bspline_surface.v_knots(),
            bspline_surface.v_multiplicities(),
        );
        let degree = (
            bspline_surface.u_degree() as usize,
            bspline_surface.v_degree() as usize,
        );
        let surface = crate::surface::BSplineSurface::new(
            Grid::from_vec(
                control_points,
                bspline_surface.control_points_list()[0].len(),
            ),
            (u_knots, v_knots),
            degree,
        );
        return Some(SurfacePatch {
            surface: Box::new(surface) as Box<dyn Surface>,
            parameter_range: ((0.0, 1.0), (0.0, 1.0)),
            parameter_division: (16, 16),
        });
    }
    if let Some(surfaces) = reader.get_entity::<Vec<Box<dyn Any>>>(face.face_geometry()) {
        let mut control_points_list = None;
        let mut degree = None;
        let mut u_knots = None;
        let mut v_knots = None;
        let mut weighted_control_points = None;
        for surface in surfaces {
            if let Some(bspline_surface) = surface.downcast_ref::<BSplineSurface>() {
                control_points_list = Some(bspline_surface.control_points_list());
                degree = Some((
                    bspline_surface.u_degree() as usize,
                    bspline_surface.v_degree() as usize,
                ));
            }
            if let Some(bspline_surface) = surface.downcast_ref::<BSplineSurfaceWithKnots>() {
                u_knots = Some(extract_knot_vector(
                    bspline_surface.u_knots(),
                    bspline_surface.u_multiplicities(),
                ));
                v_knots = Some(extract_knot_vector(
                    bspline_surface.v_knots(),
                    bspline_surface.v_multiplicities(),
                ));
            }
            if let Some(bspline_surface) = surface.downcast_ref::<RationalBSplineSurface>() {
                weighted_control_points = Some(extract_weighted_control_points(
                    reader,
                    control_points_list.unwrap(),
                    bspline_surface.weights_data(),
                ));
            }
        }
        if let Some(weighted_control_points) = weighted_control_points {
            if let (Some(u_knots), Some(v_knots)) = (u_knots, v_knots) {
                let surface = crate::surface::BSplineSurface::new(
                    Grid::from_vec(
                        weighted_control_points,
                        control_points_list.unwrap()[0].len(),
                    ),
                    (u_knots, v_knots),
                    degree.unwrap(),
                );
                return Some(SurfacePatch {
                    surface: Box::new(surface) as Box<dyn Surface>,
                    parameter_range: ((0.0, 1.0), (0.0, 1.0)),
                    parameter_division: (16, 16),
                });
            }
        }
    }
    return None;
}

pub struct ModelReader {}

impl ModelReader {
    pub fn read_model<P: AsRef<std::path::Path>>(
        file: P,
    ) -> std::io::Result<Model<Box<dyn Surface>>> {
        let mut reader = Ap214Reader::new();
        reader.read(file)?;

        let mut model = Model::new();
        for advanced_face in reader.get_entities::<AdvancedFace>() {
            // let id = advanced_face.face_geometry().0;
            // println!("{}: {}", id, reader.get_type_name(id));
            if let Some(surface) = extract_surface(&reader, advanced_face) {
                model.add_surface(surface);
            }
        }
        Ok(model)
    }
}
