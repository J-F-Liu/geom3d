use crate::{Float, Point2};

pub trait Point {
    fn distance_to_line_segment(&self, a: Self, b: Self) -> Float;
    fn is_inside_triangle(&self, a: Self, b: Self, c: Self) -> bool;
}

impl Point for Point2 {
    fn distance_to_line_segment(&self, a: Point2, b: Point2) -> Float {
        let ap = *self - a;
        let ab = b - a;
        let product = ap.dot(ab);
        if product <= 0.0 {
            return ap.length();
        }
        if product >= ab.length_squared() {
            let bp = *self - b;
            return bp.length();
        }
        return ap.perp_dot(ab).abs() / ab.length();
    }

    fn is_inside_triangle(&self, a: Point2, b: Point2, c: Point2) -> bool {
        let pa = a - *self;
        let pb = b - *self;
        let pc = c - *self;
        pa.perp_dot(pb) >= 0.0 && pb.perp_dot(pc) >= 0.0 && pc.perp_dot(pa) >= 0.0
    }
}

#[test]
fn test_inside_triangle() {
    let p = Point2::new(-5.645197896412814, 196.2149812486018);
    let a = Point2::new(-6.77392, 199.37674);
    let b = Point2::new(-5.41912, 194.42925);
    let c = Point2::new(-3.972502120793216, 197.33392534246588);
    dbg!(p.is_inside_triangle(a, b, c));
}
