use std::{cell::RefCell, rc::Rc};

use crate::input_adapter::logical_position;
use crate::{actions::*, input_adapter};
use gpui::{
    App, Bounds, Context, DispatchPhase, Element, ElementId, FocusHandle, GlobalElementId, Hitbox,
    HitboxBehavior, InspectorElementId, IntoElement, KeyDownEvent, KeyUpEvent, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PinchEvent, Pixels, Point, Render,
    ScrollDelta, ScrollWheelEvent, Style, Subscription, Window, div, prelude::*, rgb,
};
use ruvector_canvas::CanvasState;
use ruvector_editor::{EditorAction, EditorState, PointerPhase};
use ruvector_input::{Key, KeyEvent};
use ruvector_tools::{CursorStyle as ToolCursor, ToolId};

fn scroll_camera(state: &mut CanvasState, event: &ScrollWheelEvent, origin: Point<Pixels>) -> bool {
    let before = state.camera;
    let delta = logical_position(event.delta.pixel_delta(gpui::px(32.0)));
    if event.modifiers.control
        || event.modifiers.platform
        || matches!(event.delta, ScrollDelta::Lines(_))
    {
        // Limit extreme input before exp; Camera2D applies the actual zoom limits.
        let factor = (delta.y * 0.005).clamp(-20.0, 20.0).exp();
        state
            .camera
            .zoom_at(logical_position(event.position - origin), factor);
    } else {
        state.camera.pan(delta);
    }
    state.camera != before
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod native {
    use gpui::{
        AlphaMode, ExternalCompositorRegistry, ExternalSlotDescriptor, ExternalSlotFormat,
        ExternalSlotHandle,
    };
    use gpui_wgpu::{
        ExternalComposeOutput, WgpuCompositorBackendCtx, WgpuExternalCompositor,
        register_external_compositor,
    };
    use ruvector_canvas::Viewport;
    use ruvector_renderer::GpuRenderer;

    use super::*;
    struct Compositor {
        viewport: Viewport,
        navigation: Rc<RefCell<CanvasState>>,
        editor: Rc<RefCell<EditorState>>,
        renderer: Option<GpuRenderer>,
        view: gpui::WeakEntity<CanvasView>,
        app: gpui::AsyncApp,
    }

    impl WgpuExternalCompositor for Compositor {
        fn compose(
            &mut self,
            _slot: ExternalSlotHandle,
            context: &mut WgpuCompositorBackendCtx<'_>,
        ) -> ExternalComposeOutput {
            if self.renderer.is_none() {
                match GpuRenderer::new(&context.device, self.viewport) {
                    Ok(renderer) => {
                        tracing::debug!(
                            width = self.viewport.width(),
                            height = self.viewport.height(),
                            generation = context.context_generation,
                            "external canvas initialized"
                        );
                        self.renderer = Some(renderer);
                    }
                    Err(error) => {
                        tracing::error!(%error, "canvas initialization failed");
                        return ExternalComposeOutput::NotReady;
                    }
                }
            }
            let editor = self.editor.borrow();
            match self
                .renderer
                .as_mut()
                .expect("renderer initialized")
                .render(
                    self.navigation.borrow().camera,
                    &editor.document,
                    &editor.selection,
                    editor.tool_manager.overlay(),
                    &context.device,
                    &context.queue,
                    context.encoder,
                ) {
                Ok(view) => ExternalComposeOutput::Ready { view },
                Err(error) => {
                    tracing::error!(%error, "canvas rendering failed");
                    ExternalComposeOutput::NotReady
                }
            }
        }

        fn on_context_recreated(&mut self, _generation: u64) {
            self.renderer = None;
            let view = self.view.clone();
            let mut app = self.app.clone();
            self.app
                .foreground_executor()
                .spawn(async move {
                    let _ = view.update(&mut app, |_, cx| cx.notify());
                })
                .detach();
        }
    }

    struct Slot {
        registry: Rc<RefCell<ExternalCompositorRegistry>>,
        handle: ExternalSlotHandle,
        viewport: Viewport,
    }

    impl Drop for Slot {
        fn drop(&mut self) {
            let mut registry = self.registry.borrow_mut();
            let _ = registry.unregister(self.handle);
            registry.drain_pending_removals();
        }
    }

    #[derive(Default)]
    pub(super) struct GpuState {
        slot: Option<Slot>,
    }

    impl GpuState {
        pub(super) fn paint(
            gpu: &Rc<RefCell<Self>>,
            navigation: Rc<RefCell<CanvasState>>,
            editor: Rc<RefCell<EditorState>>,
            view: gpui::WeakEntity<CanvasView>,
            bounds: Bounds<Pixels>,
            window: &mut Window,
            cx: &mut App,
        ) {
            let Some(viewport) = Viewport::from_logical(
                f64::from(f32::from(bounds.size.width)),
                f64::from(f32::from(bounds.size.height)),
                f64::from(window.scale_factor()),
            ) else {
                gpu.borrow_mut().slot = None;
                return;
            };
            let Some(registry) = window.external_compositor_registry() else {
                return;
            };
            let mut state = gpu.borrow_mut();
            let matches = state.slot.as_ref().is_some_and(|slot| {
                slot.viewport == viewport && registry.borrow().is_valid(slot.handle)
            });
            if !matches {
                state.slot = None;
                let generation = registry.borrow().current_context_generation();
                match register_external_compositor(
                    &registry,
                    ExternalSlotDescriptor {
                        format: ExternalSlotFormat::Rgba8Unorm,
                        alpha_mode: AlphaMode::PreMultiplied,
                        width: viewport.width(),
                        height: viewport.height(),
                        sample_count: 1,
                        context_generation: generation,
                    },
                    Compositor {
                        viewport,
                        navigation,
                        editor,
                        renderer: None,
                        view,
                        app: cx.to_async(),
                    },
                ) {
                    Ok(handle) => {
                        state.slot = Some(Slot {
                            registry,
                            handle,
                            viewport,
                        });
                    }
                    Err(error) => tracing::error!(%error, "canvas registration failed"),
                }
            }
            if let Some(slot) = &state.slot {
                window.paint_external_compositor(bounds, slot.handle);
            }
        }
    }
}

struct CanvasElement {
    navigation: Rc<RefCell<CanvasState>>,
    editor: Rc<RefCell<EditorState>>,
    view: gpui::WeakEntity<CanvasView>,
    gpu_ready: bool,
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    gpu: Rc<RefCell<native::GpuState>>,
}

impl IntoElement for CanvasElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for CanvasElement {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some("gpu-canvas".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = gpui::relative(1.0).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        _: &mut App,
    ) -> Hitbox {
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        let cursor = match self.editor.borrow().tool_manager.cursor() {
            ToolCursor::Arrow => gpui::CursorStyle::Arrow,
            ToolCursor::Crosshair => gpui::CursorStyle::Crosshair,
            ToolCursor::Grab => gpui::CursorStyle::OpenHand,
            ToolCursor::Grabbing => gpui::CursorStyle::ClosedHand,
        };
        window.set_cursor_style(cursor, hitbox);
        if self.editor.borrow().tool_manager.is_interacting() {
            window.capture_pointer(hitbox.id);
        }
        let hitbox_down = hitbox.clone();
        let view = self.view.clone();
        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || event.button != MouseButton::Left
                || !hitbox_down.is_hovered(window)
                || !bounds.contains(&event.position)
            {
                return;
            }
            let _ = view.update(cx, |view, cx| {
                window.focus(&view.focus, cx);
                let changed = view.editor.borrow_mut().pointer_event(
                    &mut view.navigation.borrow_mut(),
                    input_adapter::pointer(
                        event.position,
                        bounds.origin,
                        Some(event.button),
                        event.modifiers,
                    ),
                    PointerPhase::Down,
                );
                if view.editor.borrow().tool_manager.is_interacting() {
                    window.capture_pointer(hitbox_down.id);
                }
                if changed {
                    view.scene_changed(cx);
                }
                cx.stop_propagation();
            });
        });
        let view = self.view.clone();
        let editor = self.editor.clone();
        let move_hitbox = hitbox.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            let interacting = editor.borrow().tool_manager.is_interacting();
            if interacting {
                if phase != DispatchPhase::Capture {
                    return;
                }
            } else if phase != DispatchPhase::Bubble
                || !move_hitbox.is_hovered(window)
                || !bounds.contains(&event.position)
            {
                return;
            }
            let _ = view.update(cx, |view, cx| {
                if interacting && event.pressed_button != Some(MouseButton::Left) {
                    view.cancel_interaction(window, cx);
                    return;
                }
                let changed = view.editor.borrow_mut().pointer_event(
                    &mut view.navigation.borrow_mut(),
                    input_adapter::pointer(
                        event.position,
                        bounds.origin,
                        event.pressed_button,
                        event.modifiers,
                    ),
                    PointerPhase::Move,
                );
                if changed {
                    view.scene_changed(cx);
                }
                if interacting {
                    cx.stop_propagation();
                }
            });
        });
        let view = self.view.clone();
        let editor = self.editor.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture
                || event.button != MouseButton::Left
                || !editor.borrow().tool_manager.is_interacting()
            {
                return;
            }
            let _ = view.update(cx, |view, cx| {
                let changed = view.editor.borrow_mut().pointer_event(
                    &mut view.navigation.borrow_mut(),
                    input_adapter::pointer(
                        event.position,
                        bounds.origin,
                        Some(event.button),
                        event.modifiers,
                    ),
                    PointerPhase::Up,
                );
                window.release_pointer();
                if changed {
                    view.scene_changed(cx);
                }
                cx.stop_propagation();
            });
        });
        let scroll_hitbox = hitbox.clone();
        let view = self.view.clone();
        window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || !scroll_hitbox.should_handle_scroll(window)
                || !bounds.contains(&event.position)
            {
                return;
            }
            let _ = view.update(cx, |view, cx| {
                let changed =
                    scroll_camera(&mut view.navigation.borrow_mut(), event, bounds.origin);
                if changed {
                    view.editor.borrow_mut().pointer_event(
                        &mut view.navigation.borrow_mut(),
                        input_adapter::pointer(
                            event.position,
                            bounds.origin,
                            None,
                            event.modifiers,
                        ),
                        PointerPhase::CameraChanged,
                    );
                    view.scene_changed(cx);
                }
                cx.stop_propagation();
            });
        });
        let pinch_hitbox = hitbox.clone();
        let view = self.view.clone();
        window.on_mouse_event(move |event: &PinchEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble
                || !pinch_hitbox.is_hovered(window)
                || !bounds.contains(&event.position)
            {
                return;
            }
            let _ = view.update(cx, |view, cx| {
                let changed = {
                    let mut state = view.navigation.borrow_mut();
                    let before = state.camera;
                    state.camera.zoom_at(
                        logical_position(event.position - bounds.origin),
                        1.0 + f64::from(event.delta),
                    );
                    state.camera != before
                };
                if changed {
                    view.editor.borrow_mut().pointer_event(
                        &mut view.navigation.borrow_mut(),
                        input_adapter::pointer(
                            event.position,
                            bounds.origin,
                            None,
                            window.modifiers(),
                        ),
                        PointerPhase::CameraChanged,
                    );
                    view.scene_changed(cx);
                }
                cx.stop_propagation();
            });
        });
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        if self.gpu_ready {
            native::GpuState::paint(
                &self.gpu,
                self.navigation.clone(),
                self.editor.clone(),
                self.view.clone(),
                bounds,
                window,
                cx,
            );
        }
        #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
        let _ = self.gpu_ready;
    }
}

pub(super) struct CanvasView {
    navigation: Rc<RefCell<CanvasState>>,
    editor: Rc<RefCell<EditorState>>,
    focus: FocusHandle,
    space_down: bool,
    gpu_ready: bool,
    _subscriptions: [Subscription; 2],
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    gpu: Rc<RefCell<native::GpuState>>,
}

impl CanvasView {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let subscriptions = [
            cx.observe_window_activation(window, |view, window, cx| {
                if !window.is_window_active() {
                    view.space_down = false;
                    view.cancel_interaction(window, cx);
                }
            }),
            cx.on_blur(&focus, window, |view, window, cx| {
                view.space_down = false;
                view.cancel_interaction(window, cx);
            }),
        ];
        let view = cx.entity().downgrade();
        window.on_next_frame(move |_, cx| {
            let _ = view.update(cx, |view, cx| {
                view.gpu_ready = true;
                cx.notify();
            });
        });
        Self {
            navigation: Default::default(),
            editor: Rc::new(RefCell::new(EditorState::seeded())),
            focus,
            space_down: false,
            gpu_ready: false,
            _subscriptions: subscriptions,
            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            gpu: Default::default(),
        }
    }

    pub(super) fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub(super) fn active_tool(&self) -> ToolId {
        self.editor.borrow().tool_manager.effective()
    }

    fn cancel_interaction(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let changed = self
            .editor
            .borrow_mut()
            .cancel(&mut self.navigation.borrow_mut());
        let hand_changed = if self.space_down {
            false
        } else {
            self.editor
                .borrow_mut()
                .temporary_hand(&mut self.navigation.borrow_mut(), false)
        };
        window.release_pointer();
        if changed || hand_changed {
            self.scene_changed(cx);
        }
    }

    fn action(&mut self, action: EditorAction, window: &mut Window, cx: &mut Context<Self>) {
        let changed = self
            .editor
            .borrow_mut()
            .action(&mut self.navigation.borrow_mut(), action);
        window.release_pointer();
        if changed {
            self.scene_changed(cx);
        }
        cx.stop_propagation();
    }
    fn scene_changed(&self, cx: &mut Context<Self>) {
        let camera = self.navigation.borrow().camera;
        tracing::debug!(
            x = camera.translation.x,
            y = camera.translation.y,
            zoom = camera.zoom,
            document_revision = self.editor.borrow().document.revision(),
            "canvas changed"
        );
        cx.notify();
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "space" {
            if !self.space_down {
                self.space_down = true;
                let changed = self
                    .editor
                    .borrow_mut()
                    .temporary_hand(&mut self.navigation.borrow_mut(), true);
                window.release_pointer();
                if changed {
                    self.scene_changed(cx);
                }
            }
            cx.stop_propagation();
        } else if event.keystroke.key == "escape" {
            let changed = self.editor.borrow_mut().key_event(
                &mut self.navigation.borrow_mut(),
                KeyEvent {
                    key: Key::Escape,
                    modifiers: input_adapter::modifiers(event.keystroke.modifiers),
                },
            );
            window.release_pointer();
            if changed {
                self.scene_changed(cx);
            }
            cx.stop_propagation();
        }
    }

    fn key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "space" {
            self.space_down = false;
            let changed = self
                .editor
                .borrow_mut()
                .temporary_hand(&mut self.navigation.borrow_mut(), false);
            window.release_pointer();
            if changed {
                self.scene_changed(cx);
            }
            cx.stop_propagation();
        }
    }

    fn modifiers_changed(
        &mut self,
        event: &gpui::ModifiersChangedEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let changed = self.editor.borrow_mut().key_event(
            &mut self.navigation.borrow_mut(),
            KeyEvent {
                key: Key::ModifiersChanged,
                modifiers: input_adapter::modifiers(event.modifiers),
            },
        );
        if changed {
            self.scene_changed(cx);
        }
    }
}

impl Render for CanvasView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("canvas")
            .track_focus(&self.focus)
            .key_context("Editor")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .bg(rgb(0x181b22))
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            .on_modifiers_changed(cx.listener(Self::modifiers_changed))
            .on_action(cx.listener(|this, _: &ActivateSelectTool, window, cx| {
                this.action(EditorAction::Activate(ToolId::Select), window, cx)
            }))
            .on_action(cx.listener(|this, _: &ActivateRectangleTool, window, cx| {
                this.action(EditorAction::Activate(ToolId::Rectangle), window, cx)
            }))
            .on_action(cx.listener(|this, _: &ActivateHandTool, window, cx| {
                this.action(EditorAction::Activate(ToolId::Hand), window, cx)
            }))
            .on_action(
                cx.listener(|this, _: &Undo, window, cx| {
                    this.action(EditorAction::Undo, window, cx)
                }),
            )
            .on_action(
                cx.listener(|this, _: &Redo, window, cx| {
                    this.action(EditorAction::Redo, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &DeleteSelection, window, cx| {
                this.action(EditorAction::Delete, window, cx)
            }))
            .on_action(cx.listener(|this, _: &DuplicateSelection, window, cx| {
                this.action(EditorAction::Duplicate, window, cx)
            }))
            .child(CanvasElement {
                navigation: self.navigation.clone(),
                editor: self.editor.clone(),
                view: cx.entity().downgrade(),
                gpu_ready: self.gpu_ready,
                #[cfg(any(target_os = "linux", target_os = "freebsd"))]
                gpu: self.gpu.clone(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, point, px};
    use ruvector_canvas::Vec2;

    #[test]
    fn wheel_zooms_about_canvas_local_cursor_even_without_modifiers() {
        let mut state = CanvasState::default();
        let origin = point(px(100.0), px(40.0));
        let event = ScrollWheelEvent {
            position: point(px(260.0), px(130.0)),
            delta: ScrollDelta::Lines(point(0.0, 1.0)),
            ..Default::default()
        };
        let cursor = Vec2::new(160.0, 90.0);
        let anchor = state.camera.screen_to_world(cursor);
        assert!(scroll_camera(&mut state, &event, origin));
        assert!(state.camera.zoom > 1.0);
        assert!((state.camera.world_to_screen(anchor) - cursor).hypot() < 1e-9);
    }

    #[test]
    fn precise_trackpad_pans_and_control_or_command_scroll_zooms() {
        let delta = point(px(12.5), px(-7.25));
        let event = ScrollWheelEvent {
            position: point(px(140.0), px(120.0)),
            delta: ScrollDelta::Pixels(delta),
            ..Default::default()
        };
        let mut state = CanvasState::default();
        assert!(scroll_camera(&mut state, &event, Point::default()));
        assert_eq!(state.camera.zoom, 1.0);
        assert_eq!(state.camera.translation, Vec2::new(60.5, 40.75));
        for modifiers in [
            Modifiers {
                control: true,
                ..Default::default()
            },
            Modifiers {
                platform: true,
                ..Default::default()
            },
        ] {
            let mut state = CanvasState::default();
            let cursor = logical_position(event.position);
            let anchor = state.camera.screen_to_world(cursor);
            let event = ScrollWheelEvent {
                modifiers,
                ..event.clone()
            };
            assert!(scroll_camera(&mut state, &event, Point::default()));
            assert!(state.camera.zoom < 1.0);
            assert!((state.camera.world_to_screen(anchor) - cursor).hypot() < 1e-9);
        }
    }

    #[test]
    fn zero_scroll_never_invalidates_camera() {
        let mut state = CanvasState::default();
        assert!(!scroll_camera(
            &mut state,
            &ScrollWheelEvent::default(),
            Point::default()
        ));
    }
}
