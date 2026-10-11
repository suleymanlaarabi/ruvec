use kurbo::Vec2;
use ruvector_document::NodeId;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CanvasInteraction {
    #[default]
    Idle,
    Panning {
        last_position: Vec2,
    },
    DraggingNode {
        node_id: NodeId,
        last_world_position: Vec2,
    },
}
