use crate::{Float, Point3};
use approx::{ulps_eq, AbsDiffEq};

/// general tolerance
pub const TOLERANCE: Float = 1.0e-7;

/// general tolerance of square order
pub const TOLERANCE2: Float = TOLERANCE * TOLERANCE;

/// Defines a tolerance in the whole package
pub trait Tolerance: AbsDiffEq<Epsilon = Float> + Sized {
    /// The "distance" is less than `TOLERANCE`.
    fn near(&self, other: Self) -> bool {
        self.abs_diff_eq(&other, TOLERANCE)
    }

    /// The "distance" is less than `TOLERANCR2`.
    fn near2(&self, other: Self) -> bool {
        self.abs_diff_eq(&other, TOLERANCE2)
    }
}

impl Tolerance for Float {}

pub fn inv_or_zero(delta: Float) -> Float {
    if ulps_eq!(delta, 0.0) {
        0.0
    } else {
        1.0 / delta
    }
}

/// Create parameter values by uniformly divide a range
/// # Examples
/// ```
/// use geom3d::utils::*;
/// assert_eq!(
///     uniform_divide((0.0, 1.0), 8),
///     vec![0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0,]
/// );
/// assert_eq!(
///     uniform_divide((1.0, 0.0), 8),
///     vec![1.0, 0.875, 0.75, 0.625, 0.5, 0.375, 0.25, 0.125, 0.0,]
/// );
/// ```
pub fn uniform_divide(range: (Float, Float), division: usize) -> Vec<Float> {
    let (begin, end) = range;
    let step = (end - begin) / division as Float;
    let mut parameters = Vec::with_capacity(division + 1);
    parameters.push(begin);
    let mut u = begin;
    for _ in 0..division - 1 {
        u += step;
        parameters.push(u);
    }
    parameters.push(end);
    parameters
}

pub fn range_at((start, end): (Float, Float), ratio: Float) -> Float {
    start + (end - start) * ratio
}

pub fn find_nearest_point(points: &[Point3], point: Point3) -> usize {
    let mut min = Float::MAX;
    let mut min_index = 0;

    for (index, &vertex) in points.iter().enumerate() {
        let distance = (point - vertex).length_squared();
        if distance < min {
            min = distance;
            min_index = index;
        }
    }

    min_index
}

pub fn distance_to_line_segment(a: Point3, b: Point3, p: Point3) -> Float {
    let ap = p - a;
    let ab = b - a;
    let product = ap.dot(ab);
    if product <= 0.0 {
        return ap.length();
    }
    if product >= ab.length_squared() {
        let bp = p - b;
        return bp.length();
    }
    return ap.cross(ab).length() / ab.length();
}

use crate::curve::Curve;

pub fn find_nearest_parameter(
    curve: &impl Curve,
    der1: &impl Curve,
    der2: &impl Curve,
    point: Point3,
    parameters: &[Float],
    trials: usize,
) -> Float {
    // compute initial approximate value
    let mut u = parameters[find_nearest_point(
        &parameters
            .iter()
            .map(|&u| curve.get_point(u))
            .collect::<Vec<_>>(),
        point,
    )];
    // use Newton iteration to minimize the distance between point and curve
    for _ in 0..trials {
        let delta = curve.get_point(u) - point;
        if delta.length_squared().near(0.0) {
            return u;
        }
        let tangent = der1.get_point(u);
        let f = tangent.dot(delta);
        if f.near(0.0) {
            return u;
        }
        let fprime = der2.get_point(u).dot(delta) + tangent.length_squared();
        u -= f / fprime;
    }
    u
}
