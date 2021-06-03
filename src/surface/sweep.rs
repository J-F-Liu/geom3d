use super::{Curve, Surface};
use crate::{Float, Point3};

/// Sweep surface is created by moving a section curve along a path curve.
#[derive(Debug)]
pub struct SweepSurface<P: Curve, S: Curve> {
    pub path: P,
    pub section: S,
}

impl<P: Curve, S: Curve> Surface for SweepSurface<P, S> {
    fn get_point(&self, p: Float, s: Float) -> Point3 {
        self.path.get_point(p) + self.section.get_point(s)
    }
}
