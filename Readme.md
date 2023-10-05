
I made other CS mods, but because the engine I used (CSE2) has been DMCA'd, I cannot share them publicly.

But now...
I'm going to try and learn how to use rust.

There is a CS engine written in rust that *hasn't* been DMCA'd, so I *can* share that publicly.
Here it is.



to slow the game down for debugging, increase the wait value in shared_game_state.rs:
(impl TimingMode {})



Notes about issues with the rust engine:

stage.change_tile() doesn't generate smoke
toroko NPC is broken (bubble thing, fixed by someone else, my build is out of date)
ironhead fastScroll jitters



Fixed:
inventory selector blinks too fast
inventory runs TSC more than once when left/right is pressed
