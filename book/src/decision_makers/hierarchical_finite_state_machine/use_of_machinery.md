# Hierarchical use of Machinery decision maker from emergent crate

By default all [`emergent`] decision makers are designed to be used in hierarchies
so building HFSM is really simple - you just put one [`Machinery`] as a state in
another. What's new comparing to the flat Machinery setup is we have to assign
initial state decision maker that whenever Machinery gets activated, it will
activate some starting state for that Machinery.

```rust
# extern crate emergent;
# use emergent::prelude::*;
# use std::hash::Hash;
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
#   machinery: Machinery<EnemyData, EnemyState>,
# }
#
impl Enemy {
  fn new() -> Self {
    let mut data = EnemyData {
      waypoint: Target::None,
      player: Target::None,
    };

    let patrol = MachineryBuilder::default()
      .state(
        EnemyState::FindWaypoint,
        MachineryState::task(EnemyFindWaypointTask)
          .change(MachineryChange::new(
            EnemyState::WalkTowardsWaypoint,
            ClosureCondition::new(waypoint_found),
          )),
      )
      .state(
        EnemyState::WalkTowardsWaypoint,
        MachineryState::task(EnemyWalkTowardsWaypointTask)
          .change(MachineryChange::new(
            EnemyState::FindWaypoint,
            ClosureCondition::new(waypoint_reached),
          )),
      )
      .build()
      // We assign initial state decision maker if we want to make sure that
      // whenever machinery gets activated it will start at some state (useful
      // when building hierarchies).
      .initial_state_decision_maker(
        SingleDecisionMaker::new(EnemyState::FindWaypoint),
      );

    let combat = MachineryBuilder::default()
      .state(
        EnemyState::WalkTowardsPlayer,
        MachineryState::task(EnemyWalkTowardsPlayerTask)
          .change(MachineryChange::new(
            EnemyState::AttackPlayer,
            ClosureCondition::new(player_reached),
          )),
      )
      .state(
        EnemyState::AttackPlayer,
        MachineryState::task(EnemyAttackPlayerTask)
          .change(MachineryChange::new(
            EnemyState::WalkTowardsPlayer,
            ClosureCondition::new(player_found),
          )),
      )
      .build()
      .initial_state_decision_maker(
        SingleDecisionMaker::new(EnemyState::WalkTowardsPlayer),
      );

    let mut machinery = MachineryBuilder::default()
      .state(
        EnemyState::Patrol,
        MachineryState::task(patrol)
          .change(MachineryChange::new(
            EnemyState::Combat,
            ClosureCondition::new(player_found),
          )),
      )
      .state(
        EnemyState::Combat,
        MachineryState::task(combat)
          .change(MachineryChange::new(
            EnemyState::Patrol,
            ClosureCondition::new(player_dead),
          )),
      )
      .build()
      .initial_state_decision_maker(
        SingleDecisionMaker::new(EnemyState::Patrol),
      );

    // since we have assigned initial state decision maker we can activate root
    // machinery to activate its initial state.
    machinery.on_enter(&mut data);
    Self { data, machinery }
  }

  fn tick(&mut self) {
    self.machinery.process(&mut self.data);
    self.machinery.update(&mut self.data);
  }
}

// Condition functions that are used in testing possible state changes:

fn waypoint_found(data: &EnemyData) -> bool {
  data.waypoint == Target::Found
}

fn waypoint_reached(data: &EnemyData) -> bool {
  data.waypoint == Target::Reached
}

fn player_found(data: &EnemyData) -> bool {
  data.player == Target::Found
}

fn player_reached(data: &EnemyData) -> bool {
  data.player == Target::Reached
}

fn player_dead(data: &EnemyData) -> bool {
  data.player == Target::None
}

// Enemy state tasks that will process enemy data:

struct EnemyFindWaypointTask;

impl Task<EnemyData> for EnemyFindWaypointTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.waypoint = Target::Found;
  }
}

struct EnemyWalkTowardsWaypointTask;

impl Task<EnemyData> for EnemyWalkTowardsWaypointTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.waypoint = Target::Reached;
  }
}

struct EnemyWalkTowardsPlayerTask;

impl Task<EnemyData> for EnemyWalkTowardsPlayerTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.player = Target::Reached;
  }
}

struct EnemyAttackPlayerTask;

impl Task<EnemyData> for EnemyAttackPlayerTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.player = Target::None;
  }
}

let mut enemy = Enemy::new();

enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Patrol));
assert_eq!(enemy.data.waypoint, Target::Reached);
assert_eq!(enemy.data.player, Target::None);
enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Patrol));
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::None);

enemy.data.player = Target::Found;

enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Combat));
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::Reached);
enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Combat));
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::None);
enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Patrol));
assert_eq!(enemy.data.waypoint, Target::Found);
assert_eq!(enemy.data.player, Target::None);
enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Patrol));
assert_eq!(enemy.data.waypoint, Target::Reached);
assert_eq!(enemy.data.player, Target::None);
```

[`emergent`]: https://crates.io/emergent
[`Machinery`]: https://docs.rs/emergent/1.3.0/emergent/decision_makers/machinery/struct.Machinery.html
