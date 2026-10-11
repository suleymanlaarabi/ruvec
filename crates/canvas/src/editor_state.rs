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
