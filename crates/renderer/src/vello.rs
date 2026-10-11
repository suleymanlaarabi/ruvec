use ruvector_canvas::{Camera2D, Viewport};
use ruvector_document::{Document, NodeKind, Selection};
use ruvector_tools::ToolOverlay;
use vello_gpu::{
    Scene,
    kurbo::{Affine, Rect, Stroke},
    peniko::color::AlphaColor,
};

fn world_to_physical(camera: Camera2D, viewport: Viewport) -> Affine {
    // World -> logical canvas -> physical GPU pixels, in that order.
    Affine::scale(viewport.scale())
        * Affine::translate(camera.translation)
        * Affine::scale(camera.zoom)
}

pub(super) fn populate_document(
    scene: &mut Scene,
    document: &Document,
    selection: &Selection,
    overlay: ToolOverlay,
    camera: Camera2D,
    viewport: Viewport,
) {
    scene.set_transform(world_to_physical(camera, viewport));
    scene.set_paint(AlphaColor::from_rgb8(75, 124, 240));
    for node in document.nodes() {
        let NodeKind::Rectangle(rect) = &node.kind;
        scene.fill_rect(&Rect::new(
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
        ));
    }
    for node in document.nodes().filter(|node| selection.contains(node.id)) {
        let NodeKind::Rectangle(rect) = &node.kind;
        // Draw the overlay in logical space so its width does not depend on zoom.
        let top_left = camera.world_to_screen(ruvector_canvas::Vec2::new(rect.x, rect.y));
        let bottom_right = camera.world_to_screen(ruvector_canvas::Vec2::new(
            rect.x + rect.width,
            rect.y + rect.height,
        ));
        scene.set_transform(Affine::scale(viewport.scale()));
        scene.set_stroke(Stroke::new(1.5));
        scene.set_paint(AlphaColor::from_rgb8(255, 205, 95));
        scene.stroke_rect(&Rect::new(
            top_left.x,
            top_left.y,
            bottom_right.x,
            bottom_right.y,
        ));
    }
    match overlay {
        ToolOverlay::None => {}
        ToolOverlay::RectanglePreview { rect } => {
            scene.set_transform(world_to_physical(camera, viewport));
            scene.set_paint(AlphaColor::from_rgba8(75, 124, 240, 140));
            scene.fill_rect(&rect);
            stroke_overlay(
                scene,
                rect,
                camera,
                viewport,
                AlphaColor::from_rgb8(125, 170, 255),
            );
        }
        ToolOverlay::SelectionMarquee { rect } => {
            scene.set_transform(world_to_physical(camera, viewport));
            scene.set_paint(AlphaColor::from_rgba8(125, 170, 255, 35));
            scene.fill_rect(&rect);
            stroke_overlay(
                scene,
                rect,
                camera,
                viewport,
                AlphaColor::from_rgb8(125, 170, 255),
            );
        }
    }
}

fn stroke_overlay(
    scene: &mut Scene,
    rect: Rect,
    camera: Camera2D,
    viewport: Viewport,
    color: AlphaColor<vello_gpu::peniko::color::Srgb>,
) {
    let a = camera.world_to_screen(ruvector_canvas::Vec2::new(rect.x0, rect.y0));
    let b = camera.world_to_screen(ruvector_canvas::Vec2::new(rect.x1, rect.y1));
    scene.set_transform(Affine::scale(viewport.scale()));
    scene.set_stroke(Stroke::new(1.));
    scene.set_paint(color);
    scene.stroke_rect(&Rect::new(a.x, a.y, b.x, b.y));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruvector_canvas::Vec2;
    use vello_gpu::kurbo::Point;

    #[test]
    fn hidpi_and_resize_preserve_logical_camera_and_world_anchor() {
        let mut camera = Camera2D::default();
        camera.pan(Vec2::new(40.0, -10.0));
        camera.zoom_at(Vec2::new(200.0, 120.0), 2.5);
        let world = Vec2::new(60.0, 80.0);
        let logical = camera.world_to_screen(world);
        for (width, height, scale) in [
            (800.0, 600.0, 1.0),
            (600.0, 400.0, 2.0),
            (1200.0, 800.0, 1.5),
        ] {
            let viewport = Viewport::from_logical(width, height, scale).unwrap();
            let physical = world_to_physical(camera, viewport) * Point::new(world.x, world.y);
            assert!((physical.to_vec2() - logical * scale).hypot() < 1e-9);
        }
    }
}
