//! Score mappings are used to remap scores calculated by considerations.

use crate::Scalar;
use std::ops::Range;

/// Score mapping is used to manipulate score calculated by consideration.
///
/// Usually it can be applied to [`ConsiderationRemap::new`](fn@crate::consideration::ConsiderationRemap::new)
/// or [`Consideration::remap`](fn@crate::consideration::Consideration::remap).
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Squared;
///
/// impl ScoreMapping for Squared {
///     fn remap(&self, score: Scalar) -> Scalar {
///         score * score
///     }
/// }
///
/// assert_eq!(0.5.remap(Squared).score(&()), 0.25);
/// ```
pub trait ScoreMapping: Send + Sync {
    /// Remaps score got from consideration.
    fn remap(&self, score: Scalar) -> Scalar;

    /// Chains this mapping with `other`.
    ///
    /// See [`ChainedScoreMapping`].
    fn chain<T>(self, other: T) -> ChainedScoreMapping<Self, T>
    where
        T: ScoreMapping,
        Self: Sized,
    {
        ChainedScoreMapping::new(self, other)
    }
}

/// Does nothing - returns score got as input.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// assert_eq!(0.5.remap(NoScoreMapping).score(&()), 0.5);
/// ```
#[derive(Debug, Default, Copy, Clone)]
pub struct NoScoreMapping;

impl ScoreMapping for NoScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score
    }
}

/// Wraps score mapping closure.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// assert_eq!(0.5.remap(ClosureScoreMapping::new(|score| score * score)).score(&()), 0.25);
/// ```
pub struct ClosureScoreMapping(pub Box<dyn Fn(Scalar) -> Scalar + Send + Sync>);

impl ClosureScoreMapping {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Scalar) -> Scalar + 'static + Send + Sync,
    {
        Self(Box::new(f))
    }
}

impl ScoreMapping for ClosureScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        (self.0)(score)
    }
}

impl std::fmt::Debug for ClosureScoreMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureScoreMapping").finish()
    }
}

/// Chains two score mappings so [`Self::first`] remaps input score and then [`Self::second`] remaps
/// score got from [`Self::first`]: `second.remap(first.remap(score))`.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// assert_eq!(0.0.remap(ReverseScoreMapping.chain(ReverseScoreMapping)).score(&()), 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct ChainedScoreMapping<A, B>
where
    A: ScoreMapping,
    B: ScoreMapping,
{
    pub first: A,
    pub second: B,
}

impl<A, B> ChainedScoreMapping<A, B>
where
    A: ScoreMapping,
    B: ScoreMapping,
{
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> ScoreMapping for ChainedScoreMapping<A, B>
where
    A: ScoreMapping,
    B: ScoreMapping,
{
    fn remap(&self, score: Scalar) -> Scalar {
        self.second.remap(self.first.remap(score))
    }
}

/// Remaps score from source range ([`Self::from`]) to target range ([`Self::to`]).
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// assert_eq!(5.0.remap(ScoreRemap::new(0.0..10.0, 0.0..1.0)).score(&()), 0.5);
/// ```
#[derive(Debug, Clone)]
pub struct ScoreRemap {
    pub from: Range<Scalar>,
    pub to: Range<Scalar>,
}

impl ScoreRemap {
    pub fn new(from: Range<Scalar>, to: Range<Scalar>) -> Self {
        Self { from, to }
    }
}

impl ScoreMapping for ScoreRemap {
    fn remap(&self, score: Scalar) -> Scalar {
        let factor = (score - self.from.start) / (self.from.end - self.from.start);
        factor * (self.to.end - self.to.start) + self.to.start
    }
}

/// Applies `1.0 - score`.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// assert_eq!(1.0.remap(ReverseScoreMapping).score(&()), 0.0);
/// ```
#[derive(Debug, Default, Copy, Clone)]
pub struct ReverseScoreMapping;

impl ScoreMapping for ReverseScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        1.0 - score
    }
}

/// Applies `1.0 / score`.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// assert_eq!(10.0.remap(InverseScoreMapping).score(&()), 0.1);
/// ```
#[derive(Debug, Default, Copy, Clone)]
pub struct InverseScoreMapping;

impl ScoreMapping for InverseScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        1.0 / score
    }
}

/// Applies fast sigmoid function.
/// See [WolframAlpha](https://www.wolframalpha.com/input/?i2d=true&i=f%5C%2840%29x%5C%2841%29%3D+Divide%5Bx%2C1+%2B+Abs%5Bx%5D%5D).
#[derive(Debug, Default, Copy, Clone)]
pub struct FastSigmoidScoreMapping;

impl ScoreMapping for FastSigmoidScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score / (1.0 + score.abs())
    }
}

/// Applies approximated sigmoid function.
/// See [WolframAlpha](https://www.wolframalpha.com/input/?i2d=true&i=f%5C%2840%29x%5C%2841%29%3D+Divide%5Bx%2CSqrt%5B1+%2B+Power%5Bx%2C2%5D%5D%5D).
#[derive(Debug, Default, Copy, Clone)]
pub struct ApproxSigmoidScoreMapping;

impl ScoreMapping for ApproxSigmoidScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score / (1.0 + (score * score).sqrt())
    }
}

/// Applies ReLU (maximum between score and 0).
#[derive(Debug, Default, Copy, Clone)]
pub struct ReluScoreMapping;

impl ScoreMapping for ReluScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score.max(0.0)
    }
}

/// Applies soft version of ReLU.
#[derive(Debug, Default, Copy, Clone)]
pub struct SoftplusScoreMapping;

impl ScoreMapping for SoftplusScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        #[cfg(not(feature = "scalar64"))]
        let base = std::f32::consts::E;
        #[cfg(feature = "scalar64")]
        let base = std::f64::consts::E;
        (1.0 + score.exp()).log(base)
    }
}
