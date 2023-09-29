use crate::*;

// TODO: this should eventually turn into a comfy-hecs crate
pub static WORLD: Lazy<Arc<AtomicRefCell<World>>> =
    Lazy::new(|| Arc::new(AtomicRefCell::new(World::new())));

pub static COMMANDS: Lazy<Arc<AtomicRefCell<CommandBuffer>>> =
    Lazy::new(|| Arc::new(AtomicRefCell::new(CommandBuffer::new())));

pub fn world() -> AtomicRef<'static, World> {
    WORLD.borrow()
}

pub fn world_mut() -> AtomicRefMut<'static, World> {
    WORLD.borrow_mut()
}

pub fn commands() -> AtomicRefMut<'static, CommandBuffer> {
    COMMANDS.borrow_mut()
}

pub fn reset_world() {
    *world_mut() = World::new();
}
