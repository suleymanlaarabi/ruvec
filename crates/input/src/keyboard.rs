use crate::Modifiers;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Escape,
    Space,
    ModifiersChanged,
}
#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}
