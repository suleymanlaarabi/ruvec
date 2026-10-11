use crate::{Node, NodeId, NodeKind, RectangleNode};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct Document {
    next_id: u64,
    nodes: Vec<Node>,
    revision: u64,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            nodes: Vec::new(),
            revision: NEXT_REVISION.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn create_rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id = self.next_id.checked_add(1).expect("node IDs exhausted");
        self.nodes.push(Node {
            id,
            kind: NodeKind::Rectangle(RectangleNode {
                x,
                y,
                width,
                height,
            }),
        });
        self.changed();
        id
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        let index = self.nodes.iter().position(|node| node.id == id)?;
        // Mutable access invalidates cached rendering, including replacement of a document.
        self.changed();
        Some(&mut self.nodes[index])
    }

    pub fn nodes(&self) -> impl DoubleEndedIterator<Item = &Node> + ExactSizeIterator {
        self.nodes.iter()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn changed(&mut self) {
        self.revision = NEXT_REVISION.fetch_add(1, Ordering::Relaxed);
    }
}
