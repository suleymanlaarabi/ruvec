#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(crate) u64);

#[derive(Debug, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
}

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Rectangle(RectangleNode),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectangleNode {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
