use crate::*;

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
