# How it works

Imagine we start from __Change Direction__ state and we tell FSM to make a
decision, it looks at transitions pointing from this state, here there is only
one: __Move__ wit condition: __Just Do It__ (which here it means: it always
succeeds). Since this tansition condition succeeded, FMS changes its active state
to __Move__ state and enemy moves forward for few turns in the direction previously
set by __Change Direction__ state.

In the mean time we run decision making from time to time (can be each frame,
can be each second - usually decision making runs at slower frequency than game
frames, or sometimes it is triggered only when decision making engine receives
an event that tells that important change in game state happened and FSM should
make a decision about its new state).

At some point when FSM runs its decision making, goes through __Move__ state
transitions and finds out that condition of a transition pointing to __Wait__
state reports no more move turns, so FSM changes its active state to __Wait__
which will wait few turns doing nothing.

When there is no more waiting turns left, FSM switches back to __Change Direction__
state, which all of it makes "Change Direction, Move, Wait" behavior of our Enemy.
Really simple, right?
