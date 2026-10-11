mod camera;
mod hit_test;
mod interaction;
mod state;

pub use camera::Camera2D;
pub use hit_test::hit_test;
pub use interaction::CanvasInteraction;
pub use kurbo::Vec2;
pub use state::CanvasState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale: f64,
}

impl Viewport {
    pub fn from_logical(width: f64, height: f64, scale: f64) -> Option<Self> {
        if !width.is_finite()
            || !height.is_finite()
            || !scale.is_finite()
            || width <= 0.0
            || height <= 0.0
            || scale <= 0.0
        {
            return None;
        }
        let width = (width * scale).ceil();
        let height = (height * scale).ceil();
        if width > f64::from(u16::MAX) || height > f64::from(u16::MAX) {
            return None;
        }
        Some(Self {
            width: width as u32,
            height: height as u32,
            scale,
        })
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }

    pub fn scale(self) -> f64 {
        self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidpi_rounds_outward_and_rejects_invalid_targets() {
        let viewport = Viewport::from_logical(100.25, 80.5, 1.5).unwrap();
        assert_eq!((viewport.width(), viewport.height()), (151, 121));
        for (width, height, scale) in [
            (0.0, 100.0, 1.0),
            (100.0, 100.0, 0.0),
            (f64::NAN, 100.0, 1.0),
            (100.0, 100.0, f64::INFINITY),
            (40000.0, 100.0, 2.0),
        ] {
            assert!(Viewport::from_logical(width, height, scale).is_none());
        }
    }
}
