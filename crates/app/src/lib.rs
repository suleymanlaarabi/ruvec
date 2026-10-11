use gpui::{App, AppContext, Bounds, KeyBinding, WindowBounds, WindowOptions, actions, px, size};
use ruvector_ui::EditorView;

actions!(ruvector, [Quit]);

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ruvector=info".into()),
        )
        .init();
    let args: Vec<_> = std::env::args().skip(1).collect();
    let smoke = args.iter().any(|arg| arg == "--smoke");
    let windows = if args.iter().any(|arg| arg == "--two-windows") {
        2
    } else {
        1
    };
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    tracing::warn!("the external GPU compositor currently requires Linux/FreeBSD");
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_base::init(cx);
        ruvector_ui::init(cx);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([
            #[cfg(target_os = "macos")]
            KeyBinding::new("cmd-q", Quit, None),
            #[cfg(not(target_os = "macos"))]
            KeyBinding::new("ctrl-q", Quit, None),
        ]);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        for _ in 0..windows {
            let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |window, cx| {
                    window.set_window_title("Ruvector");
                    let view = cx.new(|cx| EditorView::new(window, cx));
                    cx.new(|cx| gpui_base::Root::new(view, window, cx))
                },
            )
            .expect("failed to open Ruvector window");
        }
        cx.activate(true);
        if smoke {
            cx.spawn(async move |cx| {
                for (width, height) in [(1100.0, 720.0), (960.0, 640.0), (1280.0, 800.0)] {
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(1))
                        .await;
                    cx.update(|cx| {
                        for handle in cx.windows() {
                            let _ = handle.update(cx, |_, window, _| {
                                window.resize(size(px(width), px(height)));
                            });
                        }
                    });
                }
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                cx.update(|cx| cx.quit());
            })
            .detach();
        }
    });
}
