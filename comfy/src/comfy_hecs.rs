use crate::*;

pub static WORLD: Lazy<AtomicRefCell<World>> =
    Lazy::new(|| AtomicRefCell::new(World::new()));

pub static COMMANDS: Lazy<AtomicRefCell<CommandBuffer>> =
    Lazy::new(|| AtomicRefCell::new(CommandBuffer::new()));

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
