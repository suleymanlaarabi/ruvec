use gpui::{
    AlphaMode, ExternalCompositorError, ExternalCompositorRegistry, ExternalSlotDescriptor,
    ExternalSlotFormat,
};

fn descriptor(generation: u64) -> ExternalSlotDescriptor {
    ExternalSlotDescriptor {
        format: ExternalSlotFormat::Rgba8Unorm,
        alpha_mode: AlphaMode::PreMultiplied,
        width: 128,
        height: 128,
        sample_count: 1,
        context_generation: generation,
    }
}

#[test]
fn stale_handle_cannot_remove_a_new_canvas() {
    let mut registry = ExternalCompositorRegistry::new();
    let original = registry.register(descriptor(1), Box::new(())).unwrap();
    registry.unregister(original).unwrap();
    let replacement = registry.register(descriptor(1), Box::new(())).unwrap();
    assert_ne!(original, replacement);
    assert!(matches!(
        registry.unregister(original),
        Err(ExternalCompositorError::StaleHandle)
    ));
    assert!(registry.is_valid(replacement));
}

#[test]
fn device_recreation_invalidates_every_canvas_and_allows_reregistration() {
    let mut registry = ExternalCompositorRegistry::new();
    let first = registry.register(descriptor(1), Box::new(())).unwrap();
    let second = registry.register(descriptor(1), Box::new(())).unwrap();
    registry.on_context_recreated(2);
    assert!(!registry.is_valid(first));
    assert!(!registry.is_valid(second));
    assert!(matches!(
        registry.register(descriptor(1), Box::new(())),
        Err(ExternalCompositorError::GenerationMismatch { got: 1, current: 2 })
    ));
    registry.unregister(first).unwrap();
    registry.unregister(second).unwrap();
    let current = registry.register(descriptor(2), Box::new(())).unwrap();
    assert!(registry.is_valid(current));
}

#[test]
fn unresolved_multisample_textures_cannot_be_registered() {
    let mut registry = ExternalCompositorRegistry::new();
    let mut target = descriptor(1);
    target.sample_count = 4;
    assert!(matches!(
        registry.register(target, Box::new(())),
        Err(ExternalCompositorError::InvalidSampleCount(4))
    ));
}
