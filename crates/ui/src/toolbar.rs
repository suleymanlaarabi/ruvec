use crate::actions::{ActivateHandTool, ActivateRectangleTool, ActivateSelectTool};
use gpui::{
    Action, AppContext, Context, FocusHandle, IntoElement, Render, Window, div, prelude::*, px, rgb,
};
use ruvector_tools::ToolId;

struct Tooltip(&'static str);
impl Render for Tooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(0x303644))
            .text_color(rgb(0xe7eaf0))
            .text_sm()
            .child(self.0)
    }
}
fn button(
    id: &'static str,
    icon: &'static str,
    tooltip: &'static str,
    active: bool,
    action: impl Action,
    focus: FocusHandle,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(36.))
        .h(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .bg(rgb(if active { 0x3a568d } else { 0x252a34 }))
        .text_color(rgb(0xe7eaf0))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x3a4354)))
        .tooltip(move |_, cx| cx.new(|_| Tooltip(tooltip)).into())
        .on_click(move |_, window, cx| {
            window.focus(&focus, cx);
            window.dispatch_action(action.boxed_clone(), cx);
        })
        .child(icon)
}
pub(crate) fn toolbar(active: ToolId, focus: FocusHandle) -> impl IntoElement {
    div()
        .id("toolbar")
        .w(px(48.))
        .flex_shrink_0()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .py_2()
        .bg(rgb(0x20242c))
        .child(button(
            "select-tool",
            "↖",
            "Sélection (V)",
            active == ToolId::Select,
            ActivateSelectTool,
            focus.clone(),
        ))
        .child(button(
            "rectangle-tool",
            "□",
            "Rectangle (M) · Shift : carré · Alt : centre",
            active == ToolId::Rectangle,
            ActivateRectangleTool,
            focus.clone(),
        ))
        .child(button(
            "hand-tool",
            "✋",
            "Main (H) · maintenir Espace",
            active == ToolId::Hand,
            ActivateHandTool,
            focus,
        ))
}
