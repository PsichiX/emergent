//! Score mappings are used to remap scores calculated by considerations.

use crate::Scalar;
use std::ops::Range;

/// Score mapping is used to manipulate score calculated by consideration.
///
/// Usually it can be applied to [`ConsiderationRemap::new`](fn@crate::consideration::ConsiderationRemap::new).
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
/// let consideration = ConsiderationRemap::new(
///     0.5, // consideration constant score
///     Squared,
/// );
/// assert_eq!(consideration.score(&()), 0.25);
/// ```
pub trait ScoreMapping {
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

impl ScoreMapping for dyn Fn(Scalar) -> Scalar {
    fn remap(&self, score: Scalar) -> Scalar {
        self(score)
    }
}

/// Does nothing - returns score got as input.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let consideration = ConsiderationRemap::new(
///     0.5, // consideration constant score
///     NoScoreMapping,
/// );
/// assert_eq!(consideration.score(&()), 0.5);
/// ```
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
/// let consideration = ConsiderationRemap::new(
///     0.5, // consideration constant score
///     ClosureScoreMapping::new(|score| score * score),
/// );
/// assert_eq!(consideration.score(&()), 0.25);
/// ```
pub struct ClosureScoreMapping(pub Box<dyn Fn(Scalar) -> Scalar>);

impl ClosureScoreMapping {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Scalar) -> Scalar + 'static,
    {
        Self(Box::new(f))
    }
}

impl ScoreMapping for ClosureScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        (self.0)(score)
    }
}

/// Chains two score mappings so [`Self::first`] remaps input score and then [`Self::second`] remaps
/// score got from [`Self::first`]: `second.remap(first.remap(score))`.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let consideration = ConsiderationRemap::new(
///     0.0, // consideration constant score
///     ReverseScoreMapping.chain(ReverseScoreMapping),
/// );
/// assert_eq!(consideration.score(&()), 0.0);
/// ```
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
/// let consideration = ConsiderationRemap::new(
///     5.0, // consideration constant score
///     ScoreRemap::new(0.0..10.0, 0.0..1.0),
/// );
/// assert_eq!(consideration.score(&()), 0.5);
/// ```
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
/// let consideration = ConsiderationRemap::new(
///     1.0, // consideration constant score
///     ReverseScoreMapping,
/// );
/// assert_eq!(consideration.score(&()), 0.0);
/// ```
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
/// let consideration = ConsiderationRemap::new(
///     10.0, // consideration constant score
///     InverseScoreMapping,
/// );
/// assert_eq!(consideration.score(&()), 0.1);
/// ```
pub struct InverseScoreMapping;

impl ScoreMapping for InverseScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        1.0 / score
    }
}

/// Applies fast sigmoid function.
/// See [WolframAlpha](https://www.wolframalpha.com/input/?i2d=true&i=f%5C%2840%29x%5C%2841%29%3D+Divide%5Bx%2C1+%2B+Abs%5Bx%5D%5D).
pub struct FastSigmoidScoreMapping;

impl ScoreMapping for FastSigmoidScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score / (1.0 + score.abs())
    }
}

/// Applies approximated sigmoid function.
/// See [WolframAlpha](https://www.wolframalpha.com/input/?i2d=true&i=f%5C%2840%29x%5C%2841%29%3D+Divide%5Bx%2CSqrt%5B1+%2B+Power%5Bx%2C2%5D%5D%5D).
pub struct ApproxSigmoidScoreMapping;

impl ScoreMapping for ApproxSigmoidScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score / (1.0 + (score * score).sqrt())
    }
}
