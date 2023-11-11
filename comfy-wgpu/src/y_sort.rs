use crate::*;

static Y_SORT_FLAGS: Lazy<AtomicRefCell<HashMap<i32, bool>>> =
    Lazy::new(|| AtomicRefCell::new(HashMap::default()));

pub fn set_y_sort(z_index: i32, value: bool) {
    Y_SORT_FLAGS.borrow_mut().insert(z_index, value);
}

pub fn get_y_sort(z_index: i32) -> bool {
    *Y_SORT_FLAGS.borrow().get(&z_index).unwrap_or(&false)
}
