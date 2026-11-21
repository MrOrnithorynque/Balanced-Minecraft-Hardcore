// The item physics code has no Game. Its caller supplies the filter for this level's tick.
use std::cell::Cell;

thread_local! {
    static FILTER: Cell<Option<fn(&str) -> bool>> = const { Cell::new(None) };
}

pub fn set_filter(filter: Option<fn(&str) -> bool>) {
    FILTER.set(filter);
}

pub fn protected(item: &str) -> bool {
    FILTER.get().is_some_and(|matches| matches(item))
}
