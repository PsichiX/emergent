use crate::Scalar;
use std::ops::Range;

pub trait ScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar;

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

pub struct NoScoreMapping;

impl ScoreMapping for NoScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score
    }
}

pub struct ChainedScoreMapping<A, B>
where
    A: ScoreMapping,
    B: ScoreMapping,
{
    first: A,
    second: B,
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

pub struct ScoreRemap {
    pub from: Range<Scalar>,
    pub to: Range<Scalar>,
}

impl ScoreMapping for ScoreRemap {
    fn remap(&self, score: Scalar) -> Scalar {
        let factor = (score - self.from.start) / (self.from.end - self.from.start);
        factor * (self.to.end - self.to.start) + self.to.start
    }
}

pub struct ReverseScoreMapping;

impl ScoreMapping for ReverseScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        1.0 - score
    }
}

pub struct InverseScoreMapping;

impl ScoreMapping for InverseScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        1.0 / score
    }
}

pub struct FastSigmoidScoreMapping;

impl ScoreMapping for FastSigmoidScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score / (1.0 + score.abs())
    }
}

pub struct ApproxSigmoidScoreMapping;

impl ScoreMapping for ApproxSigmoidScoreMapping {
    fn remap(&self, score: Scalar) -> Scalar {
        score / (1.0 + (score * score).sqrt())
    }
}
