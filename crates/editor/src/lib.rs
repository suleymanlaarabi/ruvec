use ruvector_canvas::{CanvasState, Vec2};
use ruvector_commands::{DeleteNodes, DuplicateNodes, History};
use ruvector_document::{Document, Selection};
use ruvector_input::{KeyEvent, Modifiers, PointerButton, PointerEvent};
use ruvector_tools::{ToolContext, ToolId, ToolManager};

#[derive(Clone, Copy)]
pub enum EditorAction {
    Activate(ToolId),
    Undo,
    Redo,
    Delete,
    Duplicate,
}
#[derive(Clone, Copy)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
    CameraChanged,
}

#[derive(Default)]
pub struct EditorState {
    pub document: Document,
    pub selection: Selection,
    pub history: History,
    pub tool_manager: ToolManager,
    pointer: Vec2,
    modifiers: Modifiers,
}
impl EditorState {
    pub fn seeded() -> Self {
        let mut state = Self::default();
        state.document.create_rectangle(0., 0., 300., 200.);
        state.document.create_rectangle(210., 130., 260., 180.);
        state.document.create_rectangle(530., 45., 180., 260.);
        state
    }
    fn context<'a>(
        document: &'a mut Document,
        selection: &'a mut Selection,
        history: &'a mut History,
        canvas: &'a mut CanvasState,
        pointer: Vec2,
        modifiers: Modifiers,
    ) -> ToolContext<'a> {
        let pointer_world = canvas.camera.screen_to_world(pointer);
        ToolContext {
            document,
            selection,
            history,
            canvas,
            pointer_screen: pointer,
            pointer_world,
            modifiers,
        }
    }
    pub fn pointer_event(
        &mut self,
        canvas: &mut CanvasState,
        event: PointerEvent,
        phase: PointerPhase,
    ) -> bool {
        if !event.position.is_finite() {
            return false;
        }
        self.pointer = event.position;
        self.modifiers = event.modifiers;
        if matches!(phase, PointerPhase::Down | PointerPhase::Up)
            && event.button != Some(PointerButton::Left)
        {
            return false;
        }
        let mut ctx = Self::context(
            &mut self.document,
            &mut self.selection,
            &mut self.history,
            canvas,
            self.pointer,
            self.modifiers,
        );
        match phase {
            PointerPhase::Down => self.tool_manager.pointer_down(&mut ctx),
            PointerPhase::Move => self.tool_manager.pointer_move(&mut ctx),
            PointerPhase::Up => self.tool_manager.pointer_up(&mut ctx),
            PointerPhase::CameraChanged => self.tool_manager.camera_changed(&mut ctx),
        }
    }
    pub fn temporary_hand(&mut self, canvas: &mut CanvasState, active: bool) -> bool {
        let mut ctx = Self::context(
            &mut self.document,
            &mut self.selection,
            &mut self.history,
            canvas,
            self.pointer,
            self.modifiers,
        );
        self.tool_manager.temporary_hand(active, &mut ctx)
    }
    pub fn cancel(&mut self, canvas: &mut CanvasState) -> bool {
        let mut ctx = Self::context(
            &mut self.document,
            &mut self.selection,
            &mut self.history,
            canvas,
            self.pointer,
            self.modifiers,
        );
        self.tool_manager.cancel(&mut ctx)
    }
    pub fn key_event(&mut self, canvas: &mut CanvasState, event: KeyEvent) -> bool {
        self.modifiers = event.modifiers;
        let mut ctx = Self::context(
            &mut self.document,
            &mut self.selection,
            &mut self.history,
            canvas,
            self.pointer,
            self.modifiers,
        );
        self.tool_manager.key_down(event, &mut ctx)
    }
    pub fn action(&mut self, canvas: &mut CanvasState, action: EditorAction) -> bool {
        let mut ctx = Self::context(
            &mut self.document,
            &mut self.selection,
            &mut self.history,
            canvas,
            self.pointer,
            self.modifiers,
        );
        let canceled = self.tool_manager.cancel(&mut ctx);
        let changed = match action {
            EditorAction::Activate(id) => self.tool_manager.activate(id, &mut ctx),
            EditorAction::Undo => ctx.history.undo(ctx.document, ctx.selection),
            EditorAction::Redo => ctx.history.redo(ctx.document, ctx.selection),
            EditorAction::Delete => ctx.history.execute(
                Box::new(DeleteNodes::new(ctx.selection)),
                ctx.document,
                ctx.selection,
            ),
            EditorAction::Duplicate => ctx.history.execute(
                Box::new(DuplicateNodes::new(ctx.document, ctx.selection)),
                ctx.document,
                ctx.selection,
            ),
        };
        changed || canceled
    }
}
