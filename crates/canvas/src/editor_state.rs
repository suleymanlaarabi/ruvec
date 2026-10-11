use crate::{CanvasInteraction, CanvasState, Vec2, hit_test};
use ruvector_document::{Document, NodeKind, Selection};

#[derive(Debug, Default)]
pub struct EditorState {
    pub document: Document,
    pub selection: Selection,
}

impl EditorState {
    pub fn seeded() -> Self {
        let mut state = Self::default();
        state.document.create_rectangle(0., 0., 300., 200.);
        state.document.create_rectangle(210., 130., 260., 180.);
        state.document.create_rectangle(530., 45., 180., 260.);
        state
    }

    pub fn pointer_down(&mut self, canvas: &mut CanvasState, position: Vec2, space: bool) -> bool {
        if !position.is_finite() {
            return false;
        }
        if space {
            canvas.begin_pan(position);
            return false;
        }
        let before = self.selection;
        let world = canvas.camera.screen_to_world(position);
        if let Some(id) = hit_test(&self.document, world) {
            self.selection.set(id);
            canvas.interaction = CanvasInteraction::DraggingNode {
                node_id: id,
                last_world_position: world,
            };
        } else {
            self.selection.clear();
            canvas.end_pan();
        }
        before != self.selection
    }

    pub fn drag_to(&mut self, canvas: &mut CanvasState, position: Vec2) -> bool {
        if matches!(canvas.interaction, CanvasInteraction::Panning { .. }) {
            return canvas.pan_to(position);
        }
        let CanvasInteraction::DraggingNode {
            node_id,
            last_world_position,
        } = canvas.interaction
        else {
            return false;
        };
        if !position.is_finite() {
            return false;
        }
        let world = canvas.camera.screen_to_world(position);
        let delta = world - last_world_position;
        if !delta.is_finite() || delta == Vec2::ZERO {
            return false;
        }
        let Some(node) = self.document.node(node_id) else {
            canvas.end_pan();
            return false;
        };
        let NodeKind::Rectangle(rect) = &node.kind;
        let next = Vec2::new(rect.x, rect.y) + delta;
        if !next.is_finite() {
            return false;
        }
        let NodeKind::Rectangle(rect) = &mut self.document.node_mut(node_id).unwrap().kind;
        rect.x = next.x;
        rect.y = next.y;
        canvas.interaction = CanvasInteraction::DraggingNode {
            node_id,
            last_world_position: world,
        };
        true
    }

    pub fn rebase_drag(&self, canvas: &mut CanvasState, position: Vec2) {
        if let CanvasInteraction::DraggingNode { node_id, .. } = canvas.interaction {
            canvas.interaction = CanvasInteraction::DraggingNode {
                node_id,
                last_world_position: canvas.camera.screen_to_world(position),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Camera2D;
    #[test]
    fn zoom_during_drag_does_not_jump_on_next_pointer_move() {
        let mut editor = EditorState::seeded();
        let mut canvas = CanvasState::default();
        let cursor = Vec2::new(100., 100.);
        editor.pointer_down(&mut canvas, cursor, false);
        let id = editor.selection.selected().unwrap();
        canvas.camera.zoom_at(Vec2::new(500., 400.), 2.);
        editor.rebase_drag(&mut canvas, cursor);
        let revision = editor.document.revision();
        assert!(!editor.drag_to(&mut canvas, cursor));
        assert_eq!(editor.document.revision(), revision);
        assert!(editor.drag_to(&mut canvas, cursor + Vec2::new(20., 10.)));
        let NodeKind::Rectangle(rect) = &editor.document.node(id).unwrap().kind;
        assert_eq!((rect.x, rect.y), (10., 5.));
    }

    #[test]
    fn selection_drag_at_every_zoom_pan_priority_and_deselection() {
        for zoom in [0.01, 0.5, 1., 4., 256.] {
            let mut editor = EditorState::seeded();
            let mut canvas = CanvasState {
                camera: Camera2D {
                    translation: Vec2::new(40., 50.),
                    zoom,
                },
                ..Default::default()
            };
            let point = canvas.camera.world_to_screen(Vec2::new(20., 30.));
            assert!(editor.pointer_down(&mut canvas, point, false));
            let id = editor.selection.selected().unwrap();
            let revision = editor.document.revision();
            assert!(!editor.drag_to(&mut canvas, point));
            assert_eq!(editor.document.revision(), revision);
            assert!(editor.drag_to(&mut canvas, point + Vec2::new(10., -5.) * zoom));
            let NodeKind::Rectangle(rect) = &editor.document.node(id).unwrap().kind;
            assert!((rect.x - 10.).abs() < 1e-8 && (rect.y + 5.).abs() < 1e-8);
            let moved_revision = editor.document.revision();
            canvas.end_pan();
            assert!(!editor.drag_to(&mut canvas, point));
            assert!(!editor.pointer_down(&mut canvas, point, true));
            assert_eq!(editor.selection.selected(), Some(id));
            assert!(editor.drag_to(&mut canvas, point + Vec2::new(5., 7.)));
            assert_eq!(editor.document.revision(), moved_revision);
            let empty = canvas.camera.world_to_screen(Vec2::new(-100., -100.));
            assert!(editor.pointer_down(&mut canvas, empty, false));
            assert_eq!(editor.selection.selected(), None);
        }
    }
}
