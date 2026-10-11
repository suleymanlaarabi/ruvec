mod canvas_element;

use gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*, rgb};

pub struct EditorView {
    canvas: Entity<canvas_element::CanvasView>,
}

impl EditorView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            canvas: cx.new(|cx| canvas_element::CanvasView::new(window, cx)),
        }
    }
}

impl Render for EditorView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("editor")
            .size_full()
            .flex()
            .bg(rgb(0x181b22))
            .child(self.canvas.clone())
    }
}
