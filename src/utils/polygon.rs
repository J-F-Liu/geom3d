use crate::utils::Tolerance;
use crate::Point2;
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
