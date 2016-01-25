// Tests for amethyst_engine::state

extern crate amethyst_engine;

use amethyst_engine::{StateMachine, State, Trans, Duration};

struct State1(u8);
struct State2;

impl State for State1 {
    fn update(&mut self, _delta: Duration) -> Trans {
        if self.0 > 0 {
            self.0 -= 1;
            Trans::None
        } else {
            Trans::Switch(Box::new(State2))
        }
    }
}

impl State for State2 {
    fn update(&mut self, _delta: Duration) -> Trans {
        Trans::Pop
    }
}

#[test]
fn statemachine_switch_pop() {
    let mut sm = StateMachine::new(State1(7));
    sm.start();
    for _ in 0..8 {
        sm.update(Duration::seconds(0));
        assert!(sm.is_running());
    }
    sm.update(Duration::seconds(0));
    assert!(!sm.is_running());
}
