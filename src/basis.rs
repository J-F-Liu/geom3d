use crate::Float;

pub fn create_parameters(parameter_range: (Float, Float), division: usize) -> Vec<Float> {
    let (begin, end) = parameter_range;
    let step = (end - begin) / division as Float;
    let mut parameters = Vec::with_capacity(division + 1);
    parameters.push(begin);
    let mut u = begin;
    for _ in 0..division - 1 {
        u += step;
        parameters.push(u);
    }
    parameters.push(end);
    parameters
}

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
        create_parameters((0.0, 1.0), 8),
        vec![0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0,]
    );
    assert_eq!(
        create_parameters((1.0, 0.0), 8),
        vec![1.0, 0.875, 0.75, 0.625, 0.5, 0.375, 0.25, 0.125, 0.0,]
    );
    assert_eq!(
        bernstein(4, 0.25),
        vec![0.421875, 0.421875, 0.140625, 0.015625,]
    );
}
