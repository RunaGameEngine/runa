use std::{collections::HashMap, sync::Arc};

use crate::{resources::texture::GpuTexture, sprite::pipeline::SpritePipeline};
use runa_render_api::{command::RenderCommands, queue::RenderQueue};
use wgpu::util::DeviceExt;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub view_proj: [[f32; 4]; 4],
}

pub struct Renderer {
    pub context: GpuContext,

    sprite_pipeline: SpritePipeline,

    quad_vertex_buffer: wgpu::Buffer,
    quad_vertex_count: u32,

    globals_buffer: wgpu::Buffer,

    textures: HashMap<usize, GpuTexture>,
}

impl Renderer {
    pub fn new(context: GpuContext, format: wgpu::TextureFormat) -> Self {
        let sprite_pipeline = SpritePipeline::new(&context.device, format);

        let vertices: &[Vertex] = &[
            Vertex {
                position: [-0.5, -0.5],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5],
                tex_coords: [0.0, 1.0],
            },
        ];

        let quad_vertex_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let quad_vertex_count = vertices.len() as u32;

        let globals = Globals {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        };

        let globals_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Globals Buffer"),
                contents: bytemuck::bytes_of(&globals),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            context,
            sprite_pipeline,
            quad_vertex_buffer,
            quad_vertex_count,
            globals_buffer,
            textures: HashMap::new(),
        }
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &RenderQueue,
    ) {
        for cmd in &queue.commands {
            if let RenderCommands::Sprite { texture, model: _ } = cmd {
                let key = Arc::as_ptr(&texture.inner) as usize;

                let gpu_texture = self.textures.entry(key).or_insert_with(|| {
                    GpuTexture::from_asset(
                        &self.context.device,
                        &self.context.queue,
                        &texture.inner,
                    )
                });

                let bind_group =
                    self.context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.sprite_pipeline.bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: self.globals_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&gpu_texture.view),
                                },
                            ],
                            label: Some("Sprite BindGroup"),
                        });

                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Sprite Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                rpass.set_pipeline(&self.sprite_pipeline.pipeline);
                rpass.set_bind_group(0, &bind_group, &[]);
                rpass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                rpass.draw(0..self.quad_vertex_count, 0..1);
            }
        }
    }
}
