use crate::{
    CursorStyle, Tool, ToolContext, ToolOverlay, hand::HandTool, rectangle::RectangleTool,
    select::SelectTool,
};
use ruvector_input::{Key, KeyEvent};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolId {
    #[default]
    Select,
    Rectangle,
    Hand,
}
#[derive(Default)]
pub struct ToolManager {
    active: ToolId,
    temporary_hand: bool,
    select: SelectTool,
    rectangle: RectangleTool,
    hand: HandTool,
}
impl ToolManager {
    pub fn active(&self) -> ToolId {
        self.active
    }
    pub fn effective(&self) -> ToolId {
        if self.temporary_hand {
            ToolId::Hand
        } else {
            self.active
        }
    }
    fn tool(&mut self) -> &mut dyn Tool {
        match self.effective() {
            ToolId::Select => &mut self.select,
            ToolId::Rectangle => &mut self.rectangle,
            ToolId::Hand => &mut self.hand,
        }
    }
    pub fn activate(&mut self, id: ToolId, ctx: &mut ToolContext<'_>) -> bool {
        let changed = self.tool().cancel(ctx) || self.active != id;
        self.active = id;
        changed
    }
    pub fn temporary_hand(&mut self, active: bool, ctx: &mut ToolContext<'_>) -> bool {
        if self.temporary_hand == active {
            return false;
        }
        self.tool().cancel(ctx);
        self.temporary_hand = active;
        true
    }
    pub fn is_interacting(&self) -> bool {
        match self.effective() {
            ToolId::Select => self.select.is_interacting(),
            ToolId::Rectangle => self.rectangle.is_interacting(),
            ToolId::Hand => self.hand.is_interacting(),
        }
    }
    pub fn pointer_down(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        let canceled = self.tool().cancel(ctx);
        self.tool().pointer_down(ctx) || canceled
    }
    pub fn pointer_move(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.tool().pointer_move(ctx)
    }
    pub fn pointer_up(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.tool().pointer_up(ctx)
    }
    pub fn cancel(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.tool().cancel(ctx)
    }
    pub fn key_down(&mut self, event: KeyEvent, ctx: &mut ToolContext<'_>) -> bool {
        if event.key == Key::Escape {
            return self.cancel(ctx);
        }
        self.tool().key_down(event, ctx)
    }
    pub fn camera_changed(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.select.rebase(ctx.pointer_world);
        if self.effective() == ToolId::Select && self.select.is_interacting() {
            self.select.pointer_move(ctx)
        } else if self.effective() == ToolId::Rectangle {
            self.rectangle.pointer_move(ctx)
        } else {
            false
        }
    }
    pub fn overlay(&self) -> ToolOverlay {
        match self.effective() {
            ToolId::Select => self.select.overlay(),
            ToolId::Rectangle => self.rectangle.overlay(),
            ToolId::Hand => ToolOverlay::None,
        }
    }
    pub fn cursor(&self) -> CursorStyle {
        match self.effective() {
            ToolId::Select => self.select.cursor(),
            ToolId::Rectangle => self.rectangle.cursor(),
            ToolId::Hand => self.hand.cursor(),
        }
    }
}
