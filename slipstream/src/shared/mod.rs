pub mod mem_logger;
pub mod object_dialog;
pub mod vertex;
pub mod wgsl_include;

use std::sync::Arc;

use eframe::egui_wgpu;
use egui::mutex::RwLock;

use crate::viewer::pipeline::ViewerPipeline;
use crate::viewer::registry::{ViewerId, ViewerRegistry};

#[derive(Clone)]
pub struct GraphicsState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub renderer: Arc<RwLock<egui_wgpu::Renderer>>
}

impl GraphicsState {
    pub fn insert(&self, pipeline: ViewerPipeline) -> ViewerId {
        let mut guard = self.renderer.write();
        let registry = guard
            .callback_resources
            .entry::<ViewerRegistry>()
            .or_insert_with(ViewerRegistry::default);

        registry.insert(pipeline)
    }

    pub fn inspect_viewer<F, T>(&self, viewer: ViewerId, inspect_fn: F) -> Option<T>
    where
        F: FnOnce(&ViewerPipeline) -> T
    {
        let guard = self.renderer.read();
        guard
            .callback_resources
            .get::<ViewerRegistry>()
            .and_then(|reg| reg.get(viewer).map(inspect_fn))
    }

    pub fn update_viewer<F, T>(&self, viewer: ViewerId, update_fn: F) -> Option<T>
    where
        F: FnOnce(&mut ViewerPipeline) -> T
    {
        let mut guard = self.renderer.write();
        guard
            .callback_resources
            .get_mut::<ViewerRegistry>()
            .and_then(|reg| reg.get_mut(viewer).map(update_fn))
    }
}
