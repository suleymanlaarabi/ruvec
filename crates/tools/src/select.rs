use crate::{CursorStyle, Tool, ToolContext, ToolOverlay};
use kurbo::{Rect, Vec2};
use ruvector_canvas::hit_test;
use ruvector_commands::Transaction;
use ruvector_document::{NodeId, NodeKind, Selection};

enum SelectState {
    Idle,
    Dragging {
        last_world: Vec2,
        transaction: Transaction,
        collapse_on_click: Option<NodeId>,
    },
    BoxSelecting {
        start: Vec2,
        current: Vec2,
        before: Selection,
        additive: bool,
    },
}
impl Default for SelectState {
    fn default() -> Self {
        Self::Idle
    }
}
#[derive(Default)]
pub(crate) struct SelectTool {
    state: SelectState,
}
fn bounds(a: Vec2, b: Vec2) -> Rect {
    Rect::new(a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y))
}
impl SelectTool {
    pub fn is_interacting(&self) -> bool {
        !matches!(self.state, SelectState::Idle)
    }
    pub fn overlay(&self) -> ToolOverlay {
        match &self.state {
            SelectState::BoxSelecting { start, current, .. } => ToolOverlay::SelectionMarquee {
                rect: bounds(*start, *current),
            },
            _ => ToolOverlay::None,
        }
    }
    pub fn rebase(&mut self, world: Vec2) {
        if let SelectState::Dragging { last_world, .. } = &mut self.state {
            *last_world = world;
        }
    }
}
impl Tool for SelectTool {
    fn pointer_down(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        if let Some(id) = hit_test(ctx.document, ctx.pointer_world) {
            let mut collapse = None;
            if ctx.modifiers.shift {
                ctx.selection.toggle(id);
                if !ctx.selection.contains(id) {
                    self.state = SelectState::Idle;
                    return true;
                }
            } else if !ctx.selection.contains(id) {
                ctx.selection.set(id);
            } else {
                collapse = Some(id);
            }
            self.state = SelectState::Dragging {
                last_world: ctx.pointer_world,
                transaction: Transaction::begin(ctx.document, ctx.selection),
                collapse_on_click: collapse,
            };
        } else {
            let before = ctx.selection.clone();
            if !ctx.modifiers.shift {
                ctx.selection.clear();
            }
            ctx.selection.reserve(ctx.document.nodes().len());
            self.state = SelectState::BoxSelecting {
                start: ctx.pointer_world,
                current: ctx.pointer_world,
                before,
                additive: ctx.modifiers.shift,
            };
        }
        true
    }
    fn pointer_move(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        match &mut self.state {
            SelectState::Idle => false,
            SelectState::Dragging {
                last_world,
                transaction,
                ..
            } => {
                let delta = ctx.pointer_world - *last_world;
                let changed = transaction.preview(ctx.document, delta.x, delta.y);
                if changed {
                    *last_world = ctx.pointer_world;
                }
                changed
            }
            SelectState::BoxSelecting {
                start,
                current,
                before,
                additive,
            } => {
                if *current == ctx.pointer_world {
                    return false;
                }
                *current = ctx.pointer_world;
                let marquee = bounds(*start, *current);
                for node in ctx.document.nodes() {
                    let NodeKind::Rectangle(r) = &node.kind;
                    let intersects = r.x <= marquee.x1
                        && r.x + r.width >= marquee.x0
                        && r.y <= marquee.y1
                        && r.y + r.height >= marquee.y0;
                    if intersects || (*additive && before.contains(node.id)) {
                        ctx.selection.add(node.id);
                    } else {
                        ctx.selection.remove(node.id);
                    }
                }
                true
            }
        }
    }
    fn pointer_up(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        self.pointer_move(ctx);
        match std::mem::take(&mut self.state) {
            SelectState::Idle => false,
            SelectState::Dragging {
                transaction,
                collapse_on_click,
                ..
            } => {
                if let Some(command) = transaction.commit() {
                    ctx.history
                        .execute(Box::new(command), ctx.document, ctx.selection);
                } else if let Some(id) = collapse_on_click {
                    ctx.selection.set(id);
                }
                true
            }
            SelectState::BoxSelecting { .. } => true,
        }
    }
    fn cancel(&mut self, ctx: &mut ToolContext<'_>) -> bool {
        match std::mem::take(&mut self.state) {
            SelectState::Idle => false,
            SelectState::Dragging { transaction, .. } => {
                transaction.cancel(ctx.document);
                true
            }
            SelectState::BoxSelecting { before, .. } => {
                ctx.selection.replace(&before);
                true
            }
        }
    }
    fn cursor(&self) -> CursorStyle {
        CursorStyle::Arrow
    }
}
