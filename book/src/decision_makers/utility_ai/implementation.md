# How to implement

First we have to define alias types for `Score` and `Scorer`:

```rust
type Score = f32;
type Scorer<T> = fn(&T) -> Score;
```

We aliased `f32` as `Score` in case we would want to change it to `f64` at some
point and be able to perform that change only in alias declaration.

`Scorer` is just an alias to function that will take context as input and produce
score as output, used later as state probability score for decision making.

`State` trait stays the same as what we have made for FSM.

Next we define an `UtilityState` that will store an actual state and scorer:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Score = f32;
# type Scorer<T> = fn(&T) -> Score;
#
# trait State<T> {}
#
struct UtilityState<T> {
  state: Box<dyn State<T>>,
  scorer: Scorer<T>,
}
```

Now let's implement `Utility` decision maker:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Score = f32;
# type Scorer<T> = fn(&T) -> Score;
#
# trait State<T> {
#   fn activated(&mut self, &mut T) {}
#   fn decide(&mut self, &mut T) {}
#   fn update(&mut self, &mut T) {}
# }
#
# struct UtilityState<T> {
#   state: Box<dyn State<T>>,
#   scorer: Scorer<T>,
# }
#
struct Utility<K, T> {
  states: HashMap<K, UtilityState<T>>,
  active_state: K,
}

impl<K: Hash + Eq, T> Utility<K, T> {
  fn new(active_state: K) -> Self {
    Self {
      states: Default::default(),
      active_state,
    }
  }

  fn state<S>(mut self, id: K, state: S, scorer: Scorer<T>) -> Self
  where
    S: State<T> + 'static,
  {
    let state = UtilityState {
      state: Box::new(state),
      scorer,
    };
    self.states.insert(id, state);
    self
  }

  fn set_active_state(&mut self, id: K, context: &mut T, forced: bool) {
    if forced || id != self.active_state {
      if let Some(state) = self.states.get_mut(&id) {
        state.state.activated(context);
        self.active_state = id;
      }
    }
  }

  fn decide(&mut self, context: &mut T) where K: Clone {
    let winner = self
      .states
      .iter()
      .map(|(k, v)| (k, (v.scorer)(context)))
      .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
      .map(|(id, _)| id.clone());
    if let Some(id) = winner {
      self.set_active_state(id, context, false);
    }
    if let Some(state) = self.states.get_mut(&self.active_state) {
      state.state.decide(context);
    }
  }

  fn update(&mut self, context: &mut T) {
    if let Some(state) = self.states.get_mut(&self.active_state) {
      state.state.update(context);
    }
  }
}
```

Methods: `new`, `set_active_state` and `update` arent much different from what
we have done for FSM, but `decide` method changes completely, obviously.

It performs iterator operations suck as:
- First map each state ID and `UtilityState` into state ID and state score (by
  executing scorer function with the context).
- Then it finds an item wth the highest score using `max_by` iterator operation.
- And finally it maps result to get only found state ID.

Then if there is winner state ID found, sets active state to one selected (it
will change state only if new one is different than currently active state).

At the end it calls `decide` method on currently active state, to properly
propagate decision making in case we use `Utility` in hierarchy just like we did
with HFSM (yes, we are making `Utility` support hierarchies).

We also need to implement `State` trait for `Utility` type:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Score = f32;
# type Scorer<T> = fn(&T) -> Score;
#
# trait State<T> {
#   fn activated(&mut self, &mut T) {}
#   fn decide(&mut self, &mut T) {}
#   fn update(&mut self, &mut T) {}
# }
#
# struct UtilityState<T> {
#   state: Box<dyn State<T>>,
#   scorer: Scorer<T>,
# }
#
# struct Utility<K, T> {
#   states: HashMap<K, UtilityState<T>>,
#   active_state: K,
# }
#
# impl<K: Hash + Eq, T> Utility<K, T> {
#   fn decide(&mut self, context: &mut T) where K: Clone {}
#   fn update(&mut self, context: &mut T) {}
# }
#
impl<K: Clone + Hash + Eq, T> State<T> for Utility<K, T> {
  fn decide(&mut self, context: &mut T) {
    self.decide(context);
  }

  fn update(&mut self, context: &mut T) {
    self.update(context);
  }
}
```

Ok, so we have completed `Utility` decision maker implementation, we can now make
state ID and data types for enemy:

```rust
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum EnemyState {
  Idle,
  GatherFood,
  GatherWood,
  AttackOpponent,
}

struct EnemyData {
  hunger: f32,
  distance_to_food: f32,
  distance_to_trees: f32,
  wood_needed: usize,
  distance_to_opponent: f32,
  opponent_strength: f32,
}
```

We keep these types same as what we have defined in [explanation chapter](./explanation.md).

Now take a look how we can translate scorer operations for these states into
actual scoring functions:

```rust
# type Score = f32;
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
fn scorer_idle(context: &EnemyData) -> Score {
  0.001
}

fn scorer_gather_food(context: &EnemyData) -> Score {
  context.hunger * (1.0 - context.distance_to_food)
}

fn scorer_gather_wood(context: &EnemyData) -> Score {
  let wood_needed = if context.wood_needed > 0 { 1.0 } else { 0.0 };
  wood_needed * (1.0 - context.distance_to_trees)
}

fn scorer_attack_opponent(context: &EnemyData) -> Score {
  (1.0 - context.distance_to_opponent) + context.opponent_strength
}
```

Obviously if we would like to reuse parts of these scorers logic we can put these
operations into separate functions, the point is that all what matters is to
produce a single score, you can organize your scorers as you wish.

Next step is to implement enemy states that will mutate enemy data to reflect
changes in the world:

```rust
# type Score = f32;
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
# trait State<T> {
#   fn activated(&mut self, &mut T) {}
#   fn decide(&mut self, &mut T) {}
#   fn update(&mut self, &mut T) {}
# }
#
struct EnemyIdleState;

impl State<EnemyData> for EnemyIdleState {}

struct EnemyGatherFoodState;

impl State<EnemyData> for EnemyGatherFoodState {
  fn activated(&mut self, context: &mut EnemyData) {
    context.hunger = 0.0;
    context.distance_to_food = 1.0;
  }
}

struct EnemyGatherWoodState;

impl State<EnemyData> for EnemyGatherWoodState {
  fn activated(&mut self, context: &mut EnemyData) {
    context.wood_needed = context.wood_needed.max(1) - 1;
    context.distance_to_trees = 1.0;
  }
}

struct EnemyAttackOpponentState;

impl State<EnemyData> for EnemyAttackOpponentState {
  fn activated(&mut self, context: &mut EnemyData) {
    context.distance_to_opponent = 1.0;
    context.opponent_strength = 0.0;
  }
}
```

Finally we implement `Enemy` type itself and its `Utility` decision maker setup:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Score = f32;
# type Scorer<T> = fn(&T) -> Score;
#
# trait State<T> {
#   fn activated(&mut self, &mut T) {}
#   fn decide(&mut self, &mut T) {}
#   fn update(&mut self, &mut T) {}
# }
#
# struct UtilityState<T> {
#   state: Box<dyn State<T>>,
#   scorer: Scorer<T>,
# }
#
# struct Utility<K, T> {
#   states: HashMap<K, UtilityState<T>>,
#   active_state: K,
# }
#
# impl<K: Hash + Eq, T> Utility<K, T> {
#   fn new(active_state: K) -> Self {
#     Self {
#       states: Default::default(),
#       active_state,
#     }
#   }
#
#   fn state<S>(mut self, id: K, state: S, scorer: Scorer<T>) -> Self
#   where
#     S: State<T> + 'static,
#   {
#     let state = UtilityState {
#       state: Box::new(state),
#       scorer,
#     };
#     self.states.insert(id, state);
#     self
#   }
#
#   fn set_active_state(&mut self, id: K, context: &mut T, forced: bool) {
#     if forced || id != self.active_state {
#       if let Some(state) = self.states.get_mut(&id) {
#         state.state.activated(context);
#         self.active_state = id;
#       }
#     }
#   }
#
#   fn decide(&mut self, context: &mut T) where K: Clone {
#     let winner = self
#       .states
#       .iter()
#       .map(|(k, v)| (k, (v.scorer)(context)))
#       .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
#       .map(|(id, _)| id.clone());
#     if let Some(id) = winner {
#       self.set_active_state(id, context, false);
#     }
#     if let Some(state) = self.states.get_mut(&self.active_state) {
#       state.state.decide(context);
#     }
#   }
#
#   fn update(&mut self, context: &mut T) {
#     if let Some(state) = self.states.get_mut(&self.active_state) {
#       state.state.update(context);
#     }
#   }
# }
#
# impl<K: Clone + Hash + Eq, T> State<T> for Utility<K, T> {
#   fn decide(&mut self, context: &mut T) {
#     self.decide(context);
#   }
#
#   fn update(&mut self, context: &mut T) {
#     self.update(context);
#   }
# }
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
  utility: Utility<EnemyState, EnemyData>,
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
    let utility = Utility::new(EnemyState::Idle)
      .state(EnemyState::Idle, EnemyIdleState, scorer_idle)
      .state(EnemyState::GatherFood, EnemyGatherFoodState, scorer_gather_food)
      .state(EnemyState::GatherWood, EnemyGatherWoodState, scorer_gather_wood)
      .state(
        EnemyState::AttackOpponent,
        EnemyAttackOpponentState,
        scorer_attack_opponent,
      );

    Self { data, utility }
  }

  fn tick(&mut self) {
    self.utility.decide(&mut self.data);
    self.utility.update(&mut self.data);
  }
}
#
# fn scorer_idle(context: &EnemyData) -> Score {
#   0.001
# }
#
# fn scorer_gather_food(context: &EnemyData) -> Score {
#   context.hunger * (1.0 - context.distance_to_food)
# }
#
# fn scorer_gather_wood(context: &EnemyData) -> Score {
#   let wood_needed = if context.wood_needed > 0 { 1.0 } else { 0.0 };
#   wood_needed * (1.0 - context.distance_to_trees)
# }
#
# fn scorer_attack_opponent(context: &EnemyData) -> Score {
#   (1.0 - context.distance_to_opponent) + context.opponent_strength
# }
#
# struct EnemyIdleState;
#
# impl State<EnemyData> for EnemyIdleState {}
#
# struct EnemyGatherFoodState;
#
# impl State<EnemyData> for EnemyGatherFoodState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.hunger = 0.0;
#     context.distance_to_food = 1.0;
#   }
# }
#
# struct EnemyGatherWoodState;
#
# impl State<EnemyData> for EnemyGatherWoodState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.wood_needed = context.wood_needed.max(1) - 1;
#     context.distance_to_trees = 1.0;
#   }
# }
#
# struct EnemyAttackOpponentState;
#
# impl State<EnemyData> for EnemyAttackOpponentState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.distance_to_opponent = 1.0;
#     context.opponent_strength = 0.0;
#   }
# }
```

And what's left is to test how enemy states react to changes in the world:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Score = f32;
# type Scorer<T> = fn(&T) -> Score;
#
# trait State<T> {
#   fn activated(&mut self, &mut T) {}
#   fn decide(&mut self, &mut T) {}
#   fn update(&mut self, &mut T) {}
# }
#
# struct UtilityState<T> {
#   state: Box<dyn State<T>>,
#   scorer: Scorer<T>,
# }
#
# struct Utility<K, T> {
#   states: HashMap<K, UtilityState<T>>,
#   active_state: K,
# }
#
# impl<K: Hash + Eq, T> Utility<K, T> {
#   fn new(active_state: K) -> Self {
#     Self {
#       states: Default::default(),
#       active_state,
#     }
#   }
#
#   fn state<S>(mut self, id: K, state: S, scorer: Scorer<T>) -> Self
#   where
#     S: State<T> + 'static,
#   {
#     let state = UtilityState {
#       state: Box::new(state),
#       scorer,
#     };
#     self.states.insert(id, state);
#     self
#   }
#
#   fn set_active_state(&mut self, id: K, context: &mut T, forced: bool) {
#     if forced || id != self.active_state {
#       if let Some(state) = self.states.get_mut(&id) {
#         state.state.activated(context);
#         self.active_state = id;
#       }
#     }
#   }
#
#   fn decide(&mut self, context: &mut T) where K: Clone {
#     let winner = self
#       .states
#       .iter()
#       .map(|(k, v)| (k, (v.scorer)(context)))
#       .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
#       .map(|(id, _)| id.clone());
#     if let Some(id) = winner {
#       self.set_active_state(id, context, false);
#     }
#     if let Some(state) = self.states.get_mut(&self.active_state) {
#       state.state.decide(context);
#     }
#   }
#
#   fn update(&mut self, context: &mut T) {
#     if let Some(state) = self.states.get_mut(&self.active_state) {
#       state.state.update(context);
#     }
#   }
# }
#
# impl<K: Clone + Hash + Eq, T> State<T> for Utility<K, T> {
#   fn decide(&mut self, context: &mut T) {
#     self.decide(context);
#   }
#
#   fn update(&mut self, context: &mut T) {
#     self.update(context);
#   }
# }
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
# struct Enemy {
#   data: EnemyData,
#   utility: Utility<EnemyState, EnemyData>,
# }
#
# impl Enemy {
#   fn new() -> Self {
#     let data = EnemyData {
#       hunger: 0.0,
#       distance_to_food: 1.0,
#       distance_to_trees: 1.0,
#       wood_needed: 0,
#       distance_to_opponent: 1.0,
#       opponent_strength: 0.0,
#     };
#     let utility = Utility::new(EnemyState::Idle)
#       .state(EnemyState::Idle, EnemyIdleState, scorer_idle)
#       .state(EnemyState::GatherFood, EnemyGatherFoodState, scorer_gather_food)
#       .state(EnemyState::GatherWood, EnemyGatherWoodState, scorer_gather_wood)
#       .state(
#         EnemyState::AttackOpponent,
#         EnemyAttackOpponentState,
#         scorer_attack_opponent,
#       );
#
#     Self { data, utility }
#   }
#
#   fn tick(&mut self) {
#     self.utility.decide(&mut self.data);
#     self.utility.update(&mut self.data);
#   }
# }
#
# fn scorer_idle(context: &EnemyData) -> Score {
#   0.001
# }
#
# fn scorer_gather_food(context: &EnemyData) -> Score {
#   context.hunger * (1.0 - context.distance_to_food)
# }
#
# fn scorer_gather_wood(context: &EnemyData) -> Score {
#   let wood_needed = if context.wood_needed > 0 { 1.0 } else { 0.0 };
#   wood_needed * (1.0 - context.distance_to_trees)
# }
#
# fn scorer_attack_opponent(context: &EnemyData) -> Score {
#   (1.0 - context.distance_to_opponent) + context.opponent_strength
# }
#
# struct EnemyIdleState;
#
# impl State<EnemyData> for EnemyIdleState {}
#
# struct EnemyGatherFoodState;
#
# impl State<EnemyData> for EnemyGatherFoodState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.hunger = 0.0;
#     context.distance_to_food = 1.0;
#   }
# }
#
# struct EnemyGatherWoodState;
#
# impl State<EnemyData> for EnemyGatherWoodState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.wood_needed = context.wood_needed.max(1) - 1;
#     context.distance_to_trees = 1.0;
#   }
# }
#
# struct EnemyAttackOpponentState;
#
# impl State<EnemyData> for EnemyAttackOpponentState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.distance_to_opponent = 1.0;
#     context.opponent_strength = 0.0;
#   }
# }
#
let mut enemy = Enemy::new();
assert_eq!(enemy.utility.active_state, EnemyState::Idle);
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
assert_eq!(enemy.utility.active_state, EnemyState::AttackOpponent);
assert_eq!(enemy.data.hunger, 0.5);
assert_eq!(enemy.data.distance_to_food, 0.9);
assert_eq!(enemy.data.distance_to_trees, 0.5);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.tick();
assert_eq!(enemy.utility.active_state, EnemyState::GatherFood);
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 0.5);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.data.wood_needed = 1;

enemy.tick();
assert_eq!(enemy.utility.active_state, EnemyState::GatherWood);
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 1.0);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);

enemy.tick();
assert_eq!(enemy.utility.active_state, EnemyState::Idle);
assert_eq!(enemy.data.hunger, 0.0);
assert_eq!(enemy.data.distance_to_food, 1.0);
assert_eq!(enemy.data.distance_to_trees, 1.0);
assert_eq!(enemy.data.wood_needed, 0);
assert_eq!(enemy.data.distance_to_opponent, 1.0);
assert_eq!(enemy.data.opponent_strength, 0.0);
```

As you can see, all state changes are completely environment-driven - no fixed
transitions, fully emergent behavior.

Not much more is needed to be explained because most implmentation is very similar
if not the same, as with FSM, we just had to implement scoring feature for decision
making - that's how easy is to move from FSM to Utility AI!
