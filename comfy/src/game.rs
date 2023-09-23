use crate::*;

// pub type StateBuilder<T> = fn(&mut EngineContext) -> T;

pub struct SimpleGame<T> {
    pub engine: EngineState,
    pub state_builder: StateBuilder<T>,
    pub state: Option<T>,
    pub setup: fn(&mut T, &mut EngineContext),
    pub update: fn(&mut T, &mut EngineContext),
}

impl<T> SimpleGame<T> {
    pub fn new(
        engine: EngineState,
        state_builder: StateBuilder<T>,
        setup: fn(&mut T, &mut EngineContext),
        update: fn(&mut T, &mut EngineContext),
    ) -> Self {
        Self { state_builder, state: None, engine, setup, update }
    }

    pub fn update(&mut self, c: &mut EngineContext) {
        if self.state.is_none() {
            let mut state = (self.state_builder)(c);
            (self.setup)(&mut state, c);

            self.state = Some(state);
        }

        if let Some(state) = self.state.as_mut() {
            (self.update)(state, c);
        }
    }
}

pub type StateBuilder<T> = fn(&mut EngineContext) -> T;

pub struct ComfyGame<S, C> {
    pub engine: EngineState,
    pub state_builder: StateBuilder<S>,
    pub state: Option<S>,
    pub setup: fn(&mut S, &mut EngineContext),
    pub update: fn(&mut C),
    pub make_context: fn(&mut S, &mut EngineContext) -> C,
}

impl<S, C> ComfyGame<S, C> {
    pub fn new(
        engine: EngineState,
        state_builder: StateBuilder<S>,
        setup: fn(&mut S, &mut EngineContext),
        update: fn(&mut C),
        make_context: fn(&mut S, &mut EngineContext) -> C,
    ) -> Self {
        Self { state_builder, state: None, engine, setup, update, make_context }
    }

    pub fn update(&mut self, c: &mut EngineContext) {
        if self.state.is_none() {
            let mut state = (self.state_builder)(c);
            (self.setup)(&mut state, c);

            self.state = Some(state);
        }

        if let Some(state) = self.state.as_mut() {
            let mut game_c = (self.make_context)(state, c);
            (self.update)(&mut game_c);
        }
    }
}
