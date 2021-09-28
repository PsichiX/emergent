# Use of Reasoner decision maker from emergent crate

Usage of [`Reasoner`] decision maker from [`emergent`] crate isn't much different
from what we have made by ourselves - what's different, considerations are actually
separate structs for which we implement [`Consideration`] trait but we could also
just use scoring functions and wrap them with [`ClosureConsideration`], but we
want to keep ourselves on the full modularity track - the point is, there are
many ways you can organize your logic with [`emergent`] crate, it is all for you
to decide what works best for you.

```rust
# extern crate emergent;
# use emergent::prelude::*;
#
# #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
# enum EnemyState {
#   Idle,
#   GatherFood,
#   GatherWood,
#   AttackOpponent,
# }
#
# struct EnemyData {
#   hunger: f32,
#   distance_to_food: f32,
#   distance_to_trees: f32,
#   wood_needed: usize,
#   distance_to_opponent: f32,
#   opponent_strength: f32,
# }
#
struct Enemy {
  data: EnemyData,
  reasoner: Reasoner<EnemyData, EnemyState>,
}

impl Enemy {
  fn new() -> Self {
    let data = EnemyData {
      hunger: 0.0,
      distance_to_food: 1.0,
      distance_to_trees: 1.0,
      wood_needed: 0,
      distance_to_opponent: 1.0,
      opponent_strength: 0.0,
    };
    let reasoner = ReasonerBuilder::default()
      // Just like with conditions, Consideration trait is implemented to scalars
      // so we can just use constants here.
      // This translates to: `0.001`
      .state(EnemyState::Idle, ReasonerState::new(0.001, EnemyIdleTask))
      .state(EnemyState::GatherFood, ReasonerState::new(
        // Product evaluator will multiply all its children consideration scores.
        // This translates to: `hunger * (1 - food proximity)`
        EvaluatorProduct::default()
          .consideration(Hunger)
          .consideration(ConsiderationRemap::new(
            FoodProximity,
            ReverseScoreMapping,
          )),
        EnemyGatherFoodTask,
      ))
      .state(EnemyState::GatherWood, ReasonerState::new(
        // This translates to: `(wood needed ? 1 : 0) * (1 - trees proximity)`
        EvaluatorProduct::default()
          .consideration(ConditionConsideration::unit(WoodNeeded))
          .consideration(ConsiderationRemap::new(
            TreesProximity,
            ReverseScoreMapping,
          )),
        EnemyGatherWoodTask,
      ))
      .state(EnemyState::AttackOpponent, ReasonerState::new(
        // This translates to: `(1 - opponent proximity) + opponent strength`
        EvaluatorSum::default()
          .consideration(ConsiderationRemap::new(
            OpponentProximity,
            ReverseScoreMapping,
          ))
          .consideration(OpponentStrength),
        EnemyAttackOpponentTask,
      ))
      .build();

    Self { data, reasoner }
  }

  fn tick(&mut self) {
    self.reasoner.process(&mut self.data);
    self.reasoner.update(&mut self.data);
  }
}

// Conditions

struct WoodNeeded;

impl Condition<EnemyData> for WoodNeeded {
  fn validate(&self, memory: &EnemyData) -> bool {
    memory.wood_needed > 0
  }
}

// Considerations

struct Hunger;

impl Consideration<EnemyData> for Hunger {
  fn score(&self, memory: &EnemyData) -> Scalar {
    memory.hunger
  }
}

struct FoodProximity;

impl Consideration<EnemyData> for FoodProximity {
  fn score(&self, memory: &EnemyData) -> Scalar {
    memory.distance_to_food
  }
}

struct TreesProximity;

impl Consideration<EnemyData> for TreesProximity {
  fn score(&self, memory: &EnemyData) -> Scalar {
    memory.distance_to_trees
  }
}

struct OpponentProximity;

impl Consideration<EnemyData> for OpponentProximity {
  fn score(&self, memory: &EnemyData) -> Scalar {
    memory.distance_to_opponent
  }
}

struct OpponentStrength;

impl Consideration<EnemyData> for OpponentStrength {
  fn score(&self, memory: &EnemyData) -> Scalar {
    memory.opponent_strength
  }
}

// Enemy state tasks

struct EnemyIdleTask;

impl Task<EnemyData> for EnemyIdleTask {}

struct EnemyGatherFoodTask;

impl Task<EnemyData> for EnemyGatherFoodTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.hunger = 0.0;
    memory.distance_to_food = 1.0;
  }
}

struct EnemyGatherWoodTask;

impl Task<EnemyData> for EnemyGatherWoodTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.wood_needed = memory.wood_needed.max(1) - 1;
    memory.distance_to_trees = 1.0;
  }
}

struct EnemyAttackOpponentTask;

impl Task<EnemyData> for EnemyAttackOpponentTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.distance_to_opponent = 1.0;
    memory.opponent_strength = 0.0;
  }
}

// Test run

let mut enemy = Enemy::new();

assert_eq!(enemy.reasoner.active_state(), None);
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 1.0);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.data.hunger = 0.5;
enemy.data.distance_to_food = 0.9;
enemy.data.distance_to_trees = 0.5;
enemy.data.opponent_strength = 0.2;

enemy.tick();
assert_eq!(enemy.reasoner.active_state(), Some(&EnemyState::AttackOpponent));
assert_eq!(enemy.data.hunger, 0.5);
assert_eq!(enemy.data.distance_to_food, 0.9);
assert_eq!(enemy.data.distance_to_trees, 0.5);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.tick();
assert_eq!(enemy.reasoner.active_state(), Some(&EnemyState::GatherFood));
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 0.5);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.data.wood_needed = 1;

enemy.tick();
assert_eq!(enemy.reasoner.active_state(), Some(&EnemyState::GatherWood));
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 1.0);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.tick();
assert_eq!(enemy.reasoner.active_state(), Some(&EnemyState::Idle));
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 1.0);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);
```

[`emergent`]: https://crates.io/emergent
[`Reasoner`]: https://docs.rs/emergent/1.5.0/emergent/decision_makers/reasoner/struct.Reasoner.html
[`Consideration`]: https://docs.rs/emergent/1.5.0/emergent/consideration/trait.Consideration.html
[`ClosureConsideration`]: https://docs.rs/emergent/1.5.0/emergent/consideration/struct.ClosureConsideration.html
