use gpui::{App, KeyBinding, actions};
actions!(
    editor,
    [
        ActivateSelectTool,
        ActivateRectangleTool,
        ActivateHandTool,
        Undo,
        Redo,
        DeleteSelection,
        DuplicateSelection
    ]
);
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("v", ActivateSelectTool, Some("Editor")),
        KeyBinding::new("m", ActivateRectangleTool, Some("Editor")),
        KeyBinding::new("h", ActivateHandTool, Some("Editor")),
        KeyBinding::new("delete", DeleteSelection, Some("Editor")),
        KeyBinding::new("backspace", DeleteSelection, Some("Editor")),
    ]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-z", Undo, Some("Editor")),
        KeyBinding::new("cmd-shift-z", Redo, Some("Editor")),
        KeyBinding::new("cmd-d", DuplicateSelection, Some("Editor")),
    ]);
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-z", Undo, Some("Editor")),
        KeyBinding::new("ctrl-shift-z", Redo, Some("Editor")),
        KeyBinding::new("ctrl-d", DuplicateSelection, Some("Editor")),
    ]);
}
