use crate::{Camera2D, CanvasInteraction, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasState {
    pub camera: Camera2D,
    pub interaction: CanvasInteraction,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            camera: Camera2D {
                translation: Vec2::new(48.0, 48.0),
                ..Camera2D::default()
            },
            interaction: CanvasInteraction::Idle,
        }
    }
}

impl CanvasState {
    pub fn begin_pan(&mut self, position: Vec2) {
        if position.is_finite() {
            self.interaction = CanvasInteraction::Panning {
                last_position: position,
            };
        }
    }

    pub fn pan_to(&mut self, position: Vec2) -> bool {
        let CanvasInteraction::Panning { last_position } = self.interaction else {
            return false;
        };
        if !position.is_finite() {
            return false;
        }
        let before = self.camera;
        self.camera.pan(position - last_position);
        self.interaction = CanvasInteraction::Panning {
            last_position: position,
        };
        self.camera != before
    }

    pub fn end_pan(&mut self) {
        self.interaction = CanvasInteraction::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_uses_logical_deltas_and_stops_on_release() {
        let mut state = CanvasState::default();
        state.camera.zoom = 4.0;
        assert!(!state.pan_to(Vec2::new(500.0, 500.0)));
        state.begin_pan(Vec2::new(100.0, 200.0));
        assert!(!state.pan_to(Vec2::new(100.0, 200.0)));
        assert!(state.pan_to(Vec2::new(124.5, 188.0)));
        assert!(state.pan_to(Vec2::new(131.0, 190.0)));
        assert_eq!(state.camera.translation, Vec2::new(79.0, 38.0));
        assert_eq!(state.camera.zoom, 4.0);
        state.end_pan();
        let released = state;
        assert!(!state.pan_to(Vec2::ZERO));
        assert_eq!(state, released);
    }
}
