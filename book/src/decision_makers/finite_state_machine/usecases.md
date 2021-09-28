# Typical use cases

- __Simple deterministic AI behavior__

  Indie games with small number of AI agent states, usually platformers or
  fast-paced games like shooters or bullet hells - in general games that doesn't
  need long-term or strategic planning and they have to make decision fast to
  adapt to quickly changed environment.

- __Animations__

  All modern game engines use FSM to manage animations, transition between them
  also contains information about blending between states to not make states snap
  from one to another instantly.

- __Game states / game modes__

  Imagine you have a game session with several states: Overworld, Battle, Shop,
  Crafting. You might use FSM to manage changes between these different game modes.
