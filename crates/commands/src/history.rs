use crate::Command;
use ruvector_document::{Document, Selection};

struct Entry {
    command: Box<dyn Command>,
    before: Selection,
    after: Selection,
}

#[derive(Default)]
pub struct History {
    undo_stack: Vec<Entry>,
    redo_stack: Vec<Entry>,
}

impl History {
    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        document: &mut Document,
        selection: &mut Selection,
    ) -> bool {
        let before = selection.clone();
        if !command.apply(document) {
            return false;
        }
        command.update_selection(selection);
        self.redo_stack.clear();
        self.undo_stack.push(Entry {
            command,
            before,
            after: selection.clone(),
        });
        true
    }
    pub fn undo(&mut self, document: &mut Document, selection: &mut Selection) -> bool {
        let Some(mut entry) = self.undo_stack.pop() else {
            return false;
        };
        entry.command.undo(document);
        selection.replace(&entry.before);
        self.redo_stack.push(entry);
        true
    }
    pub fn redo(&mut self, document: &mut Document, selection: &mut Selection) -> bool {
        let Some(mut entry) = self.redo_stack.pop() else {
            return false;
        };
        entry.command.apply(document);
        selection.replace(&entry.after);
        self.undo_stack.push(entry);
        true
    }
}
