use super::Curve;
use crate::{Float, Point3};

#[derive(Debug)]
pub struct Polyline {
    pub vertices: Vec<Point3>,
    pub segment_lengths: Vec<Float>,
    pub length: Float,
}

impl Polyline {
    pub fn new(vertices: Vec<Point3>) -> Self {
        assert!(vertices.len() >= 2);
        let segment_lengths = vertices
            .windows(2)
            .map(|pair| pair[1].distance(pair[0]))
            .collect::<Vec<_>>();
        let length = segment_lengths.iter().sum();
        Polyline {
            vertices,
            segment_lengths,
            length,
        }
    }

    pub fn start(&self) -> Point3 {
        self.vertices[0]
    }

    pub fn end(&self) -> Point3 {
        self.vertices[self.vertices.len() - 1]
    }

    /// Get index of nearest vertex
    pub fn nearest_vertex(&self, point: Point3) -> usize {
        let mut min = Float::MAX;
        let mut min_index = 0;

        for (index, &vertex) in self.vertices.iter().enumerate() {
            let distance = (point - vertex).length_squared();
            if distance < min {
                min = distance;
                min_index = index;
            }
        }

        min_index
    }
}

impl Curve for Polyline {
    fn get_point(&self, t: Float) -> Point3 {
        if t <= 0.0 {
            self.start()
        } else if t >= 1.0 {
            self.end()
        } else {
            let mut length = t * self.length;
            for (index, &segment_length) in self.segment_lengths.iter().enumerate() {
                if length > segment_length {
                    length -= segment_length;
                } else {
                    return self.vertices[index]
                        .lerp(self.vertices[index + 1], length / segment_length);
                }
            }
            unreachable!()
        }
    }

    fn project(&self, point: Point3) -> Float {
        let vertex_index = self.nearest_vertex(point);
        if vertex_index == 0 {
            0.0
        } else if vertex_index == self.vertices.len() - 1 {
            1.0
        } else {
            let distance_prev = (point - self.vertices[vertex_index - 1])
                .cross(self.vertices[vertex_index] - self.vertices[vertex_index - 1])
                .length()
                / self.segment_lengths[vertex_index - 1];
            let distance_next = (point - self.vertices[vertex_index])
                .cross(self.vertices[vertex_index + 1] - self.vertices[vertex_index])
                .length()
                / self.segment_lengths[vertex_index];
            let min_index = if distance_prev < distance_next {
                vertex_index - 1
            } else {
                vertex_index
            };
            let length = self.segment_lengths.iter().take(min_index).sum::<Float>()
                + (point - self.vertices[min_index])
                    .dot((self.vertices[min_index + 1] - self.vertices[min_index]).normalize());
            (length / self.length).clamp(0.0, 1.0)
        }
    }
}
