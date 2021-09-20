pub mod builders;
pub mod combinators;
pub mod condition;
pub mod consideration;
pub mod decision_makers;
pub mod evaluators;
pub mod score_mapping;
pub mod task;

#[cfg(test)]
pub mod tests;

#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;
#[cfg(feature = "scalar64")]
pub type Scalar = f64;

pub type DefaultKey = String;
