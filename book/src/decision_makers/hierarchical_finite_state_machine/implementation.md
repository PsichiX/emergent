# How to implement

Happily there is not much work to do to turn FSM into HFSM - what we need to do
is to add `decide` method to `State` and implement `State` for `FSM` type so we
can just use it as states, also adapt existing code to process these networks as
states correctly.

Let's start with `State`:

```rust
trait State<T> {
  fn activated(&mut self, context: &mut T) {}
  // This method will be called whenever owning FSM doesn't change its state, so
  // we can use it to handle inner decision making - in case of FSM we call its
  // decision making here.
  fn decide(&mut self, context: &mut T) {}
  fn update(&mut self, context: &mut T) {}
}
```

Then we improve `FSM` to forward decision making and update calls to its children
networks:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Condition<T> = fn(&T) -> bool;
#
# trait State<T> {
#   fn activated(&mut self, context: &mut T) {}
#   fn decide(&mut self, context: &mut T) {}
#   fn update(&mut self, context: &mut T) {}
# }
#
# struct FSMTransition<K, T> {
#   to: K,
#   condition: Condition<T>,
# }
#
# struct FSMState<K, T> {
#   state: Box<dyn State<T>>,
#   transitions: Vec<FSMTransition<K, T>>,
# }
#
# impl<K, T> FSMState<K, T> {
#   fn new<S: State<T> + 'static>(state: S) -> Self {
#     Self {
#       state: Box::new(state),
#       transitions: vec![],
#     }
#   }
#
#   fn transition(mut self, to: K, condition: Condition<T>) -> Self {
#     self.transitions.push(FSMTransition {to, condition});
#     self
#   }
#
#   fn decide(&self, context: &T) -> Option<K> where K: Clone {
#     for transition in &self.transitions {
#       if (transition.condition)(context) {
#         return Some(transition.to.clone());
#       }
#     }
#     None
#   }
# }
#
struct FSM<K, T> {
  states: HashMap<K, FSMState<K, T>>,
  active_state: K,
  // This one will be used to reset active state when this FSM will get activated.
  initial_state: K,
}

impl<K: Hash + Eq, T> FSM<K, T> {
  fn new(active_state: K) -> Self where K: Clone {
    Self {
      states: Default::default(),
      initial_state: active_state.clone(),
      active_state,
    }
  }

# fn state(mut self, id: K, state: FSMState<K, T>) -> Self {
#   self.states.insert(id, state);
#   self
# }
#
  // We have added `forced` argument to be able to force change because from now
  // on state won't be activated if it's the same as currently active state.
  // User would call this method with `forced` set to true after FSM creation to
  // initialize newly created FSM.
  fn set_active_state(&mut self, id: K, context: &mut T, forced: bool) {
    if forced || id != self.active_state {
      if let Some(state) = self.states.get_mut(&id) {
        state.state.activated(context);
        self.active_state = id;
      }
    }
  }

  fn decide(&mut self, context: &mut T) where K: Clone {
    if let Some(state) = self.states.get_mut(&self.active_state) {
      if let Some(id) = state.decide(context) {
        self.set_active_state(id, context, false);
      } else {
        // From now on in case of FSM not having to change its state, we can tell
        // active state to optionally handle its decision making (this is useful
        // for nested FSMs).
        state.state.decide(context);
      }
    }
  }
#
# fn update(&mut self, context: &mut T) {
#   if let Some(state) = self.states.get_mut(&self.active_state) {
#     state.state.update(context);
#   }
# }
}
```

What's left to do is to setup FSM within enemy type.

First, for the sake of the tutorial, we create a simplified data types that will
describe enemy state:

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Target {
  None,
  Found,
  Reached,
}

struct EnemyData {
  waypoint: Target,
  player: Target,
}
```

`EnemyData` will contain information about its targets: waypoint and player.
`Target` will describe what we know about the target. In case of waypoint it can
be either Found or Reached since there is always some waypoint in the world.
In case of player None means player is dead, Found means player is in enemy range
and Reached means enemy is in contact range of player and can attack him.

Here are all possible states for all FSM hierarchy levels:

```rust
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum EnemyState {
  Patrol,
  Combat,
  FindWaypoint,
  WalkTowardsWaypoint,
  WalkTowardsPlayer,
  AttackPlayer,
}
```

Patrol and Combat are the root level FSM states and these will contain sub-networks.
FindWaypoint and WalkTowardsWaypoint belongs to Patrol FSM, WalkTowardsPlayer
and AttackPlayer belongs to Combat FSM.

Ok, now let's finally see the enemy FSM setup:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Condition<T> = fn(&T) -> bool;
#
# trait State<T> {
#   fn activated(&mut self, context: &mut T) {}
#   fn decide(&mut self, context: &mut T) {}
#   fn update(&mut self, context: &mut T) {}
# }
#
# struct FSMTransition<K, T> {
#   to: K,
#   condition: Condition<T>,
# }
#
# struct FSMState<K, T> {
#   state: Box<dyn State<T>>,
#   transitions: Vec<FSMTransition<K, T>>,
# }
#
# impl<K, T> FSMState<K, T> {
#   fn new<S: State<T> + 'static>(state: S) -> Self {
#     Self {
#       state: Box::new(state),
#       transitions: vec![],
#     }
#   }
#
#   fn transition(mut self, to: K, condition: Condition<T>) -> Self {
#     self.transitions.push(FSMTransition {to, condition});
#     self
#   }
#
#   fn decide(&self, context: &T) -> Option<K> where K: Clone {
#     for transition in &self.transitions {
#       if (transition.condition)(context) {
#         return Some(transition.to.clone());
#       }
#     }
#     None
#   }
# }
#
# struct FSM<K, T> {
#   states: HashMap<K, FSMState<K, T>>,
#   active_state: K,
#   initial_state: K,
# }
#
# impl<K: Hash + Eq, T> FSM<K, T> {
#   fn new(active_state: K) -> Self where K: Clone {
#     Self {
#       states: Default::default(),
#       initial_state: active_state.clone(),
#       active_state,
#     }
#   }
#
#   fn state(mut self, id: K, state: FSMState<K, T>) -> Self {
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
#     if let Some(state) = self.states.get_mut(&self.active_state) {
#       if let Some(id) = state.decide(context) {
#         self.set_active_state(id, context, false);
#       } else {
#         state.state.decide(context);
#       }
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
# impl<K: Clone + Hash + Eq, T> State<T> for FSM<K, T> {
#   fn activated(&mut self, context: &mut T) {
#     self.set_active_state(self.initial_state.clone(), context, true);
#   }
#
#   fn decide(&mut self, context: &mut T) {
#     self.decide(context);
#   }
#
#   fn update(&mut self, context: &mut T) {
#     self.update(context);
#   }
# }
#
# #[derive(Debug, Copy, Clone, PartialEq, Eq)]
# enum Target {
#   None,
#   Found,
#   Reached,
# }
#
# #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
# enum EnemyState {
#   Patrol,
#   Combat,
#   FindWaypoint,
#   WalkTowardsWaypoint,
#   WalkTowardsPlayer,
#   AttackPlayer,
# }
#
# struct EnemyData {
#   waypoint: Target,
#   player: Target,
# }
#
# struct Enemy {
#   data: EnemyData,
#   fsm: FSM<EnemyState, EnemyData>,
# }
#
impl Enemy {
  fn new() -> Self {
    let mut data = EnemyData {
      waypoint: Target::None,
      player: Target::None,
    };

    let patrol = FSM::new(EnemyState::FindWaypoint)
      .state(
        EnemyState::FindWaypoint,
        FSMState::new(EnemyFindWaypointState)
          .transition(EnemyState::WalkTowardsWaypoint, waypoint_found),
      )
      .state(
        EnemyState::WalkTowardsWaypoint,
        FSMState::new(EnemyWalkTowardsWaypointState)
          .transition(EnemyState::FindWaypoint, waypoint_reached),
      );

    let combat = FSM::new(EnemyState::WalkTowardsPlayer)
      .state(
        EnemyState::WalkTowardsPlayer,
        FSMState::new(EnemyWalkTowardsPlayerState)
          .transition(EnemyState::AttackPlayer, player_reached),
      )
      .state(
        EnemyState::AttackPlayer,
        FSMState::new(EnemyAttackPlayerState)
          .transition(EnemyState::WalkTowardsPlayer, player_found),
      );

    let mut fsm = FSM::new(EnemyState::Patrol)
      .state(
        EnemyState::Patrol,
        FSMState::new(patrol)
          .transition(EnemyState::Combat, player_found),
      )
      .state(
        EnemyState::Combat,
        FSMState::new(combat)
          .transition(EnemyState::Patrol, player_dead),
      );

    fsm.set_active_state(EnemyState::Patrol, &mut data, true);
    Self { data, fsm }
  }
#
# fn tick(&mut self) {
#   self.fsm.decide(&mut self.data);
#   self.fsm.update(&mut self.data);
# }
}
#
# fn waypoint_found(data: &EnemyData) -> bool {
#   data.waypoint == Target::Found
# }
#
# fn waypoint_reached(data: &EnemyData) -> bool {
#   data.waypoint == Target::Reached
# }
#
# fn player_found(data: &EnemyData) -> bool {
#   data.player == Target::Found
# }
#
# fn player_reached(data: &EnemyData) -> bool {
#   data.player == Target::Reached
# }
#
# fn player_dead(data: &EnemyData) -> bool {
#   data.player == Target::None
# }
#
# struct EnemyFindWaypointState;
#
# impl State<EnemyData> for EnemyFindWaypointState {}
#
# struct EnemyWalkTowardsWaypointState;
#
# impl State<EnemyData> for EnemyWalkTowardsWaypointState {}
#
# struct EnemyWalkTowardsPlayerState;
#
# impl State<EnemyData> for EnemyWalkTowardsPlayerState {}
#
# struct EnemyAttackPlayerState;
#
# impl State<EnemyData> for EnemyAttackPlayerState {}
```

All what we had to do was to put patrol and combat FSMs as we put states in root
FSM.

So how all of this setup changes enemy state in time:

```rust
# use std::{collections::HashMap, hash::Hash};
#
# type Condition<T> = fn(&T) -> bool;
#
# trait State<T> {
#   fn activated(&mut self, context: &mut T) {}
#   fn decide(&mut self, context: &mut T) {}
#   fn update(&mut self, context: &mut T) {}
# }
#
# struct FSMTransition<K, T> {
#   to: K,
#   condition: Condition<T>,
# }
#
# struct FSMState<K, T> {
#   state: Box<dyn State<T>>,
#   transitions: Vec<FSMTransition<K, T>>,
# }
#
# impl<K, T> FSMState<K, T> {
#   fn new<S: State<T> + 'static>(state: S) -> Self {
#     Self {
#       state: Box::new(state),
#       transitions: vec![],
#     }
#   }
#
#   fn transition(mut self, to: K, condition: Condition<T>) -> Self {
#     self.transitions.push(FSMTransition {to, condition});
#     self
#   }
#
#   fn decide(&self, context: &T) -> Option<K> where K: Clone {
#     for transition in &self.transitions {
#       if (transition.condition)(context) {
#         return Some(transition.to.clone());
#       }
#     }
#     None
#   }
# }
#
# struct FSM<K, T> {
#   states: HashMap<K, FSMState<K, T>>,
#   active_state: K,
#   initial_state: K,
# }
#
# impl<K: Hash + Eq, T> FSM<K, T> {
#   fn new(active_state: K) -> Self where K: Clone {
#     Self {
#       states: Default::default(),
#       initial_state: active_state.clone(),
#       active_state,
#     }
#   }
#
#   fn state(mut self, id: K, state: FSMState<K, T>) -> Self {
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
#     if let Some(state) = self.states.get_mut(&self.active_state) {
#       if let Some(id) = state.decide(context) {
#         self.set_active_state(id, context, false);
#       } else {
#         state.state.decide(context);
#       }
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
# impl<K: Clone + Hash + Eq, T> State<T> for FSM<K, T> {
#   fn activated(&mut self, context: &mut T) {
#     self.set_active_state(self.initial_state.clone(), context, true);
#   }
#
#   fn decide(&mut self, context: &mut T) {
#     self.decide(context);
#   }
#
#   fn update(&mut self, context: &mut T) {
#     self.update(context);
#   }
# }
#
# #[derive(Debug, Copy, Clone, PartialEq, Eq)]
# enum Target {
#   None,
#   Found,
#   Reached,
# }
#
# #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
# enum EnemyState {
#   Patrol,
#   Combat,
#   FindWaypoint,
#   WalkTowardsWaypoint,
#   WalkTowardsPlayer,
#   AttackPlayer,
# }
#
# struct EnemyData {
#   waypoint: Target,
#   player: Target,
# }
#
# struct Enemy {
#   data: EnemyData,
#   fsm: FSM<EnemyState, EnemyData>,
# }
#
# impl Enemy {
#   fn new() -> Self {
#     let mut data = EnemyData {
#       waypoint: Target::None,
#       player: Target::None,
#     };
#
#     let patrol = FSM::new(EnemyState::FindWaypoint)
#       .state(
#         EnemyState::FindWaypoint,
#         FSMState::new(EnemyFindWaypointState)
#           .transition(EnemyState::WalkTowardsWaypoint, waypoint_found),
#       )
#       .state(
#         EnemyState::WalkTowardsWaypoint,
#         FSMState::new(EnemyWalkTowardsWaypointState)
#           .transition(EnemyState::FindWaypoint, waypoint_reached),
#       );
#
#     let combat = FSM::new(EnemyState::WalkTowardsPlayer)
#       .state(
#         EnemyState::WalkTowardsPlayer,
#         FSMState::new(EnemyWalkTowardsPlayerState)
#           .transition(EnemyState::AttackPlayer, player_reached),
#       )
#       .state(
#         EnemyState::AttackPlayer,
#         FSMState::new(EnemyAttackPlayerState)
#           .transition(EnemyState::WalkTowardsPlayer, player_found),
#       );
#
#     let mut fsm = FSM::new(EnemyState::Patrol)
#       .state(
#         EnemyState::Patrol,
#         FSMState::new(patrol)
#           .transition(EnemyState::Combat, player_found),
#       )
#       .state(
#         EnemyState::Combat,
#         FSMState::new(combat)
#           .transition(EnemyState::Patrol, player_dead),
#       );
#
#     fsm.set_active_state(EnemyState::Patrol, &mut data, true);
#     Self { data, fsm }
#   }
#
#   fn tick(&mut self) {
#     self.fsm.decide(&mut self.data);
#     self.fsm.update(&mut self.data);
#   }
# }
#
# fn waypoint_found(data: &EnemyData) -> bool {
#   data.waypoint == Target::Found
# }
#
# fn waypoint_reached(data: &EnemyData) -> bool {
#   data.waypoint == Target::Reached
# }
#
# fn player_found(data: &EnemyData) -> bool {
#   data.player == Target::Found
# }
#
# fn player_reached(data: &EnemyData) -> bool {
#   data.player == Target::Reached
# }
#
# fn player_dead(data: &EnemyData) -> bool {
#   data.player == Target::None
# }
#
# struct EnemyFindWaypointState;
#
# impl State<EnemyData> for EnemyFindWaypointState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.waypoint = Target::Found;
#   }
# }
#
# struct EnemyWalkTowardsWaypointState;
#
# impl State<EnemyData> for EnemyWalkTowardsWaypointState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.waypoint = Target::Reached;
#   }
# }
#
# struct EnemyWalkTowardsPlayerState;
#
# impl State<EnemyData> for EnemyWalkTowardsPlayerState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.player = Target::Reached;
#   }
# }
#
# struct EnemyAttackPlayerState;
#
# impl State<EnemyData> for EnemyAttackPlayerState {
#   fn activated(&mut self, context: &mut EnemyData) {
#     context.player = Target::None;
#   }
# }
#
let mut enemy = Enemy::new();
enemy.data.waypoint = Target::Found;

enemy.tick();
assert_eq!(enemy.fsm.active_state, EnemyState::Patrol);
assert_eq!(enemy.data.waypoint, Target::Reached);
assert_eq!(enemy.data.player, Target::None);
enemy.tick();
assert_eq!(enemy.fsm.active_state, EnemyState::Patrol);
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::None);

enemy.data.player = Target::Found;

enemy.tick();
assert_eq!(enemy.fsm.active_state, EnemyState::Combat);
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::Reached);
enemy.tick();
assert_eq!(enemy.fsm.active_state, EnemyState::Combat);
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::None);
enemy.tick();
assert_eq!(enemy.fsm.active_state, EnemyState::Patrol);
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::None);
enemy.tick();
assert_eq!(enemy.fsm.active_state, EnemyState::Patrol);
assert_eq!(enemy.data.waypoint, Target::Reached);
assert_eq!(enemy.data.player, Target::None);
```
