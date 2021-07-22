use crate::surface::{EdgeLoop, Surface};
use crate::{
    face::Face, utils, utils::Tolerance, Float, Grid, KnotVector, Mat2, Point2, Point3, Point4,
    TriangleMesh, Vec2, Vec3,
};

#[derive(Debug, Clone)]
pub struct BSplineSurface<P> {
    pub control_points: Grid<P>,
    pub knots: (KnotVector, KnotVector),
    pub degree: (u8, u8),
}

impl<P: Clone> BSplineSurface<P> {
    pub fn new(
        control_points: Grid<P>,
        knots: (KnotVector, KnotVector),
        degree: (usize, usize),
    ) -> Self {
        let (u_deg, v_deg) = degree;
        assert_eq!(control_points.rows() + u_deg + 1, knots.0.len());
        assert_eq!(control_points.cols() + v_deg + 1, knots.1.len());
        Self {
            control_points,
            knots,
            degree: (u_deg as u8, v_deg as u8),
        }
    }

    pub fn uniform_clamped(control_points: Grid<P>, degree: (usize, usize)) -> Self {
        let (u_deg, v_deg) = degree;
        assert!(control_points.rows() > u_deg && control_points.cols() > v_deg);

        let u_knots = KnotVector::uniform_knot(u_deg, control_points.rows() - u_deg);
        let v_knots = KnotVector::uniform_knot(v_deg, control_points.cols() - v_deg);
        Self {
            control_points,
            knots: (u_knots, v_knots),
            degree: (u_deg as u8, v_deg as u8),
        }
    }
}

impl<P: std::ops::Sub<Output = P> + std::ops::Mul<Float, Output = P> + Copy + Default>
    BSplineSurface<P>
{
    // The derivative by u of a (p,q) degree B-Spline surface is a (p - 1, q) degree B-Spline surface
    pub fn derivative_u(&self) -> BSplineSurface<P> {
        let (p, q) = self.degree;
        if p == 0 {
            return BSplineSurface::uniform_clamped(
                Grid::from_vec(vec![P::default(); 4], 2),
                (1, 1),
            );
        }
        let degree = (p - 1, q);
        let (u_knots, v_knots) = self.knots.clone();
        let u_knots = u_knots.shrink();
        let u_spans = u_knots.spans(p as usize);
        let p = p as Float;
        let (n, m) = self.control_points.size();
        let mut points = Vec::with_capacity((n - 1) * m);
        for i in 0..n - 1 {
            for j in 0..m {
                let point = self.control_points[i + 1][j] - self.control_points[i][j];
                let span = u_spans[i];
                points.push(point * p * utils::inv_or_zero(span));
            }
        }
        let control_points = Grid::from_vec(points, m);
        BSplineSurface {
            control_points,
            knots: (u_knots, v_knots),
            degree,
        }
    }
    // The derivative by v of a (p,q) degree B-Spline surface is a (p, q - 1) degree B-Spline surface
    pub fn derivative_v(&self) -> BSplineSurface<P> {
        let (p, q) = self.degree;
        if q == 0 {
            return BSplineSurface::uniform_clamped(
                Grid::from_vec(vec![P::default(); 4], 2),
                (1, 1),
            );
        }
        let degree = (p, q - 1);
        let (u_knots, v_knots) = self.knots.clone();
        let v_knots = v_knots.shrink();
        let v_spans = v_knots.spans(q as usize);
        let q = q as Float;
        let (n, m) = self.control_points.size();
        let mut points = Vec::with_capacity(n * (m - 1));
        for i in 0..n {
            for j in 0..m - 1 {
                let point = self.control_points[i][j + 1] - self.control_points[i][j];
                let span = v_spans[j];
                points.push(point * q * utils::inv_or_zero(span));
            }
        }
        let control_points = Grid::from_vec(points, m - 1);
        BSplineSurface {
            control_points,
            knots: (u_knots, v_knots),
            degree,
        }
    }
}

impl BSplineSurface<Point3> {
    pub fn project_points(&self, points: &[Point3]) -> Vec<Point2> {
        let der_u = self.derivative_u();
        let der_v = self.derivative_v();
        let der_uu = der_u.derivative_u();
        let der_uv = der_u.derivative_v();
        let der_vu = der_v.derivative_u();
        let der_vv = der_v.derivative_v();

        let (n, m) = self.control_points.size();
        let (u_range, v_range) = (self.knots.0.range(), self.knots.1.range());
        let u_parameters = utils::uniform_divide(u_range, n * 4);
        let v_parameters = utils::uniform_divide(v_range, m * 4);
        let vertices = Grid::from_vec(
            u_parameters
                .iter()
                .map(|&u| v_parameters.iter().map(move |&v| self.get_point(u, v)))
                .flatten()
                .collect::<Vec<Point3>>(),
            m * 4 + 1,
        );

        let trials = 20;
        points
            .iter()
            .map(|point| {
                let (i, j) = utils::find_nearest_point_in_grid(&vertices, *point);
                let (mut u, mut v) = (u_parameters[i], v_parameters[j]);
                for _ in 0..trials {
                    let r = self.get_point(u, v) - *point;
                    if r.length_squared().near(0.0) {
                        return Point2::new(u, v);
                    }
                    let su = der_u.get_point(u, v);
                    let sv = der_v.get_point(u, v);
                    if su.dot(r).near(0.0) && sv.dot(r).near(0.0) {
                        return Point2::new(u, v);
                    }

                    let suu = der_uu.get_point(u, v);
                    let suv = der_uv.get_point(u, v);
                    let svu = der_vu.get_point(u, v);
                    let svv = der_vv.get_point(u, v);
                    let fu = su.length_squared() + r.dot(suu);
                    let gu = su.dot(sv) + r.dot(svu);
                    let fv = su.dot(sv) + r.dot(suv);
                    let gv = sv.length_squared() + r.dot(svv);

                    let j = Mat2::from_cols_array(&[fu, gu, fv, gv]);
                    let k = Vec2::new(-r.dot(su), -r.dot(sv));
                    let delta = j.inverse() * k;
                    u = utils::clamp_in_range(u + delta.x, u_range);
                    v = utils::clamp_in_range(v + delta.y, v_range);
                }
                return Point2::new(u, v);
            })
            .collect::<Vec<_>>()
    }
}

/// 3D BSpline Surface
impl Surface for BSplineSurface<Point3> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        let (u_knots, v_knots) = &self.knots;
        let (p, q) = self.degree;
        let basis_u = u_knots.bspline_basis(p as usize, u);
        let basis_v = v_knots.bspline_basis(q as usize, v);
        let (n, m) = self.control_points.size();
        let mut point = Point3::ZERO;
        for i in 0..n {
            for j in 0..m {
                let p = self.control_points[i][j];
                point += basis_u[i] * basis_v[j] * p;
            }
        }
        point
    }

    fn get_normals(&self, params: &[Point2]) -> Vec<Vec3> {
        let der_u = self.derivative_u();
        let der_v = self.derivative_v();

        params
            .iter()
            .map(|p| {
                der_u
                    .get_point(p.x, p.y)
                    .cross(der_v.get_point(p.x, p.y))
                    .normalize()
            })
            .collect()
    }

    fn trim(&self, bounds: &[EdgeLoop]) -> TriangleMesh {
        // let patch = crate::surface::SurfacePatch {
        //     surface: self.clone(),
        //     parameter_range: (self.knots.0.range(), self.knots.1.range()),
        //     parameter_division: (16, 16),
        // };

        // patch.get_points().into()
        let mut polygons = Vec::with_capacity(bounds.len() + 1);
        polygons.push(0);
        let mut end = 0;
        let mut vertices = bounds
            .iter()
            .map(|bound| {
                let polygon = bound.to_polygon();
                if polygon.len() > 0 {
                    end += polygon.len();
                    polygons.push(end);
                }
                polygon
            })
            .flatten()
            .collect::<Vec<_>>();
        save_bound_as_obj(&vertices, "bound.obj").unwrap();
        let mut points: Vec<Point2> = self.project_points(&vertices);
        let boundary_point_count = points.len();

        if polygons.len() == 2 {
            if utils::is_polygon_counter_clockwise(&points) {
                let (points, triangles) = utils::generate_triangular_mesh(&points, &polygons);
                vertices.extend(
                    points[boundary_point_count..]
                        .iter()
                        .map(|&p| self.get_point(p.x, p.y)),
                );

                return TriangleMesh {
                    vertices,
                    triangles,
                    normals: Vec::new(),
                }
                .reverse_winding_direction();
            } else {
                vertices.reverse();
                points.reverse();
                let (points, triangles) = utils::generate_triangular_mesh(&points, &polygons);
                vertices.extend(
                    points[boundary_point_count..]
                        .iter()
                        .map(|&p| self.get_point(p.x, p.y)),
                );

                return TriangleMesh {
                    vertices,
                    triangles,
                    normals: Vec::new(),
                };
            }
        } else if polygons.len() > 2 {
            // triangulate polygon with holes
            dbg!(polygons.len());
            let (points, triangles) = utils::generate_triangular_mesh(&points, &polygons);
            vertices.extend(
                points[boundary_point_count..]
                    .iter()
                    .map(|p| self.get_point(p.x, p.y)),
            );
            return TriangleMesh {
                vertices,
                triangles,
                normals: Vec::new(),
            };
        } else {
            return TriangleMesh::new();
        }
    }
}

/// Rational BSpline Surface, point (x,y,z) with weight w is (wx,wy,wz,w)
impl Surface for BSplineSurface<Point4> {
    fn get_point(&self, u: Float, v: Float) -> Point3 {
        let (u_knots, v_knots) = &self.knots;
        let (p, q) = self.degree;
        let basis_u = u_knots.bspline_basis(p as usize, u);
        let basis_v = v_knots.bspline_basis(q as usize, v);
        let (n, m) = self.control_points.size();
        let mut point = Point4::ZERO;
        for i in 0..n {
            for j in 0..m {
                let p = self.control_points[i][j];
                point += basis_u[i] * basis_v[j] * p;
            }
        }
        (1.0 / point.w) * point.truncate()
    }
}

pub fn save_bound_as_obj<P: AsRef<std::path::Path>>(
    vertices: &[Point3],
    filename: P,
) -> std::io::Result<()> {
    use std::io::Write;
    let file = std::fs::File::create(filename)?;
    let mut writer = std::io::LineWriter::new(file);

    for point in vertices {
        writeln!(writer, "v {} {} {}", point.x, point.y, point.z)?;
    }
    for i in 1..vertices.len() {
        writeln!(writer, "l {} {}", i, i + 1)?;
    }
    writeln!(writer, "l {} {}", vertices.len(), 1)?;
    Ok(())
}
