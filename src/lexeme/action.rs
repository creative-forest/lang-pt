use super::Action;

impl<T> Default for Action<T> {
    fn default() -> Self {
        Self::None { discard: false }
    }
}

impl<T> Action<T> {
    pub fn remove(discard: bool) -> Self {
        Self::Pop { discard }
    }
    pub fn append(state: T, discard: bool) -> Self {
        Self::Append { state, discard }
    }
    pub fn switch(state: T, discard: bool) -> Self {
        Self::Switch { state, discard }
    }
}
