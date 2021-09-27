# Use of Machinery decision maker from emergent crate

If you would like to just use existing solution for FSM, consider trying
[`emergent`] crate and its [`Machinery`] decision making engine:

```rust
# extern crate emergent;
# use emergent::prelude::*;
# use std::hash::Hash;
#
# #[derive(Debug, Copy, Clone, PartialEq, Eq)]
# enum Direction {
#   Up,
#   Down,
#   Left,
#   Right,
# }
#
# impl Direction {
#   fn horizontal(&self) -> isize {
#     match self {
#       Self::Left => -1,
#       Self::Right => 1,
#       _ => 0,
#     }
#   }
#
#   fn vertical(&self) -> isize {
#     match self {
#       Self::Up => -1,
#       Self::Down => 1,
#       _ => 0,
#     }
#   }
#
#   fn next(&self) -> Self {
#     match self {
#       Self::Up => Self::Right,
#       Self::Down => Self::Left,
#       Self::Left => Self::Up,
#       Self::Right => Self::Down,
#     }
#   }
# }
#
# #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
# enum EnemyState {
#   Wait,
#   Move,
#   ChangeDirection,
# }
#
# struct EnemyData {
#   position: (isize, isize),
#   direction: Direction,
#   turns: usize,
# }
#
struct WaitTask(pub usize);

// Tasks are units that do the actual work of the state.
impl Task<EnemyData> for WaitTask {
  // While task is locked, FSM won't change to another state even if it can.
  // We lock this task for the time there are turns left.
  fn is_locked(&self, memory: &EnemyData) -> bool {
    memory.turns > 0
  }

  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.turns = self.0;
  }

  fn on_update(&mut self, memory: &mut EnemyData) {
    memory.turns = memory.turns.max(1) - 1;
  }
}

struct MoveTask(pub usize);

impl Task<EnemyData> for MoveTask {
  fn is_locked(&self, memory: &EnemyData) -> bool {
    memory.turns > 0
  }

  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.turns = self.0;
  }

  fn on_update(&mut self, memory: &mut EnemyData) {
    if memory.turns > 0 {
      memory.turns -= 1;
      memory.position.0 += memory.direction.horizontal();
      memory.position.1 += memory.direction.vertical();
    }
  }
}

struct ChangeDirectionTask;

impl Task<EnemyData> for ChangeDirectionTask {
  fn on_enter(&mut self, memory: &mut EnemyData) {
    memory.direction = memory.direction.next();
  }
}

struct Enemy {
  data: EnemyData,
  machinery: Machinery<EnemyData, EnemyState>,
}

impl Enemy {
  fn new(x: isize, y: isize, direction: Direction) -> Self {
    let mut data = EnemyData {
      position: (x, y),
      direction,
      turns: 0,
    };
    let mut machinery = MachineryBuilder::default()
      .state(
        EnemyState::Wait,
        MachineryState::task(WaitTask(1))
          // In `emergent` Conditions are traits that are implemented also for
          // booleans, which means we can just use constants as conditions so
          // here we make this transition always passing and the state locking
          // controls how long task will run.
          .change(MachineryChange::new(EnemyState::ChangeDirection, true)),
      )
      .state(
        EnemyState::Move,
        MachineryState::task(MoveTask(2))
          .change(MachineryChange::new(EnemyState::Wait, true)),
      )
      .state(
        EnemyState::ChangeDirection,
        MachineryState::task(ChangeDirectionTask)
          .change(MachineryChange::new(EnemyState::Move, true)),
      )
      .build();
    // Newly created decision makers doesn't have any state activated and since
    // FSM can change its states starting from active state, we need to activate
    // first state by ourself.
    machinery.change_active_state(
      Some(EnemyState::ChangeDirection),
      &mut data,
      true,
    );

    Self { data, machinery }
  }

  fn tick(&mut self) {
    // `process` method performs decision making.
    self.machinery.process(&mut self.data);
    self.machinery.update(&mut self.data);
  }
}

let mut enemy = Enemy::new(0, 0, Direction::Up);

assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::ChangeDirection));
assert_eq!(enemy.data.position.0, 0);
assert_eq!(enemy.data.position.1, 0);
assert_eq!(enemy.data.direction, Direction::Right);
for i in 1..3 {
  enemy.tick();
  assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Move));
  assert_eq!(enemy.data.position.0, i);
  assert_eq!(enemy.data.position.1, 0);
  assert_eq!(enemy.data.direction, Direction::Right);
}
enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::Wait));
assert_eq!(enemy.data.position.0, 2);
assert_eq!(enemy.data.position.1, 0);
assert_eq!(enemy.data.direction, Direction::Right);
enemy.tick();
assert_eq!(enemy.machinery.active_state(), Some(&EnemyState::ChangeDirection));
assert_eq!(enemy.data.position.0, 2);
assert_eq!(enemy.data.position.1, 0);
assert_eq!(enemy.data.direction, Direction::Down);
```

[`emergent`]: https://crates.io/emergent
[`Machinery`]: https://docs.rs/emergent/1.5.0/emergent/decision_makers/machinery/struct.Machinery.html
