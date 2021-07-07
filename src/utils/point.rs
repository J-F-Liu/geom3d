use crate::utils::Tolerance;
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
        let pab = pa.perp_dot(pb);
        let pbc = pb.perp_dot(pc);
        let pca = pc.perp_dot(pa);
        (pab >= 0.0 || pab.near(0.0))
            && (pbc >= 0.0 || pbc.near(0.0))
            && (pca >= 0.0 || pca.near(0.0))
    }
}

#[test]
fn test_distance_to_line_segment() {
    let a = Point2::new(-19.06804, 200.03492);
    let b = Point2::new(-17.58058, 199.41942);
    let c = Point2::new(-17.09570, 196.54255);
    dbg!(c.distance_to_line_segment(a, b));
}

#[test]
fn test_inside_triangle() {
    let p = Point2::new(-5.645197896412814, 196.2149812486018);
    let a = Point2::new(-6.77392, 199.37674);
    let b = Point2::new(-5.41912, 194.42925);
    let c = Point2::new(-3.972502120793216, 197.33392534246588);
    dbg!(p.is_inside_triangle(a, b, c));
}
