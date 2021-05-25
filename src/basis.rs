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
