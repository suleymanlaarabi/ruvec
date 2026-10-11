use gpui::{Modifiers as GpuiModifiers, MouseButton, Pixels, Point};
use ruvector_canvas::Vec2;
use ruvector_input::{Modifiers, PointerButton, PointerEvent};

pub(crate) fn logical_position(position: Point<Pixels>) -> Vec2 {
    Vec2::new(
        f64::from(f32::from(position.x)),
        f64::from(f32::from(position.y)),
    )
}
pub(crate) fn modifiers(value: GpuiModifiers) -> Modifiers {
    Modifiers {
        shift: value.shift,
        alt: value.alt,
        control: value.control,
        command: value.platform,
    }
}
pub(crate) fn pointer(
    position: Point<Pixels>,
    origin: Point<Pixels>,
    button: Option<MouseButton>,
    value: GpuiModifiers,
) -> PointerEvent {
    PointerEvent {
        position: logical_position(position - origin),
        button: button.map(|b| match b {
            MouseButton::Left => PointerButton::Left,
            MouseButton::Middle => PointerButton::Middle,
            _ => PointerButton::Right,
        }),
        modifiers: modifiers(value),
    }
}
