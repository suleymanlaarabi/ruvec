use crate::Command;
use ruvector_document::{Document, Node, NodeKind, RectangleNode, Selection};

pub struct DuplicateNodes {
    source: Vec<RectangleNode>,
    created: Vec<(usize, Node)>,
}
impl DuplicateNodes {
    pub fn new(document: &Document, selection: &Selection) -> Self {
        Self {
            source: document
                .nodes()
                .filter(|node| selection.contains(node.id))
                .map(|node| {
                    let NodeKind::Rectangle(rect) = &node.kind;
                    *rect
                })
                .collect(),
            created: Vec::new(),
        }
    }
}
impl Command for DuplicateNodes {
    fn apply(&mut self, document: &mut Document) -> bool {
        if self.created.is_empty() {
            for rect in &self.source {
                let index = document.nodes().len();
                let id =
                    document.create_rectangle(rect.x + 20., rect.y + 20., rect.width, rect.height);
                self.created
                    .push((index, document.node(id).unwrap().clone()));
            }
        } else {
            for (index, node) in &self.created {
                document.insert_node(*index, node.clone());
            }
        }
        !self.created.is_empty()
    }
    fn undo(&mut self, document: &mut Document) {
        for (_, node) in self.created.iter().rev() {
            document.remove_node(node.id);
        }
    }
    fn update_selection(&self, selection: &mut Selection) {
        selection.clear();
        for (_, node) in &self.created {
            selection.add(node.id);
        }
    }
}
