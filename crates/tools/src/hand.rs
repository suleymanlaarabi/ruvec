use crate::{CursorStyle, Tool, ToolContext};
#[derive(Default)]
pub(crate) struct HandTool {
    dragging: bool,
}
impl HandTool {
    pub fn is_interacting(&self) -> bool {
        self.dragging
    }
}
impl Tool for HandTool {
    fn pointer_down(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.dragging = true;
        ctx.canvas.begin_pan(ctx.pointer_screen);
        true
    }
    fn pointer_move(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        if self.dragging {
            ctx.canvas.pan_to(ctx.pointer_screen)
        } else {
            false
        }
    }
    fn pointer_up(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        let changed = self.pointer_move(ctx);
        self.cancel(ctx) || changed
    }
    fn cancel(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        let was = self.dragging;
        self.dragging = false;
        ctx.canvas.end_pan();
        was
    }
    fn cursor(&self) -> CursorStyle {
        if self.dragging {
            CursorStyle::Grabbing
        } else {
            CursorStyle::Grab
        }
    }
}
