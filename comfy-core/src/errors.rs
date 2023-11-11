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

pub fn clear_error(id: impl Into<Cow<'static, str>>) {
    ERRORS.borrow_mut().data.remove(&id.into());
}

pub fn report_error(
    id: impl Into<Cow<'static, str>>,
    error: impl Into<Cow<'static, str>>,
) {
    ERRORS.borrow_mut().data.insert(id.into(), error.into());
}

// pub fn reported_errors_iter(
// ) -> impl Deref<Target = impl Iterator<Item = (&Cow<'static, str>, &Cow<'static, str>)>>
// {
//     AtomicRef::map(ERRORS.borrow(), |x| x.data.iter())
// }
