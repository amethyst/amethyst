//! The most basic Amethyst example.

// transitions! {
//     Intro => MainMenu;
//     MainMenu => OptionsMenu, Game;
//     Game => Paused;
//     Paused => OptionsMenu, Game, MainMenu;
// }

macro_rules! transitions {
    { $( $start:ident => $($target:ident),+ );+ ; } => {
        $(
            enum $start {
                $($target),+
            }
        )+
    }
}

trait State {
    fn new() -> Self where Self: Sized;
    fn on_start(&mut self) {}
    fn on_stop(&mut self) {}
    fn on_leave(&mut self) {}
    fn on_return(&mut self) {}
    fn handle_events(&mut self, _events: &Vec<i32>) {}
    fn fixed_update(&mut self, _delta: f32) -> Action { Action::Nothing }
    fn update(&mut self, _delta: f32) -> Action { Action::Pop }
}

enum Action {
    Nothing,
    Pop,
    Push(Box<State>),
}

struct PushdownAutomaton {
    running: bool,
    states: Vec<Box<State>>,
}

impl PushdownAutomaton {
    pub fn new<T: 'static>(initial_state: T) -> PushdownAutomaton
        where T: State
    {
        PushdownAutomaton {
            running: false,
            states: vec![Box::new(initial_state)],
        }
    }

    pub fn start(&mut self) {
        if self.is_running() {
            panic!("Error: Automaton started more than once.");
        }

        self.states.last_mut().unwrap().on_start();
        self.running = true;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update(&mut self, delta: f32) {
        let action = self.states.last_mut().unwrap().update(delta);
        self.check_action(action);
    }

    fn push(&mut self, state: Box<State>) {
        if let Some(current) = self.states.last_mut() {
            current.on_leave();
        }

        self.states.push(state);
        self.states.last_mut().unwrap().on_start();
    }

    pub fn pop(&mut self) {
        self.states.pop().unwrap().on_stop();

        if let Some(next) = self.states.last_mut() {
            next.on_return();
        } else {
            self.running = false;
        }
    }

    fn check_action(&mut self, action: Action) {
        match action {
            Action::Nothing => (),
            Action::Pop => self.pop(),
            Action::Push(state) => self.push(state),
        }
    }
}

struct Intro;
impl State for Intro {
    fn new() -> Intro {
        Intro
    }

    fn update(&mut self, _delta: f32) -> Action {
        Action::Pop
    }
}

fn main() { 
    let mut game = PushdownAutomaton::new(Intro::new());
    game.start();

    while game.is_running() {
        game.update(0.0);
    }
}
