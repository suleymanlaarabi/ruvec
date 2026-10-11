use crate::Modifiers;
use kurbo::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct PointerEvent {
    pub position: Vec2,
    pub button: Option<PointerButton>,
    pub modifiers: Modifiers,
}
