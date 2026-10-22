//! Translates an intermediate model to a wgpu-compatible one.

use slipstream_ir::tpl::{EncodedImage, Rgba8Image};
use slipstream_shared::SlipstreamResult;
use wgpu::util::DeviceExt;

use crate::viewer::pipeline::CameraState;
use crate::viewer::translation::{IntermediateModel, IntermediatePolygon};
use crate::viewer::wgpu::{PipelineDescriptor, PipelineRegistry, PipelineSignature};

pub struct WgpuModel {
    pub camera_bind_group: wgpu::BindGroup,

    pub bone_count: u32,
    pub bind_pose_group_layout: wgpu::BindGroupLayout,
    pub bind_pose_group: wgpu::BindGroup,

    pub pipelines: PipelineRegistry,

    pub textures: Vec<WgpuTexture>,
    pub polygons: Vec<WgpuPolygon>
}

impl WgpuModel {
    pub fn from_intermediate(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_state: &CameraState,
        ir: IntermediateModel
    ) -> SlipstreamResult<Self> {
        let bone_transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bone transform storage buffer"),
            contents: bytemuck::cast_slice(&ir.bone_transforms),
            usage: wgpu::BufferUsages::STORAGE
        });

        let bone_parent_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bone parent storage buffer"),
            contents: bytemuck::cast_slice(&ir.bone_parents),
            usage: wgpu::BufferUsages::STORAGE
        });

        let bone_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("transform matrix buffer group layout"),
            entries: &[
                // bone transformations
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        min_binding_size: None,
                        has_dynamic_offset: false
                    },
                    count: None
                },
                // bone parents
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        min_binding_size: None,
                        has_dynamic_offset: false
                    },
                    count: None
                }
            ]
        });

        let bone_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transform matrix buffer bind group"),
            layout: &bone_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &bone_transform_buffer,
                        offset: 0,
                        size: None
                    })
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &bone_parent_buffer,
                        offset: 0,
                        size: None
                    })
                }
            ]
        });

        let mut pipelines = PipelineRegistry::new(device.clone());

        let mut textures = Vec::with_capacity(ir.textures.len());
        for texture in ir.textures {
            textures.push(WgpuTexture::from_intermediate(device, queue, texture)?);
        }

        let mut polygons = Vec::with_capacity(ir.polygons.len());
        for polygon in ir.polygons {
            polygons.push(WgpuPolygon::from_intermediate(
                device,
                &bone_bg_layout,
                &camera_state.bind_group_layout,
                &mut pipelines,
                polygon
            )?);
        }

        Ok(Self {
            camera_bind_group: camera_state.bind_group.clone(),
            bone_count: ir.bone_transforms.len() as u32,
            bind_pose_group_layout: bone_bg_layout,
            bind_pose_group: bone_bind_group,
            pipelines,
            textures,
            polygons
        })
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        for polygon in &self.polygons {
            polygon.draw(
                &self.pipelines,
                &self.camera_bind_group,
                &self.bind_pose_group,
                render_pass
            );
        }
    }
}

pub struct WgpuTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView
}

impl WgpuTexture {
    pub fn from_intermediate(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba8: Rgba8Image
    ) -> SlipstreamResult<Self> {
        let dims = rgba8.dimensions();

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("translated model texture"),
                size: wgpu::Extent3d {
                    width: dims.x as u32,
                    height: dims.y as u32,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm]
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba8.bytes().expect("Rgba8Image::bytes never fails")
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("translated model texture view"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None
        });

        Ok(Self { texture, view })
    }
}

pub struct WgpuPolygon {
    pipeline: PipelineSignature,

    indices: u32,
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    uv_buffer: wgpu::Buffer
}

impl WgpuPolygon {
    pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
    pub const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: 72,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x4,
            3 => Uint32x4,
            4 => Float32x4,
        ],
        step_mode: wgpu::VertexStepMode::Vertex
    };
    pub const UV_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: 64,
        attributes: &wgpu::vertex_attr_array![
            5 => Float32x4,
            6 => Float32x4,
            7 => Float32x4,
            8 => Float32x4
        ],
        step_mode: wgpu::VertexStepMode::Vertex
    };

    pub fn from_intermediate(
        device: &wgpu::Device,
        bind_pose_layout: &wgpu::BindGroupLayout,
        camera_layout: &wgpu::BindGroupLayout,
        pipelines: &mut PipelineRegistry,
        ir: IntermediatePolygon
    ) -> SlipstreamResult<Self> {
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polygon index buffer"),
            contents: bytemuck::cast_slice(&ir.indices),
            usage: wgpu::BufferUsages::INDEX
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polygon vertex buffer"),
            contents: bytemuck::cast_slice(&ir.vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let uv_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("polygon uv buffer"),
            contents: bytemuck::cast_slice(&ir.uv_buffer),
            usage: wgpu::BufferUsages::VERTEX
        });

        let pipeline_signature = pipelines.register(PipelineDescriptor {
            name: "polygon",
            bind_group_layouts: &[Some(camera_layout), Some(bind_pose_layout)],
            vertex_layouts: &[Some(Self::VERTEX_LAYOUT), Some(Self::UV_LAYOUT)]
        });

        tracing::trace!("Generated wgpu model with {} indices", ir.indices.len());

        Ok(Self {
            pipeline: pipeline_signature,
            indices: ir.indices.len() as u32,

            vertex_buffer,
            uv_buffer,
            index_buffer
        })
    }

    pub fn draw(
        &self,
        pipelines: &PipelineRegistry,
        camera_group: &wgpu::BindGroup,
        bind_pose_group: &wgpu::BindGroup,
        render_pass: &mut wgpu::RenderPass
    ) {
        let pipeline = pipelines.get(self.pipeline).expect("pipeline not found");

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, camera_group, &[]);
        render_pass.set_bind_group(1, bind_pose_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.uv_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), Self::INDEX_FORMAT);
        render_pass.draw_indexed(0..self.indices, 0, 0..1);
    }
}
