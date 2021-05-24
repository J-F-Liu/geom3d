use crate::Float;

pub fn create_parameters(parameter_range: (Float, Float), count: usize) -> Vec<Float> {
    let (begin, end) = parameter_range;
    let step = (end - begin) / (count - 1) as Float;
    let mut parameters = Vec::with_capacity(count);
    let mut u = begin;
    for _ in 0..count {
        parameters.push(u);
        u += step;
    }
    parameters
}

/// Compute values of (n-1)th-degree Bernstein polynomial
pub fn bernstein(n: usize, u: Float) -> Vec<Float> {
    let u1 = 1.0 - u;
    let mut values = vec![0.0; n];
    values[1] = 1.0;

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
