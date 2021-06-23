use crate::utils::Tolerance;
use crate::{Float, Point2};
use std::collections::VecDeque;

pub fn compute_vertex_convexity(points: &[Point2]) -> (VecDeque<usize>, Vec<usize>) {
    let n = points.len();
    let mut vertices = VecDeque::with_capacity(n);
    let mut concave_points = Vec::new();
    for i in 0..n {
        match compute_convexity(
            points[(n + i - 1).rem_euclid(n)],
            points[i],
            points[(i + 1).rem_euclid(n)],
        ) {
            Convexity::Convex => vertices.push_back(i),
            Convexity::Concave => {
                vertices.push_back(i);
                concave_points.push(i);
            }
            Convexity::Colinear => {} // skip colinear or duplicate point
        }
    }

    (vertices, concave_points)
}

pub fn find_concave_vertices(points: &[Point2], vertices: &mut VecDeque<usize>) -> Vec<usize> {
    let n = vertices.len();
    let mut to_delete = vec![];
    let mut concave_points = Vec::new();
    for i in 0..n {
        match compute_convexity(
            points[vertices[(n + i - 1).rem_euclid(n)]],
            points[vertices[i]],
            points[vertices[(i + 1).rem_euclid(n)]],
        ) {
            Convexity::Convex => {}
            Convexity::Concave => {
                concave_points.push(vertices[i]);
            }
            Convexity::Colinear => {
                // remove colinear or duplicate point
                to_delete.push(i);
            }
        }
    }
    to_delete.reverse();
    for j in to_delete {
        vertices.remove(j);
    }
    concave_points.sort();
    concave_points
}

/// Triangulation by Ear Clipping
pub fn trianglate_polygon(
    points: &[Point2],
    mut vertices: VecDeque<usize>,
    mut concave_points: Vec<usize>,
) -> Vec<u32> {
    let n = vertices.len();
    let mut triangles = Vec::with_capacity((n - 2) * 3);
    let mut prev_m = usize::MAX;

    loop {
        if vertices.len() >= prev_m {
            break;
        }
        let mut m = vertices.len();
        prev_m = m;
        if m < 3 {
            break;
        } else if m == 3 {
            triangles.push(vertices[0] as u32);
            triangles.push(vertices[1] as u32);
            triangles.push(vertices[2] as u32);
            break;
        }

        let mut i = 0;
        while i < m {
            let curr = vertices[i];
            if concave_points.binary_search(&curr).is_err() {
                let prev = vertices[(m + i - 1).rem_euclid(m)];
                let next = vertices[(i + 1).rem_euclid(m)];
                if is_ear(points, &concave_points, prev, curr, next) {
                    triangles.push(prev as u32);
                    triangles.push(curr as u32);
                    triangles.push(next as u32);

                    let mut to_delete = vec![i];

                    // update concave_points
                    if let Ok(j) = concave_points.binary_search(&prev) {
                        let prev_prev = vertices[(m + i - 2).rem_euclid(m)];
                        match compute_convexity(points[prev_prev], points[prev], points[next]) {
                            Convexity::Convex => {
                                concave_points.remove(j);
                            }
                            Convexity::Colinear => {
                                to_delete.push((m + i - 1).rem_euclid(m));
                            }
                            _ => {}
                        }
                    }
                    if let Ok(j) = concave_points.binary_search(&next) {
                        let next_next = vertices[(i + 2).rem_euclid(m)];
                        match compute_convexity(points[prev], points[next], points[next_next]) {
                            Convexity::Convex => {
                                concave_points.remove(j);
                            }
                            Convexity::Colinear => {
                                to_delete.push((i + 1).rem_euclid(m));
                            }
                            _ => {}
                        }
                    }

                    // drop ear
                    to_delete.sort();
                    to_delete.reverse();
                    m -= to_delete.len();
                    for j in to_delete {
                        vertices.remove(j);
                    }
                }
            }
            i += 1;
        }
    }
    triangles
}

#[derive(Debug)]
enum Convexity {
    Convex,
    Colinear,
    Concave,
}

// check if b is a convex vertex
fn compute_convexity(a: Point2, b: Point2, c: Point2) -> Convexity {
    let product = (b - a).perp_dot(c - b);
    if product.near(0.0) {
        Convexity::Colinear
    } else if product > 0.0 {
        Convexity::Convex
    } else {
        Convexity::Concave
    }
}

fn is_inside_triangle(a: Point2, b: Point2, c: Point2, p: Point2) -> bool {
    (a - p).perp_dot(b - p) >= 0.0
        && (b - p).perp_dot(c - p) >= 0.0
        && (c - p).perp_dot(a - p) >= 0.0
}

fn is_ear(
    points: &[Point2],
    concave_points: &[usize],
    prev: usize,
    curr: usize,
    next: usize,
) -> bool {
    for &other in concave_points {
        if other != prev && other != next {
            if is_inside_triangle(points[prev], points[curr], points[next], points[other]) {
                return false;
            }
        }
    }
    return true;
}

pub fn merge_polygons(points: &[Point2], polygons: &[usize]) -> VecDeque<usize> {
    // find points with maximum x
    let mut sorted_polygons = Vec::with_capacity(polygons.len());
    for pair in polygons.windows(2) {
        if let [start, end] = pair {
            let mut x_max = Float::MIN;
            let mut i_max = 0;
            for index in *start..*end {
                let point = points[index];
                if point.x > x_max {
                    x_max = point.x;
                    i_max = index;
                }
            }
            sorted_polygons.push((*start, *end, i_max, x_max));
        }
    }
    sorted_polygons.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());

    // recursively merge inner polygons to outer polygon
    let (outer_start, outer_end, _, _) = sorted_polygons.pop().unwrap();
    let mut vertices = VecDeque::with_capacity(points.len() + polygons.len() - 1);
    vertices.extend(outer_start..outer_end);
    for (start, end, i_max, _) in sorted_polygons.into_iter().rev() {
        merge_two_polygons(points, &mut vertices, start..end, i_max);
    }
    vertices
}

fn merge_two_polygons(
    points: &[Point2],
    outer: &mut VecDeque<usize>,
    inner: std::ops::Range<usize>,
    max_x_index: usize,
) {
    let inner_point = points[max_x_index];
    // find nearest edge to inner point
    let mut min = Float::MAX;
    let mut min_index = 0;
    for i in 0..outer.len() {
        let distance = distance_to_line_segment(
            points[outer[i]],
            points[outer[(i + 1).rem_euclid(outer.len())]],
            inner_point,
        );
        if distance < min {
            min = distance;
            min_index = i;
        }
    }
    // insert inner polygon at min_index
    let insert_at = min_index + 1;
    outer.insert(insert_at, outer[min_index]);
    for index in (inner.start..=max_x_index).rev() {
        outer.insert(insert_at, index);
    }
    for index in (max_x_index..inner.end).rev() {
        outer.insert(insert_at, index);
    }
}

pub fn distance_to_line_segment(a: Point2, b: Point2, p: Point2) -> Float {
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
    return ap.perp_dot(ab).abs() / ab.length();
}

#[test]
fn test_compute_convexity() {
    let a = Point2::new(7.46852, 258.22396);
    let b = Point2::new(7.51861, 258.22151);
    let c = Point2::new(7.56870, 258.21901);
    dbg!((b - a).perp_dot(c - b));
    dbg!(compute_convexity(a, b, c));
}
