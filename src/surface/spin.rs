use super::{Curve, Surface};
use crate::{Float, Point3, Quat, Vec3};

/// Spin surface is created by spin a section curve along a rotation axis.
#[derive(Debug)]
pub struct SpinSurface<C: Curve> {
    /// Origin point on the rotation axis
    pub origin: Point3,
    /// Normalized direction of rotation axis
    pub axis: Vec3,
    /// Section curve of spin surface
    pub section: C,
}

impl<C: Curve> Surface for SpinSurface<C> {
    fn get_point(&self, param: Float, angle: Float) -> Point3 {
        let vector = self.section.get_point(param) - self.origin;
        let parallel_component = self.axis * vector.dot(self.axis);
        let perpendicular_component = vector - parallel_component;
        let rotation = Quat::from_axis_angle(self.axis, angle);
        self.origin + parallel_component + rotation * perpendicular_component
    }
}
