# How it works

Rememeber that HFSM is just a FSM that can use other FSM objects as states, hence
_hierarchy_ in the name - we compose networks in other networks.

We have a hierarchy of states that looks like this:
- Patrol
  - Find Next Waypoint
  - Walk Towards Waypoint
- Combat
  - Walk towards player
  - Attack player

Each network layer doesn't know anything about other networks, they are fully
encapsulated. We start at first network layer which has __Patrol__ and __Combat__
states and start with __Patrol__ state active, it gets activated and we start
executing __Patrol__ network which has __Find Next Waypoint__ state (we start
here) and __Walk Towards Waypoint__ state. We have found waypoint so we switch
to __Walk Towards Waypoint__ state. Notice that while we are executing __Patrol__
network we are still executing root network too, which means at any time we can
get player in range condition succeed and we switch at root level to __Combat__
network, then there it starts at __Walk Towards Player__, it reaches the player,
switches to __Attack Player__, makes player dead and rot network switches back
to __Patrol__ network and its __Find Next Waypoint__ and goes on like that.

The key information is:

> In HFSM, FSM networks are equivalent to states so as long as network is active,
> it will process its states, and if we get networks tree many levels deep, all
> nodes (FMS networks) that are along the way from the root to the leaf, are
> active and they can make decisions.

This allows us to modularize our previously made FSM even more, making us able
to group states into reusable "categories" instead of copy-paste bits here and
there - think of it as nested prefabs for behaviors.
