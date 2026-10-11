use ruvector_canvas::{Camera2D, Viewport};
use ruvector_document::{Document, NodeId, Selection};
use std::sync::Arc;
use vello_gpu::{
    ClearSettings, RenderSize, RenderTargetConfig, Resources, Scene, TargetInit, TextureBindings,
};

use crate::{RenderError, vello::populate_document};

pub struct GpuRenderer {
    renderer: vello_gpu::Renderer,
    resources: Resources,
    scene: Scene,
    _texture: wgpu::Texture,
    view: Arc<wgpu::TextureView>,
    depth: wgpu::TextureView,
    size: RenderSize,
    rendered_state: Option<(Camera2D, u64, Option<NodeId>)>,
    viewport: Viewport,
}

impl GpuRenderer {
    pub fn new(device: &wgpu::Device, viewport: Viewport) -> Result<Self, RenderError> {
        let width = viewport.width();
        let height = viewport.height();
        let limit = device
            .limits()
            .max_texture_dimension_2d
            .min(u16::MAX.into());
        if width == 0 || height == 0 || width > limit || height > limit {
            return Err(RenderError::InvalidSize { width, height });
        }
        let size = RenderSize {
            width: width as u16,
            height: height as u16,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ruvector canvas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        let (renderer, resources) = vello_gpu::Renderer::new(
            device,
            &RenderTargetConfig {
                format: texture.format(),
                width: size.width,
                height: size.height,
            },
        );
        let depth = vello_gpu::Renderer::create_depth_texture_view(device, &size);
        Ok(Self {
            renderer,
            resources,
            scene: Scene::new(size.width, size.height),
            _texture: texture,
            view,
            depth,
            size,
            rendered_state: None,
            viewport,
        })
    }

    pub fn render(
        &mut self,
        camera: Camera2D,
        document: &Document,
        selection: &Selection,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Arc<wgpu::TextureView>, RenderError> {
        let state = (camera, document.revision(), selection.selected());
        if self.rendered_state == Some(state) {
            return Ok(Arc::clone(&self.view));
        }
        self.scene.reset();
        populate_document(&mut self.scene, document, selection, camera, self.viewport);
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                device,
                queue,
                encoder,
                &self.size,
                &self.view,
                Some(&self.depth),
                &TextureBindings::new(),
                TargetInit::Clear(ClearSettings::default()),
            )
            .map_err(|error| RenderError::Backend(error.to_string()))?;
        self.rendered_state = Some(state);
        Ok(Arc::clone(&self.view))
    }
}
