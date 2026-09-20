use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::mpsc,
};

use crate::{
    node::refs::{VirtualNodeId, VirtualNodeMap},
    panes::{ContentSignature, Pane, PaneAction},
    viewer::RenderState,
};

pub struct ViewerPipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,

    output: wgpu::Texture,
    output_view: wgpu::TextureView,

    msaa: wgpu::Texture,
    msaa_view: wgpu::TextureView,

    depth: wgpu::Texture,
    depth_view: wgpu::TextureView,

    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl ViewerPipeline {
    pub fn new(render_state: &RenderState) -> Self {
        todo!()
    }

    pub fn on_resize(&mut self, new_size: glam::UVec2) {}
}

pub struct ViewerPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    content_sig: ContentSignature,
    node: VirtualNodeId,
    node_map: VirtualNodeMap,
}

impl ViewerPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        content_sig: ContentSignature,
        node: VirtualNodeId,
        node_map: VirtualNodeMap,
    ) -> Box<dyn Pane> {
        // let pipeline = ViewerPipeline::new();

        Box::new(Self {
            cmd_sender,
            content_sig,
            node,
            node_map,
            // pipeline,
        })
    }
}

impl Pane for ViewerPane {
    fn content_signature(&self) -> ContentSignature {
        self.content_sig
    }

    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("3D Viewer"))
    }

    fn draw(&mut self, ui: &mut egui::Ui, tile_id: egui_tiles::TileId) -> egui_tiles::UiResponse {
        // ui.label("Viewing `{}`")

        if ui.label("this is a viewer").drag_started() {
            return egui_tiles::UiResponse::DragStarted;
        }

        egui_tiles::UiResponse::None
    }
}
