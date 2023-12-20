use super::Surface;
use crate::consts::PI;
use crate::{Float, Point3};

#[derive(Debug, Clone)]
pub struct Umbrella {
    pub radius: Float,
}

impl Surface for Umbrella {
    // create umbrella surface by rotate cycloid curve around z-axis
    // x = r(θ - sinθ)cosφ
    // y = r(θ - sinθ)sinφ
    // z = r(1 + cosθ)
    fn get_point(&self, theta: Float, phi: Float) -> Point3 {
        let r = self.radius / PI; // compute radius of cycloid curve
        let (sin_t, cos_t) = theta.sin_cos();
        let (sin, cos) = phi.sin_cos();
        Point3::new(
            r * (theta - sin_t) * cos,
            r * (theta - sin_t) * sin,
            r * (1.0 + cos_t),
        )
    }
}

#[test]
fn test_create_umbrella_surface() {
    use crate::face::Face;
    let umbrella = crate::surface::SurfacePatch {
        surface: Umbrella { radius: PI },
        parameter_range: ((0.0, PI), (0.0, 2.0 * PI)),
        parameter_division: (64, 128),
    };
    let mesh = umbrella.get_triangle_mesh();
    mesh.save_as_obj("tmp/mesh.obj").unwrap();
    let points = umbrella.get_point_grid().into_vec();
    crate::points::save_point_cloud(&points, "tmp/umbrella.xyz").expect("save points");
}
