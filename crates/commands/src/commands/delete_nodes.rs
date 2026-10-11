use crate::Command;
use ruvector_document::{Document, Node, NodeId, Selection};

pub struct DeleteNodes {
    ids: Vec<NodeId>,
    removed: Vec<(usize, Node)>,
}
impl DeleteNodes {
    pub fn new(selection: &Selection) -> Self {
        Self {
            ids: selection.iter().collect(),
            removed: Vec::new(),
        }
    }
}
impl Command for DeleteNodes {
    fn apply(&mut self, document: &mut Document) -> bool {
        if self.removed.is_empty() {
            self.removed = document
                .nodes()
                .enumerate()
                .filter(|(_, node)| self.ids.contains(&node.id))
                .map(|(index, node)| (index, node.clone()))
                .collect();
        }
        for (_, node) in self.removed.iter().rev() {
            document.remove_node(node.id);
        }
        !self.removed.is_empty()
    }
    fn undo(&mut self, document: &mut Document) {
        for (index, node) in &self.removed {
            document.insert_node(*index, node.clone());
        }
    }
    fn update_selection(&self, selection: &mut Selection) {
        selection.clear();
    }
}
