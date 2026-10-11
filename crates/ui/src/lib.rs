mod actions;
mod canvas_element;
mod input_adapter;
mod toolbar;

use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, rgb};
use ruvector_tools::ToolId;

pub fn init(cx: &mut App) {
    actions::init(cx);
}

pub struct EditorView {
    canvas: Entity<canvas_element::CanvasView>,
    active_tool: ToolId,
    _subscription: Subscription,
}
impl EditorView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let canvas = cx.new(|cx| canvas_element::CanvasView::new(window, cx));
        let subscription = cx.observe(&canvas, |this, canvas, cx| {
            let active = canvas.read(cx).active_tool();
            if this.active_tool != active {
                this.active_tool = active;
                cx.notify();
            }
        });
        Self {
            canvas,
            active_tool: ToolId::Select,
            _subscription: subscription,
        }
    }
}
impl Render for EditorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("editor")
            .size_full()
            .flex()
            .bg(rgb(0x181b22))
            .child(toolbar::toolbar(
                self.active_tool,
                self.canvas.read(cx).focus_handle(),
            ))
            .child(self.canvas.clone())
    }
}
