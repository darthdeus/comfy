use comfy::*;
use gilrs::Gilrs;

comfy_game!("Controller Demo", ControllerDemo);

pub struct ControllerDemo {
    gilrs: Option<Gilrs>,
}

impl GameLoop for ControllerDemo {
    fn new(_c: &mut EngineState) -> Self {
        Self { gilrs: Gilrs::new().ok() }
    }

    fn update(&mut self, _c: &mut EngineContext) {
        clear_background(BLACK);
        println!("is there gilrs? {}", self.gilrs.is_some());
    }
}
