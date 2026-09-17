#![feature(error_generic_member_access)]

pub mod app;
#[macro_use]
pub mod icons;
pub mod config;
pub mod decorations;
pub mod format;
pub mod model_renderer;
pub mod pages;
pub mod shared;
pub mod r#virtual;

pub mod error;
#[cfg(target_arch = "wasm32")]
mod web;

use crate::app::App;
use error::EditorResult;

fn setup_tracing() {
    #[cfg(target_arch = "wasm32")]
    {
        tracing_wasm::set_as_global_default();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt;

        color_eyre::config::HookBuilder::new()
            .panic_section("report this issue at https://github.com/RadiatedMonkey/mktools")
            // .issue_url("https://github.com/RadiatedMonkey/mktools/issues/new")
            // .add_issue_metadata("version", "v0.1.0")
            .display_location_section(true)
            .display_env_section(true)
            .install()
            .unwrap();

        let filter = tracing_subscriber::filter::filter_fn(|meta| !meta.target().contains("winit"));

        let layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_targets(true)
            .with_bracketed_fields(true)
            .with_filter(filter);

        let subscriber = tracing_subscriber::Registry::default().with(layer);

        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    tracing::debug!("Logging initialized");
}

fn window_builder_hook(builder: egui::ViewportBuilder) -> egui::ViewportBuilder {
    builder
        .with_title("Mario Kart Wii editor")
        .with_inner_size(egui::Vec2::new(600.0, 200.0))
        .with_decorations(false)
        .with_resizable(false)
}

pub fn run() -> EditorResult<()> {
    setup_tracing();

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::Arc;

        use eframe::egui_wgpu::SurfaceErrorAction;

        let wgpu_setup = eframe::egui_wgpu::WgpuSetupCreateNew {
            instance_descriptor: wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                backend_options: wgpu::BackendOptions::default(),
                display: None,
                flags: wgpu::InstanceFlags::empty(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            },
            device_descriptor: Arc::new(|adapter| {
                let adapter_info = adapter.get_info();

                tracing::info!("Using adapter {}", adapter_info.name);

                wgpu::DeviceDescriptor {
                    label: Some("device"),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::defaults(),
                    trace: wgpu::Trace::Off,
                }
            }),
            native_adapter_selector: None,
            display_handle: None,
            power_preference: wgpu::PowerPreference::default(),
        };

        let wgpu_options = eframe::WgpuConfiguration {
            on_surface_status: Arc::new(|status| match status {
                wgpu::CurrentSurfaceTexture::Outdated => {
                    tracing::debug!("Dropped frame with error: {status:?}");
                    SurfaceErrorAction::Reconfigure
                }
                wgpu::CurrentSurfaceTexture::Lost => {
                    tracing::debug!("Dropped frame with error: {status:?}");
                    SurfaceErrorAction::RecreateSurface
                }
                wgpu::CurrentSurfaceTexture::Occluded => SurfaceErrorAction::SkipFrame,
                _ => {
                    tracing::warn!("Dropped frame with error: {status:?}");
                    SurfaceErrorAction::SkipFrame
                }
            }),
            surface: eframe::SurfaceConfig::HIGH_THROUGHPUT,
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(wgpu_setup),
        };

        let native_options = eframe::NativeOptions {
            centered: true,
            dithering: true,
            renderer: eframe::Renderer::Wgpu,
            window_builder: Some(Box::new(window_builder_hook)),
            wgpu_options,
            ..Default::default()
        };

        eframe::run_native(
            "MKTools Editor",
            native_options,
            Box::new(|cc| Ok(Box::new(App::new(cc)))),
        )?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        todo!();
    }

    Ok(())
}
