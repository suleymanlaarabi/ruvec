use crate::Vec2;
use ruvector_document::{Document, NodeId, NodeKind};

pub fn hit_test(document: &Document, world_position: Vec2) -> Option<NodeId> {
    if !world_position.is_finite() {
        return None;
    }
    document.nodes().rev().find_map(|node| {
        let NodeKind::Rectangle(rect) = &node.kind;
        (world_position.x >= rect.x
            && world_position.x <= rect.x + rect.width
            && world_position.y >= rect.y
            && world_position.y <= rect.y + rect.height)
            .then_some(node.id)
    })
}
