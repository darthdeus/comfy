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

// pub type ContextBuilder<'a, 'b: 'a, S, C> =
//     fn(&'a mut S, &'b mut EngineContext<'b>) -> C;

// pub type ContextBuilder<S, C> = fn(&mut S, &mut EngineState) -> C;

pub type StateBuilder<T> = fn(&mut EngineContext) -> T;

// TODO: ... once someone smart figures out how to pass in `context_builder` things can get a bit
// nicer.

// pub struct ComfyGame<S, C> {
//     pub engine: EngineState,
//     pub state_builder: StateBuilder<S>,
//     pub state: Option<S>,
//     pub setup: fn(&mut S, &mut EngineContext),
//     pub update: fn(&mut C),
// }
//
// impl<S: 'static, C: 'static> ComfyGame<S, C> {
//     pub fn new(
//         engine: EngineState,
//         state_builder: StateBuilder<S>,
//         setup: fn(&mut S, &mut EngineContext),
//         update: fn(&mut C),
//     ) -> Self {
//         Self { state_builder, state: None, engine, setup, update }
//     }
//
//     pub fn update(&mut self, context_builder: ???) {
//         let mut c = self.engine.make_context();
//
//         if self.state.is_none() {
//             let mut state = (self.state_builder)(&mut c);
//             (self.setup)(&mut state, &mut c);
//
//             self.state = Some(state);
//         }
//
//         if let Some(state) = self.state.as_mut() {
//             run_early_update_stages(&mut c);
//             // TODO: early update
//             run_mid_update_stages(&mut c);
//
//             let mut game_c =
//                 (context_builder)(state, &mut self.engine);
//
//             (self.update)(&mut game_c);
//
//             run_late_update_stages(&mut c);
//         }
//     }
// }
