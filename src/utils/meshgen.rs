use crate::consts::HALF_SQRT_3;
use crate::{utils::Point, Float, Point2};
use std::collections::VecDeque;

/// generate triangular mesh by advancing front method
pub fn generate_triangular_mesh(points: &[Point2], polygons: &[usize]) -> (Vec<Point2>, Vec<u32>) {
    let mut triangles = Vec::new();
    let mut vertices = Vec::new();
    vertices.extend_from_slice(points);
    let mut front = VecDeque::new();

    // initialize the front with edges of polygons
    for pair in polygons.windows(2) {
        if let [start, end] = *pair {
            for i in start..(end - 1) {
                front.push_back((i, i + 1));
            }
            front.push_back((end - 1, start));
        }
    }

    // advancing the front unit it is empty
    while let Some(edge) = front.pop_back() {
        let (a, b) = (vertices[edge.0], vertices[edge.1]);
        let mut selected_node = None;

        loop {
            if let Some(node) =
                find_intersection_with_front(edge.0, edge.1, selected_node, &front, &vertices)
            {
                selected_node = Some(node);
                continue;
            }
            break;
        }

        if let Some(node) = selected_node {
            if let Some(index) = front.iter().position(|e| e.0 == node && e.1 == edge.0) {
                front.swap_remove_front(index);
            } else {
                front.push_front((edge.0, node));
            }
            if let Some(index) = front.iter().position(|e| e.0 == edge.1 && e.1 == node) {
                front.swap_remove_front(index);
            } else {
                front.push_front((node, edge.1));
            }
            triangles.extend_from_slice(&[edge.0 as u32, edge.1 as u32, node as u32]);
        } else {
            let index = vertices.len();
            let new_point = point_on_normal_bisector(a, b, HALF_SQRT_3);
            vertices.push(new_point);
            front.push_front((edge.0, index));
            front.push_front((index, edge.1));
            triangles.extend_from_slice(&[edge.0 as u32, edge.1 as u32, index as u32]);
        }
    }
    (vertices, triangles)
}

fn find_intersection_with_front(
    start: usize,
    end: usize,
    selected_node: Option<usize>,
    front: &VecDeque<(usize, usize)>,
    vertices: &Vec<Point2>,
) -> Option<usize> {
    let (a, b) = (vertices[start], vertices[end]);
    let new_point = match selected_node {
        Some(node) => vertices[node],
        None => point_on_normal_bisector(a, b, 1.0),
    };
    // it is possilbe that a node is a split node, so that multiple edges starts and ends with it
    let mut nearby_nodes = front
        .iter()
        .filter_map(|e| {
            if selected_node != Some(e.0) {
                if e.0 != start && e.0 != end && vertices[e.0].is_inside_triangle(a, b, new_point) {
                    let factor = shape_factor(a, b, vertices[e.0]);
                    // give inner nodes higher priority
                    return Some((e.0, 1.0 + factor));
                } else if selected_node != Some(e.1) {
                    let (c, d) = (vertices[e.0], vertices[e.1]);
                    if e.0 != start && e.1 != start && is_segments_intersect(a, new_point, c, d) {
                        let fc = shape_factor(a, b, c);
                        let fd = shape_factor(a, b, d);
                        return if e.0 == end {
                            Some((e.1, fd))
                        } else if fc > fd {
                            Some((e.0, fc))
                        } else {
                            Some((e.1, fd))
                        };
                    } else if e.0 != end && e.1 != end && is_segments_intersect(b, new_point, c, d)
                    {
                        let fc = shape_factor(a, b, c);
                        let fd = shape_factor(a, b, d);
                        return if e.1 == start {
                            Some((e.0, fc))
                        } else if fc > fd {
                            Some((e.0, fc))
                        } else {
                            Some((e.1, fd))
                        };
                    }
                }
            }
            return None;
        })
        .collect::<Vec<_>>();

    nearby_nodes.sort_by(|a, b| match a.0.cmp(&b.0) {
        std::cmp::Ordering::Equal => b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less),
        order @ _ => order,
    });
    nearby_nodes.dedup_by_key(|(node, _)| *node);
    if nearby_nodes.len() > 1 {
        nearby_nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Less));
    }
    nearby_nodes.pop().map(|(node, _)| node)
}

const MIN_GRID_SIZE: Float = 0.001;
fn point_on_normal_bisector(a: Point2, b: Point2, ratio: Float) -> Point2 {
    let center = (a + b) / 2.0;
    let mut displace = (b - a).perp();
    if displace.length() < MIN_GRID_SIZE {
        displace = displace.normalize() * 0.01;
    }
    center + displace * ratio
}

fn is_segments_intersect(a: Point2, b: Point2, c: Point2, d: Point2) -> bool {
    if a.x.min(b.x) > c.x.max(d.x)
        || c.x.min(d.x) > a.x.max(b.x)
        || a.y.min(b.y) > c.y.max(d.y)
        || c.y.min(d.y) > a.y.max(b.y)
    {
        return false;
    }
    let ac = c - a;
    let bc = c - b;
    let ad = d - a;
    let bd = d - b;
    if ac.perp_dot(ad) * bd.perp_dot(bc) < 0.0 {
        return false;
    }
    if ac.perp_dot(bc) * bd.perp_dot(ad) < 0.0 {
        return false;
    }
    return true;
}

fn shape_factor(a: Point2, b: Point2, c: Point2) -> Float {
    let ab = b - a;
    let ac = c - a;
    let bc = c - b;
    ab.perp_dot(ac) / (ab.length_squared() + ac.length_squared() + bc.length_squared())
}

#[test]
fn test_segments_intersect() {
    let a = Point2::new(8.680205032300453, -91.00000004229226);
    let b = Point2::new(10.924396800019334, -91.00000004229226);
    let c = Point2::new(15.412780335457098, -91.00000004229226);
    let d = Point2::new(17.656972103175978, -91.00000004229226);
    assert!(!is_segments_intersect(a, b, c, d));
}

#[test]
fn test_gen_mesh() {
    let points = vec![
        Point2::new(0.0, 0.2),
        Point2::new(-0.5, 0.0),
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(2.0, 0.0),
        Point2::new(2.0, 1.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
        Point2::new(-0.5, 1.0),
        Point2::new(0.0, 0.8),
    ];
    let (points, triangles) = generate_triangular_mesh(&points, &[0, points.len()]);
    crate::TriangleMesh {
        vertices: points.iter().map(|p| p.extend(0.0)).collect(),
        triangles,
        normals: Vec::new(),
    }
    .save_as_obj("test.obj")
    .unwrap();
}
