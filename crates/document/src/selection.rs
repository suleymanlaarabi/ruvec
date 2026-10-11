use crate::NodeId;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    selected: Option<NodeId>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.selected = None;
    }
    pub fn set(&mut self, id: NodeId) {
        self.selected = Some(id);
    }
    pub fn selected(&self) -> Option<NodeId> {
        self.selected
    }
    pub fn contains(&self, id: NodeId) -> bool {
        self.selected == Some(id)
    }
}
