use crate::*;

pub static ERRORS: Lazy<AtomicRefCell<Errors>> =
    Lazy::new(|| AtomicRefCell::new(Errors::new()));

pub struct Errors {
    pub data: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl Errors {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }
}

/// Stores an error message with the given ID. This is useful for immediate mode error reporting
/// when you don't want to pollute the log on every frame.
///
/// When `--features dev` is active, errors will show in an egui window in game.
///
/// The `id` parameter can be any string. We're using `Cow<'static, str>` to save on allocations
/// for ids/errors that can be represented as `&'static str`.
pub fn report_error(
    id: impl Into<Cow<'static, str>>,
    error: impl Into<Cow<'static, str>>,
) {
    ERRORS.borrow_mut().data.insert(id.into(), error.into());
}

/// Clears a previously set error. Use the same ID as when calling `report_error`.
pub fn clear_error(id: impl Into<Cow<'static, str>>) {
    ERRORS.borrow_mut().data.remove(&id.into());
}
