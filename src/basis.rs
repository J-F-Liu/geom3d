use crate::Float;

/// Compute values of (n-1)th-degree Bernstein polynomial
pub fn bernstein(n: usize, u: Float) -> Vec<Float> {
    let mut values = vec![0.0; n];
    values[0] = 1.0;
    let u1 = 1.0 - u;

    for j in 1..n {
        let mut saved = 0.0;
        for k in 0..j {
            let temp = values[k];
            values[k] = saved + u1 * temp;
            saved = u * temp;
        }
        values[j] = saved;
    }

    values
}

#[test]
fn test_bernstein() {
    assert_eq!(
        bernstein(4, 0.25),
        vec![0.421875, 0.421875, 0.140625, 0.015625,]
    );
}
