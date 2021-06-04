use super::Curve;
use crate::{utils, Float, Point3};

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
        utils::find_nearest_point(&self.vertices, point)
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
        let mut min = Float::MAX;
        let mut min_index = 0;

        for (index, pair) in self.vertices.windows(2).enumerate() {
            let distance = utils::distance_to_line_segment(pair[0], pair[1], point);
            if distance < min {
                min = distance;
                min_index = index;
            }
        }

        let length = self.segment_lengths.iter().take(min_index).sum::<Float>()
            + (point - self.vertices[min_index])
                .dot((self.vertices[min_index + 1] - self.vertices[min_index]).normalize());
        (length / self.length).clamp(0.0, 1.0)
    }
}
