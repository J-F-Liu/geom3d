use super::Model;
use crate::consts::TAU;
use crate::curve::{Curve, CurveSegment, Polycurve};
use crate::surface::{Surface, SurfacePatch, TrimmedSurface};
use crate::{Float, Grid, KnotVector, Point3, Point4, Vec3, Vec4};
use iso_10303::step::{EntityRef, Real, StepReader};
use iso_10303_parts::ap214::*;
use std::any::Any;

fn vec3(coordinates: &Vec<Real>) -> Vec3 {
    Vec3::new(
        coordinates[0].0 as Float,
        coordinates[1].0 as Float,
        coordinates[2].0 as Float,
    )
}

fn vec4(coordinates: &Vec<Real>, weight: &Real) -> Vec4 {
    let weight = weight.0 as Float;
    Vec4::new(
        (coordinates[0].0 as Float) * weight,
        (coordinates[1].0 as Float) * weight,
        (coordinates[2].0 as Float) * weight,
        weight,
    )
}

fn vertex_point(reader: &Ap214Reader, vertex_ref: &EntityRef) -> Point3 {
    reader
        .get_entity::<VertexPoint>(vertex_ref)
        .and_then(|vertex| reader.get_entity::<CartesianPoint>(vertex.vertex_geometry()))
        .map(|point| vec3(point.coordinates()))
        .unwrap()
}

fn axis1_placement(reader: &Ap214Reader, placement_ref: &EntityRef) -> (Point3, Option<Vec3>) {
    let placement = reader.get_entity::<Axis1Placement>(placement_ref).unwrap();
    let location = reader
        .get_entity::<CartesianPoint>(placement.location())
        .map(|point| vec3(point.coordinates()))
        .unwrap();
    let axis = reader
        .get_entity::<Direction>(placement.axis().as_ref().unwrap())
        .map(|direction| vec3(direction.direction_ratios()));
    (location, axis)
}

fn axis2_placement_3d(
    reader: &Ap214Reader,
    placement_ref: &EntityRef,
) -> (Point3, Option<Vec3>, Option<Vec3>) {
    let placement = reader
        .get_entity::<Axis2Placement3d>(placement_ref)
        .unwrap();
    let location = reader
        .get_entity::<CartesianPoint>(placement.location())
        .map(|point| vec3(point.coordinates()))
        .unwrap();
    let z_axis = reader
        .get_entity::<Direction>(placement.axis().as_ref().unwrap())
        .map(|direction| vec3(direction.direction_ratios()));
    let x_axis = reader
        .get_entity::<Direction>(placement.ref_direction().as_ref().unwrap())
        .map(|direction| vec3(direction.direction_ratios()));
    (location, z_axis, x_axis)
}

fn extract_points(reader: &Ap214Reader, points_list: &Vec<EntityRef>) -> Vec<Point3> {
    points_list
        .iter()
        .map(|point| {
            reader
                .get_entity::<CartesianPoint>(point)
                .map(|point| vec3(point.coordinates()))
                .unwrap()
        })
        .collect::<Vec<_>>()
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
                    .map(|point| vec3(point.coordinates()))
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
                    .map(|point| vec4(point.coordinates(), weight))
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
    // .normalize()
}

fn extract_curve(reader: &Ap214Reader, curve_ref: &EntityRef) -> Option<Box<dyn Curve>> {
    if let Some(line) = reader.get_entity::<Line>(curve_ref) {
        let origin = reader
            .get_entity::<CartesianPoint>(line.pnt())
            .map(|point| vec3(point.coordinates()))
            .unwrap();
        let direction = reader
            .get_entity::<Vector>(line.dir())
            .and_then(|vector| reader.get_entity::<Direction>(vector.orientation()))
            .map(|dir| vec3(dir.direction_ratios()))
            .unwrap();
        return Some(Box::new(crate::curve::Line { origin, direction }));
    }
    if let Some(circle) = reader.get_entity::<Circle>(curve_ref) {
        let (center, axis, ref_dir) = axis2_placement_3d(reader, circle.position());
        return Some(Box::new(crate::curve::Circle {
            center,
            axis: axis.unwrap(),
            ref_dir: ref_dir.unwrap().normalize(),
            radius: circle.radius().0,
        }));
    }
    if let Some(polyline) = reader.get_entity::<Polyline>(curve_ref) {
        let points = extract_points(reader, polyline.points());
        return Some(Box::new(crate::curve::Polyline::new(points)));
    }
    if let Some(bspline) = reader.get_entity::<BSplineCurveWithKnots>(curve_ref) {
        let control_points = extract_points(reader, bspline.control_points_list());
        let knots = extract_knot_vector(bspline.knots(), bspline.knot_multiplicities());
        // let closed = bspline.closed_curve() == Some(true);
        return Some(Box::new(crate::curve::BSplineCurve {
            control_points,
            knots,
            degree: bspline.degree() as u8,
        }));
    }
    let id = curve_ref.0;
    println!("{}: {} is unrecoginzed", id, reader.get_type_name(id));
    None
}

fn extract_edge_curve(
    reader: &Ap214Reader,
    edge_ref: &EntityRef,
) -> Option<CurveSegment<Box<dyn Curve>>> {
    // The curve is implicitly trimmed by the vertices of the edge, this defines the edge domain. Multiple edges can reference the same curve.
    // The sense of an edge is from the edge start vertex to the edge end vertex; the sense of a curve is in the direction of increasing parameter.
    if let Some(edge) = reader.get_entity::<EdgeCurve>(edge_ref) {
        if let Some(curve) = extract_curve(reader, edge.edge_geometry()) {
            let start = vertex_point(reader, edge.edge_start());
            let end = vertex_point(reader, edge.edge_end());
            let u0 = curve.project(start);
            let u1 = curve.project(end);
            let parameter_range = curve.refine_parameter_range((u0, u1), edge.same_sense());
            return Some(CurveSegment {
                curve,
                parameter_range,
                parameter_division: 16,
            });
        }
    }
    None
}

fn extract_surface(
    reader: &Ap214Reader,
    face: &AdvancedFace,
) -> Option<SurfacePatch<Box<dyn Surface>>> {
    // whether the sense of the surface normal agrees with the sense of the topological normal to the face
    let same_sense = face.same_sense();
    if let Some(plane) = reader.get_entity::<Plane>(face.face_geometry()) {
        let (origin, z_axis, x_axis) = axis2_placement_3d(reader, plane.position());
        let normal = if same_sense {
            z_axis.unwrap()
        } else {
            -z_axis.unwrap()
        };
        let u_axis = x_axis.unwrap();
        let surface = crate::surface::Plane {
            origin,
            normal,
            u_axis,
            v_axis: normal.cross(u_axis),
        };
        return Some(SurfacePatch {
            surface: Box::new(surface) as Box<dyn Surface>,
            parameter_range: ((0.0, 1.0), (0.0, 1.0)),
            parameter_division: (16, 16),
        });
    }
    if let Some(cylinder) = reader.get_entity::<CylindricalSurface>(face.face_geometry()) {
        let (origin, axis, ref_dir) = axis2_placement_3d(reader, cylinder.position());
        let surface = crate::surface::Cylinder {
            origin,
            axis: axis.unwrap(),
            ref_dir: ref_dir.unwrap().normalize(),
            radius: cylinder.radius().0,
        };
        return Some(SurfacePatch {
            surface: Box::new(surface) as Box<dyn Surface>,
            parameter_range: ((0.0, 1.0), (0.0, TAU)),
            parameter_division: (16, 16),
        });
    }
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
    if let Some(revolution) = reader.get_entity::<SurfaceOfRevolution>(face.face_geometry()) {
        if let Some(section) = extract_curve(reader, revolution.swept_curve()) {
            let (origin, axis) = axis1_placement(reader, revolution.axis_position());
            let surface = crate::surface::SpinSurface {
                origin,
                axis: axis.unwrap(),
                section,
            };
            return Some(SurfacePatch {
                surface: Box::new(surface) as Box<dyn Surface>,
                parameter_range: ((0.0, 1.0), (0.0, TAU)),
                parameter_division: (16, 16),
            });
        }
    }
    return None;
}

fn get_trimming_parameter_value(measure: &TrimmingSelect) -> Option<f64> {
    if let TrimmingSelect::ParameterValue(value) = measure {
        Some(value.0)
    } else {
        None
    }
}

fn extract_curve_segment(
    reader: &Ap214Reader,
    segment_ref: &EntityRef,
) -> Option<CurveSegment<Box<dyn Curve>>> {
    if let Some(segment) = reader.get_entity::<CompositeCurveSegment>(segment_ref) {
        if let Some(trimmed_curve) = reader.get_entity::<TrimmedCurve>(segment.parent_curve()) {
            if let Some(basis_curve) = extract_curve(reader, trimmed_curve.basis_curve()) {
                let param1 =
                    get_trimming_parameter_value(trimmed_curve.trim_1().iter().next().unwrap())
                        .unwrap();
                let param2 =
                    get_trimming_parameter_value(trimmed_curve.trim_2().iter().next().unwrap())
                        .unwrap();
                let parameter_range = basis_curve
                    .refine_parameter_range((param1, param2), trimmed_curve.sense_agreement());
                return Some(CurveSegment {
                    curve: basis_curve,
                    parameter_range,
                    parameter_division: 16,
                });
            }
        }
    }
    return None;
}

pub struct ModelReader {}
pub type StepModel = Model<TrimmedSurface<Box<dyn Surface>>>;

impl ModelReader {
    pub fn read_model<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<StepModel> {
        let mut reader = Ap214Reader::new();
        reader.read(file)?;

        let mut model = Model::new();
        for composite_curve in reader.get_entities::<CompositeCurve>() {
            let segments = composite_curve
                .segments()
                .iter()
                .filter_map(|segment| extract_curve_segment(&reader, segment))
                .collect();
            model.add_curve(Polycurve { segments });
        }
        for advanced_face in reader.get_entities::<AdvancedFace>() {
            if let Some(surface) = extract_surface(&reader, advanced_face) {
                let mut edges = Vec::new();
                for bound in advanced_face.bounds() {
                    if let Some(face_bound) = reader.get_entity::<FaceBound>(bound) {
                        if let Some(edge_loop) = reader.get_entity::<EdgeLoop>(face_bound.bound()) {
                            let edge_list = edge_loop.edge_list().iter().filter_map(|edge| {
                                reader.get_entity::<OrientedEdge>(edge).and_then(|edge| {
                                    extract_edge_curve(&reader, edge.edge_element())
                                })
                            });
                            edges.extend(edge_list);
                        }
                    }
                }
                let trimmed_surface = TrimmedSurface {
                    surface: surface.surface,
                    edges,
                };
                model.add_face(trimmed_surface);
            } else {
                let id = advanced_face.face_geometry().0;
                println!("{}: {} is unrecoginzed", id, reader.get_type_name(id));
            }
        }
        Ok(model)
    }
}
