use crate::Float;
use approx::ulps_eq;

fn inv_or_zero(delta: Float) -> Float {
    if ulps_eq!(delta, 0.0) {
        0.0
    } else {
        1.0 / delta
    }
}

/// knot vector
#[derive(Clone, PartialEq, Debug)]
pub struct KnotVector(Vec<Float>);

impl KnotVector {
    pub fn new(knots: Vec<Float>) -> KnotVector {
        KnotVector(knots)
    }

    pub fn from_values_and_multiplicities(
        values: Vec<Float>,
        multiplicities: Vec<usize>,
    ) -> KnotVector {
        let mut knots = Vec::with_capacity(multiplicities.iter().sum());
        for (value, multiplicity) in values.into_iter().zip(multiplicities.into_iter()) {
            knots.extend(std::iter::repeat(value).take(multiplicity));
        }
        KnotVector(knots)
    }

    pub fn normalize(&self) -> KnotVector {
        let start = self[0];
        let length = self[self.len() - 1] - self[0];
        self.iter()
            .map(|value| (value - start) / length)
            .collect::<Vec<_>>()
            .into()
    }

    /// the multiplicity of the `i`th knot
    pub fn multiplicity(&self, i: usize) -> usize {
        self.iter().filter(|u| ulps_eq!(self[i], u)).count()
    }

    /// Returns the span index of which span `u` belongs to.
    pub fn span_index(&self, u: Float) -> usize {
        if let Some(index) = self.iter().rposition(|t| *t <= u) {
            if index == self.len() - 1 {
                index - self.multiplicity(index)
            } else {
                index
            }
        } else {
            self.multiplicity(0)
        }
    }

    /// Compute values of B-Spline basis function at `u` with `degree`.
    pub fn bspline_basis(&self, degree: usize, u: Float) -> Vec<Float> {
        let n = self.len() - 1;
        let index = self.span_index(u);
        let mut values = vec![0.0; n];
        values[index] = 1.0;

        for k in 1..=degree {
            let base = if index < k { 0 } else { index - k };
            let delta = self[base + k] - self[base];
            let max = if index + k < n { index } else { n - k - 1 };
            let mut a = inv_or_zero(delta) * (u - self[base]);
            for i in base..=max {
                let delta = self[i + k + 1] - self[i + 1];
                let b = inv_or_zero(delta) * (self[i + k + 1] - u);
                values[i] = a * values[i] + b * values[i + 1];
                a = 1.0 - b;
            }
        }

        values.truncate(n - degree);

        values
    }

    /// Constructs the knot vector for the bezier spline.
    /// # Examples
    /// ```
    /// use geom3d::*;
    /// assert_eq!(
    ///     *KnotVector::bezier_knot(3),
    ///     vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn bezier_knot(degree: usize) -> KnotVector {
        let mut knots = Vec::with_capacity(degree * 2 + 2);
        knots.extend(std::iter::repeat(0.0).take(degree + 1));
        knots.extend(std::iter::repeat(1.0).take(degree + 1));
        KnotVector(knots)
    }

    /// Constructs the uniform knot vector
    /// # Examples
    /// ```
    /// use geom3d::*;
    /// assert_eq!(
    ///     *KnotVector::uniform_knot(2, 4),
    ///     vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn uniform_knot(degree: usize, division: usize) -> KnotVector {
        let step = 1.0 / division as Float;
        let mut knots = Vec::with_capacity(degree * 2 + 2);
        knots.extend(std::iter::repeat(0.0).take(degree + 1));
        knots.extend((1..division).map(|i| (i as Float) * step));
        knots.extend(std::iter::repeat(1.0).take(degree + 1));
        KnotVector(knots)
    }
}

impl From<Vec<Float>> for KnotVector {
    fn from(vec: Vec<Float>) -> KnotVector {
        KnotVector(vec)
    }
}

impl std::iter::FromIterator<Float> for KnotVector {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Float>>(iter: I) -> KnotVector {
        KnotVector::new(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<'a> IntoIterator for &'a KnotVector {
    type Item = &'a Float;
    type IntoIter = std::slice::Iter<'a, Float>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::ops::Deref for KnotVector {
    type Target = Vec<Float>;
    #[inline(always)]
    fn deref(&self) -> &Vec<Float> {
        &self.0
    }
}

impl AsRef<[Float]> for KnotVector {
    #[inline(always)]
    fn as_ref(&self) -> &[Float] {
        &self.0
    }
}

#[test]
fn test_bspline_basis() {
    let knots = KnotVector::bezier_knot(3);
    assert_eq!(knots.bspline_basis(3, 0.0), vec![1.0, 0.0, 0.0, 0.0]);
    assert_eq!(
        knots.bspline_basis(3, 0.5),
        vec![0.125, 0.375, 0.375, 0.125]
    );
    assert_eq!(knots.bspline_basis(3, 1.0), vec![0.0, 0.0, 0.0, 1.0]);
    assert_eq!(
        *KnotVector::uniform_knot(2, 2),
        vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0],
    );
}
