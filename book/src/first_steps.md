# First steps

First of all: what do we mean by the term AI?
---
When we talk about AI for games, we are in fact talking about all decision making
units of logic.

It's improtant to understand that decision making, or AI, isn't suited only for
agents in the game - it can be and already is widely used for animation systems,
events happening in game world, or even to control sounds and music.

You are probably already making an AI system without even noticing it right now!
---
For a moment let's try to write a simple enemy wandering code the way we usually do.

```rust
enum Direction {
  Up,
  Down,
  Left,
  Right,
}

impl Direction {
  fn horizontal(&self) -> isize {
    match self {
      Self::Left => -1,
      Self::Right => 1,
      _ => 0,
    }
  }

  fn vertical(&self) -> isize {
    match self {
      Self::Up => -1,
      Self::Down => 1,
      _ => 0,
    }
  }

  fn next(&self) -> Self {
    match self {
      Self::Up => Self::Right,
      Self::Down => Self::Left,
      Self::Left => Self::Up,
      Self::Right => Self::Down,
    }
  }
}

struct Enemy {
  position: (isize, isize),
  direction: Direction,
  change_direction_turns: usize,
  wait_turns: usize,
}

impl Enemy {
  fn new(x: isize, y: isize) -> Self {
    Self {
      position: (x, y),
      direction: Direction::Up,
      change_direction_turns: 0,
      wait_turns: 0,
    }
  }

  fn update(&mut self) {
    if self.wait_turns > 0 {
      self.wait_turns -= 1;
    } else if self.change_direction_turns > 0 {
      self.change_direction_turns -= 1;
      self.position.0 += self.direction.horizontal();
      self.position.1 += self.direction.vertical();
    } else {
      self.direction = self.direction.next();
      self.change_direction_turns = 4;
      self.wait_turns = 2;
    }
  }
}
```

You can notice `update` function - this is what controls this enemy simple AI.
You might not have noticed it but you have used Finite State Machine for this
problem, hardcore way thought.

Btw. That snippet was really hard to read, right? Yeah, that's how our naive
"simple" AI implementations usually end up growing in complexity.

We can do better, let me refactor it to prove it to you:

```rust
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
enum State {
  Wait(usize),
  ChangeDirection,
  Move(usize),
}

struct Enemy {
  position: (isize, isize),
  direction: Direction,
  state: State,
}

impl Enemy {
  fn new(x: isize, y: isize) -> Self {
    Self {
      position: (x, y),
      direction: Direction::Up,
      state: State::ChangeDirection,
    }
  }

  fn update(&mut self) {
    match &mut self.state {
      State::Wait(turns) => {
        if *turns > 0 {
          *turns -= 1;
        } else {
          self.state = State::ChangeDirection;
        }
      },
      State::ChangeDirection => {
        self.direction = self.direction.next();
        self.state = State::Move(4);
      },
      State::Move(turns) => {
        if *turns > 0 {
          *turns -= 1;
          self.position.0 += self.direction.horizontal();
          self.position.1 += self.direction.vertical();
        } else {
          self.state = State::Wait(2);
        }
      },
    }
  }
}
```

Now we can clearly see what is the actual behavior of this AI:
- wait few turns
- change direction
- move forward for few turns

And we have got rid of a bug where our previous magic didn't actually do what we
aimed for the way we wanted it to do. This is the simplest state machine we have
learned to make, most likely at this point you're already making them this way.

Now imagine we get more states to handle, like shooting, taking cover when on
low health, finding ammo when it ends - number of state changes and branching
grows exponentially with number of states and our code starts to look more like
a spaghetti than code with clear and easily understandable intent. Have you
encountered that frustration yet?

This is the exact reason why AI systems have been invented in first place - to
simplify readability of and iteration over AI logic.

In next chapters we will learn about AI systems and gradually work our way out
from naive AI logic to more managable solutions.
