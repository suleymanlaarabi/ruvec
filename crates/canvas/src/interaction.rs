use kurbo::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CanvasInteraction {
    #[default]
    Idle,
    Panning {
        last_position: Vec2,
    },
}
