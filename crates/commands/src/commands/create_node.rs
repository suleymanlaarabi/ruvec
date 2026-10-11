use crate::Command;
use ruvector_document::{Document, Node, RectangleNode, Selection};

pub struct CreateNode {
    rect: RectangleNode,
    created: Option<(usize, Node)>,
}
impl CreateNode {
    pub fn new(rect: RectangleNode) -> Self {
        Self {
            rect,
            created: None,
        }
    }
}
impl Command for CreateNode {
    fn apply(&mut self, document: &mut Document) -> bool {
        if let Some((index, node)) = &self.created {
            document.insert_node(*index, node.clone());
        } else {
            let index = document.nodes().len();
            let r = self.rect;
            let id = document.create_rectangle(r.x, r.y, r.width, r.height);
            self.created = Some((index, document.node(id).unwrap().clone()));
        }
        true
    }
    fn undo(&mut self, document: &mut Document) {
        if let Some((_, node)) = &self.created {
            document.remove_node(node.id);
        }
    }
    fn update_selection(&self, selection: &mut Selection) {
        if let Some((_, node)) = &self.created {
            selection.set(node.id);
        }
    }
}
