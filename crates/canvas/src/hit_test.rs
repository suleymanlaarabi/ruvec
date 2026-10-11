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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn topmost_rectangle_and_empty_space() {
        let mut doc = Document::new();
        let a = doc.create_rectangle(0., 0., 100., 100.);
        let b = doc.create_rectangle(50., 50., 100., 100.);
        assert_eq!(hit_test(&doc, Vec2::new(75., 75.)), Some(b));
        assert_eq!(hit_test(&doc, Vec2::ZERO), Some(a));
        assert_eq!(hit_test(&doc, Vec2::new(200., 200.)), None);
        assert_eq!(hit_test(&doc, Vec2::new(f64::NAN, 0.)), None);
    }
}
