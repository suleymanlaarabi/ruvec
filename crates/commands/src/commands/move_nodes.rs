use crate::Command;
use ruvector_document::{Document, NodeId, NodeKind};

pub struct NodePosition {
    pub id: NodeId,
    pub from: (f64, f64),
    pub to: (f64, f64),
}
pub struct MoveNodes {
    pub(crate) positions: Vec<NodePosition>,
}
impl MoveNodes {
    pub fn new(positions: Vec<NodePosition>) -> Self {
        Self { positions }
    }
    pub(crate) fn write(document: &mut Document, id: NodeId, position: (f64, f64)) {
        let Some(node) = document.node(id) else {
            return;
        };
        let NodeKind::Rectangle(rect) = &node.kind;
        if (rect.x, rect.y) == position {
            return;
        }
        let NodeKind::Rectangle(rect) = &mut document.node_mut(id).unwrap().kind;
        rect.x = position.0;
        rect.y = position.1;
    }
}
impl Command for MoveNodes {
    fn apply(&mut self, document: &mut Document) -> bool {
        let mut changed = false;
        for position in &self.positions {
            if position.from != position.to {
                changed = true;
            }
            Self::write(document, position.id, position.to);
        }
        changed
    }
    fn undo(&mut self, document: &mut Document) {
        for position in &self.positions {
            Self::write(document, position.id, position.from);
        }
    }
}
