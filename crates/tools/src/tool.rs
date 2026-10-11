use kurbo::{Rect, Vec2};
use ruvector_canvas::CanvasState;
use ruvector_commands::History;
use ruvector_document::{Document, Selection};
use ruvector_input::{KeyEvent, Modifiers};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ToolOverlay {
    #[default]
    None,
    RectanglePreview {
        rect: Rect,
    },
    SelectionMarquee {
        rect: Rect,
    },
}
#[derive(Clone, Copy, Debug)]
pub enum CursorStyle {
    Arrow,
    Crosshair,
    Grab,
    Grabbing,
}

pub struct ToolContext<'a> {
    pub document: &'a mut Document,
    pub selection: &'a mut Selection,
    pub history: &'a mut History,
    pub canvas: &'a mut CanvasState,
    pub pointer_screen: Vec2,
    pub pointer_world: Vec2,
    pub modifiers: Modifiers,
}

pub trait Tool {
    fn pointer_down(&mut self, ctx: &mut ToolContext<'_>) -> bool;
    fn pointer_move(&mut self, ctx: &mut ToolContext<'_>) -> bool;
    fn pointer_up(&mut self, ctx: &mut ToolContext<'_>) -> bool;
    fn cancel(&mut self, ctx: &mut ToolContext<'_>) -> bool;
    fn key_down(&mut self, _event: KeyEvent, _ctx: &mut ToolContext<'_>) -> bool {
        false
    }
    fn cursor(&self) -> CursorStyle;
}
