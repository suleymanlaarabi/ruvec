use crate::NodeId;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Selection {
    selected: Vec<NodeId>,
    revision: u64,
}

impl Selection {
    pub fn clear(&mut self) {
        if !self.selected.is_empty() {
            self.selected.clear();
            self.revision += 1;
        }
    }
    pub fn set(&mut self, id: NodeId) {
        if self.selected.as_slice() != [id] {
            self.clear();
            self.add(id);
        }
    }
    pub fn add(&mut self, id: NodeId) {
        if !self.contains(id) {
            self.selected.push(id);
            self.revision += 1;
        }
    }
    pub fn remove(&mut self, id: NodeId) {
        if let Some(index) = self.selected.iter().position(|selected| *selected == id) {
            self.selected.remove(index);
            self.revision += 1;
        }
    }
    pub fn toggle(&mut self, id: NodeId) {
        if self.contains(id) {
            self.remove(id);
        } else {
            self.add(id);
        }
    }
    pub fn contains(&self, id: NodeId) -> bool {
        self.selected.contains(&id)
    }
    pub fn iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.selected.iter().copied()
    }
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn reserve(&mut self, additional: usize) {
        self.selected.reserve(additional);
    }
    pub fn replace(&mut self, other: &Selection) {
        if self.selected != other.selected {
            self.selected.clone_from(&other.selected);
            self.revision += 1;
        }
    }
}
