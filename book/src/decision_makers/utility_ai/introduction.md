# Utility AI

In previous chapters we have been talking about FSM and HFSM as a way to build
small, yet easy to manage AI systems. These two and BT (Behavior Trees) have one
benefit that is also their drawback - they have predictable and fixed transitions.

At some point you might have been wondering: "__Is it possible to create an AI
system where transitions between states are rather fluid, and not fixed?__".
You have been actually starting to question the possibility of emergent gameplay,
where AI behaviors are less predictable and more driven by the changes in the
environment and world itself.

When you aim for emergent behavior of either your AI agents or game events, you
might find yourself struggling with defining these with FSM/HFSM or BT, so let
me introduce you to __Utility AI__ system:

![Utility](../../images/utility.svg)

Quick explanation of the terms:

- __State__ - We have already explained it with other AI systems.
- __Consideration__ - Scores probability of _someting_ (really vague explanation,
  we will explain that more later).
- __Condition__ - Tells if some fact is true.
- __Score Mapping__ - Its job is to map one or more scores into other single score.

As you can see, there are no transitions between states, so how all of these units
together decide which state agent should change into?

> Whenever decision has to be made, Utility AI goes through all its states and
> scores probability of each state. State with the highest score wins and gets
> selected.

I consider this to be the simplest rule of all AI systems covered in this book.
While transition rule is simple, scoring process might get a little bit tricky.
In the next chapter we will explain how scoring process works.
