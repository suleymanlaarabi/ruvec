use crate::{Command, MoveNodes, NodePosition};
use ruvector_document::{Document, NodeKind, Selection};

pub struct Transaction {
    command: MoveNodes,
}
impl Transaction {
    pub fn begin(document: &Document, selection: &Selection) -> Self {
        let positions = document
            .nodes()
            .filter(|node| selection.contains(node.id))
            .map(|node| {
                let NodeKind::Rectangle(rect) = &node.kind;
                NodePosition {
                    id: node.id,
                    from: (rect.x, rect.y),
                    to: (rect.x, rect.y),
                }
            })
            .collect();
        Self {
            command: MoveNodes::new(positions),
        }
    }
    pub fn preview(&mut self, document: &mut Document, dx: f64, dy: f64) -> bool {
        if !dx.is_finite() || !dy.is_finite() || (dx == 0. && dy == 0.) {
            return false;
        }
        if self
            .command
            .positions
            .iter()
            .any(|p| !(p.to.0 + dx).is_finite() || !(p.to.1 + dy).is_finite())
        {
            return false;
        }
        for p in &mut self.command.positions {
            p.to = (p.to.0 + dx, p.to.1 + dy);
            MoveNodes::write(document, p.id, p.to);
        }
        !self.command.positions.is_empty()
    }
    pub fn commit(self) -> Option<MoveNodes> {
        self.command
            .positions
            .iter()
            .any(|p| p.from != p.to)
            .then_some(self.command)
    }
    pub fn cancel(mut self, document: &mut Document) {
        self.command.undo(document);
    }
}
