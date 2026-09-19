use std::sync::Arc;

use eframe::egui_wgpu;
use egui::mutex::RwLock;

#[derive(Clone)]
pub struct RenderState {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub renderer: Arc<RwLock<egui_wgpu::Renderer>>,
}
