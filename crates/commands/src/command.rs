use ruvector_document::{Document, Selection};
pub trait Command {
    fn apply(&mut self, document: &mut Document) -> bool;
    fn undo(&mut self, document: &mut Document);
    fn update_selection(&self, _selection: &mut Selection) {}
}
