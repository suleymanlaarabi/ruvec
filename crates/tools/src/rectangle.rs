use crate::{CursorStyle, Tool, ToolContext, ToolOverlay};
use kurbo::{Rect, Vec2};
use ruvector_commands::CreateNode;
use ruvector_document::RectangleNode;
use ruvector_input::KeyEvent;

#[derive(Default)]
pub(crate) struct RectangleTool {
    start: Option<Vec2>,
    preview: Option<Rect>,
}
impl RectangleTool {
    pub fn is_interacting(&self) -> bool {
        self.start.is_some()
    }
    pub fn overlay(&self) -> ToolOverlay {
        self.preview
            .map_or(ToolOverlay::None, |rect| ToolOverlay::RectanglePreview {
                rect,
            })
    }
    fn update(&mut self, ctx: &ToolContext<'_>) -> bool {
        let Some(start) = self.start else {
            return false;
        };
        let mut delta = ctx.pointer_world - start;
        if ctx.modifiers.shift {
            let size = delta.x.abs().max(delta.y.abs());
            delta = Vec2::new(size.copysign(delta.x), size.copysign(delta.y));
        }
        let a = if ctx.modifiers.alt {
            start - delta
        } else {
            start
        };
        let b = start + delta;
        let rect = Rect::new(a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y));
        if ![rect.x0, rect.y0, rect.x1, rect.y1]
            .into_iter()
            .all(f64::is_finite)
        {
            return false;
        }
        let changed = self.preview != Some(rect);
        self.preview = Some(rect);
        changed
    }
}
impl Tool for RectangleTool {
    fn pointer_down(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.start = Some(ctx.pointer_world);
        self.update(ctx)
    }
    fn pointer_move(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.update(ctx)
    }
    fn pointer_up(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.update(ctx);
        let Some(rect) = self.preview.take() else {
            self.start = None;
            return false;
        };
        self.start = None;
        if rect.width() * ctx.canvas.camera.zoom >= 2.
            && rect.height() * ctx.canvas.camera.zoom >= 2.
        {
            ctx.history.execute(
                Box::new(CreateNode::new(RectangleNode {
                    x: rect.x0,
                    y: rect.y0,
                    width: rect.width(),
                    height: rect.height(),
                })),
                ctx.document,
                ctx.selection,
            );
        }
        true
    }
    fn cancel(&mut self, _ctx: &mut ToolContext<'_>) -> bool {
        self.start = None;
        self.preview.take().is_some()
    }
    fn key_down(&mut self, _event: KeyEvent, ctx: &mut ToolContext<'_>) -> bool {
        self.update(ctx)
    }
    fn cursor(&self) -> CursorStyle {
        CursorStyle::Crosshair
    }
}
