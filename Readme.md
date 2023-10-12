
I made other CS mods, but because the engine I used (CSE2) has been DMCA'd, I cannot share them publicly.

But now...
I'm going to try and learn how to use rust.

There is a CS engine written in rust that *hasn't* been DMCA'd, so I *can* share that publicly.
Here it is.


---
# Personal Notes:

to slow the game down for debugging, increase the wait value in shared_game_state.rs:
(impl TimingMode {})


common.rs contains typedefs for common items

player_hit contains collision code for the PC only



## Notes about issues with the rust engine:

* stage.change_tile() doesn't generate smoke
* toroko NPC is broken (bubble thing, fixed by someone else, my build is out of date)
* ironhead fastScroll jitters
* stage backgrounds do not snap to grid even if the option is enabled *(note about background jitter: this is processesed outside game ticks, so interlopation is not always correct, hence the jitter)*


Fixed:
* inventory selector blinks too fast
* inventory runs TSC more than once when left/right is pressed

* quote's gun bobbing can desync when spamming "up"
when quote looks up, his animation rect always defaults to the first step in his animation (this is default dehavior).
this reset is not applied to the gun.
the big problem is that some behaviors are tied to an NPC-like animation_no, while others are tied to a PlayerAnimationState.
(reason: self.anim_counter is not reset when the state changes, but the animationState is).
