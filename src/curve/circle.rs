use super::Curve;
use crate::{Float, Point3, Quat, Vec3};

#[derive(Debug)]
pub struct Circle {
    pub center: Point3,
    pub radius: Float,
    pub axis: Vec3,
    pub ref_dir: Vec3,
}

impl Curve for Circle {
    fn get_point(&self, angle: Float) -> Point3 {
        let rotation = Quat::from_axis_angle(self.axis, angle);
        self.center + rotation * self.ref_dir * self.radius
    }

    fn project(&self, point: Point3) -> Float {
        let y_axis = self.axis.cross(self.ref_dir).normalize();
        let direction = point - self.center;
        let x = direction.dot(self.ref_dir);
        let y = direction.dot(y_axis);
        let angle = y.atan2(x);
        if angle >= 0.0 {
            angle
        } else {
            angle + crate::consts::TAU
        }
    }
}

#[test]
fn test_circle() {
    let circle = Circle {
        center: Point3::new(212.25, -60.17616798, 5.25),
        radius: 52.52114465,
        axis: Vec3::new(0.0, 1.0, 0.0),
        ref_dir: Vec3::new(0.898686585, 0.0, 0.438591408),
    };
    let a = circle.project(Point3::new(259.450048095, -60.176167979, 28.285322751));
    let b = circle.project(Point3::new(263.834197783, -60.176168023, 15.126293507));
    let segment = crate::curve::CurveSegment {
        curve: circle,
        parameter_range: (a, b),
        parameter_division: 16,
    };
    dbg!(segment.get_points());
}
