use kurbo::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera2D {
    // Translation is measured in logical canvas pixels; zoom is independent of HiDPI.
    pub translation: Vec2,
    pub zoom: f64,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera2D {
    pub const MIN_ZOOM: f64 = 0.01;
    pub const MAX_ZOOM: f64 = 256.0;

    pub fn world_to_screen(&self, p: Vec2) -> Vec2 {
        p * self.zoom + self.translation
    }

    pub fn screen_to_world(&self, p: Vec2) -> Vec2 {
        (p - self.translation) / self.zoom
    }

    pub fn pan(&mut self, delta: Vec2) {
        let translation = self.translation + delta;
        if translation.is_finite() {
            self.translation = translation;
        }
    }

    pub fn zoom_at(&mut self, screen_pos: Vec2, factor: f64) {
        if !screen_pos.is_finite() || !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if zoom == self.zoom {
            return;
        }
        let world_pos = self.screen_to_world(screen_pos);
        let translation = screen_pos - world_pos * zoom;
        if translation.is_finite() {
            self.zoom = zoom;
            self.translation = translation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: Vec2, b: Vec2) {
        assert!((a - b).hypot() < 1e-8, "{a:?} != {b:?}");
    }

    #[test]
    fn coordinates_round_trip_after_pan_and_zoom() {
        let mut camera = Camera2D::default();
        camera.pan(Vec2::new(-213.5, 92.25));
        camera.zoom_at(Vec2::new(512.0, 384.0), 3.25);
        for world in [
            Vec2::ZERO,
            Vec2::new(-900.0, 200.0),
            Vec2::new(0.125, -0.25),
        ] {
            assert_close(camera.screen_to_world(camera.world_to_screen(world)), world);
        }
        assert_close(camera.world_to_screen(Vec2::ZERO), camera.translation);
    }

    #[test]
    fn zoom_preserves_cursor_anchor_including_both_clamps() {
        let cursor = Vec2::new(317.25, 201.5);
        let mut camera = Camera2D::default();
        camera.pan(Vec2::new(-47.5, 20.0));
        let anchor = camera.screen_to_world(cursor);
        for factor in [1.2, 0.8, 1e6, 2.0, 1e-9, 0.5, 2.0] {
            camera.zoom_at(cursor, factor);
            assert_close(camera.world_to_screen(anchor), cursor);
            assert!((Camera2D::MIN_ZOOM..=Camera2D::MAX_ZOOM).contains(&camera.zoom));
        }
    }

    #[test]
    fn invalid_input_and_clamped_zoom_do_not_move_camera() {
        let mut camera = Camera2D::default();
        let initial = camera;
        for factor in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            camera.zoom_at(Vec2::new(20.0, 10.0), factor);
        }
        camera.zoom_at(Vec2::new(f64::NAN, 0.0), 2.0);
        camera.pan(Vec2::new(f64::INFINITY, 0.0));
        assert_eq!(camera, initial);
        camera.zoom_at(Vec2::ZERO, 1e6);
        let clamped = camera;
        camera.zoom_at(Vec2::new(900.0, -20.0), 2.0);
        assert_eq!(camera, clamped);
    }
}
