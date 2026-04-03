use std::{collections::HashMap, sync::Arc};

use crate::{
    font::FontManager, pipelines::MeshPipeline, pipelines::MeshUniforms, pipelines::SpritePipeline,
    pipelines::UIPipeline, pipelines::UITexturedVertex, pipelines::UIUniforms, pipelines::UIVertex,
    resources::texture::GpuTexture,
};
use glam::Vec2;
use runa_asset::TextureAsset;
use runa_render_api::{RenderCommands, RenderQueue};
use wgpu::util::DeviceExt;
use wgpu::{MemoryHints::Performance, Trace};
use wgpu::{Texture, TextureView};
use winit::window::Window;

/// Per-instance data for sprite/tile rendering.
/// Contains transform, UV coordinates, and flip information.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub position: [f32; 3],  // x, y, z
    pub rotation: f32,       // radians
    pub scale: [f32; 3],     // x, y, z
    pub uv_offset: [f32; 2], // left-bottom UV coordinates
    pub uv_size: [f32; 2],   // UV quad size
    pub flip: u32,           // bit 0 = flip_x, bit 1 = flip_y
    pub _pad: f32,
}

/// Vertex structure for sprite quads.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

/// Global uniform buffer data containing view-projection matrix and aspect ratio.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub view_proj: [[f32; 4]; 4],
    pub aspect: f32,
    pub _padding: [f32; 7],
}

/// Offscreen render target used by the editor viewport and previews.
pub struct RenderTarget {
    _color_texture: Texture,
    color_view: TextureView,
    _depth_texture: Texture,
    depth_view: TextureView,
    size: (u32, u32),
    _format: wgpu::TextureFormat,
}

impl RenderTarget {
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn color_view(&self) -> &TextureView {
        &self.color_view
    }
}

/// Main renderer struct managing GPU resources and rendering.
pub struct Renderer<'window> {
    pub surface: wgpu::Surface<'window>,
    pub surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    sprite_pipeline: SpritePipeline,

    mesh_pipeline: MeshPipeline,

    ui_pipeline: UIPipeline,
    ui_uniform_buffer: wgpu::Buffer,
    ui_bind_group: Option<wgpu::BindGroup>,

    globals_buffer: wgpu::Buffer,

    textures: HashMap<usize, Arc<GpuTexture>>,
    nearest_sampler: wgpu::Sampler,

    font_manager: FontManager,

    textures_cache: HashMap<usize, Arc<TextureAsset>>,
    bind_group_cache: HashMap<usize, wgpu::BindGroup>,

    depth_view: TextureView,
    depth_texture: Texture,

    /// Base quad vertices (6 vertices, static).
    quad_buffer: wgpu::Buffer,
    /// Dynamic instance buffer - resized as needed.
    instance_buffer: wgpu::Buffer,
    /// Current capacity of instance buffer in number of instances.
    instance_buffer_capacity: usize,
}

impl<'window> Renderer<'window> {
    /// Creates a new renderer with the given window and vsync setting.
    ///
    /// # Arguments
    /// * `window` - The window to render to
    /// * `vsync` - Enable vertical sync for frame presentation
    pub async fn new_async(window: Arc<Window>, vsync: bool) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: Default::default(),
                memory_hints: Performance,
                trace: Trace::Off,
            })
            .await
            .expect("Failed to create device");

        let size = window.inner_size();

        let capabilities = surface.get_capabilities(&adapter);
        let preferred_format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| *format == wgpu::TextureFormat::Rgba8UnormSrgb)
            .or_else(|| {
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(|format| *format == wgpu::TextureFormat::Bgra8UnormSrgb)
            })
            .or_else(|| {
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(|format| *format == wgpu::TextureFormat::Rgba8Unorm)
            })
            .or_else(|| {
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(|format| *format == wgpu::TextureFormat::Bgra8Unorm)
            })
            .unwrap_or(capabilities.formats[0]);

        let surface_config: wgpu::SurfaceConfiguration;
        if vsync {
            surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: preferred_format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
        } else {
            surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: preferred_format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::AutoNoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
        }

        surface.configure(&device, &surface_config);

        let sprite_pipeline = SpritePipeline::new(
            &device,
            surface_config.format,
            wgpu::TextureFormat::Depth32Float,
        );

        let identity_mat = glam::Mat4::IDENTITY.to_cols_array_2d();
        let globals = Globals {
            view_proj: identity_mat,
            aspect: surface_config.width as f32 / surface_config.height as f32,
            _padding: [0.0; 7],
        };

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Buffer"),
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Pixel Art Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let font_manager = FontManager::new(&device, &queue);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Depth Texture"),
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 3D mesh pipeline
        let mesh_pipeline = MeshPipeline::new(
            &device,
            surface_config.format,
            wgpu::TextureFormat::Depth32Float,
        );

        // UI pipeline for debug rectangles and text
        let ui_pipeline = UIPipeline::new(&device, surface_config.format);

        let ui_uniforms = UIUniforms {
            screen_width: surface_config.width as f32,
            screen_height: surface_config.height as f32,
        };

        let ui_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Uniform Buffer"),
            contents: bytemuck::bytes_of(&ui_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let ui_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ui_pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ui_uniform_buffer.as_entire_binding(),
            }],
            label: Some("UI Bind Group"),
        });

        const QUAD_VERTICES: &[Vertex] = &[
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Instance buffer with initial capacity
        const INITIAL_INSTANCE_CAPACITY: usize = 1000;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceData>() * INITIAL_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            surface_config,
            device,
            queue,
            sprite_pipeline,
            mesh_pipeline,
            ui_pipeline,
            ui_uniform_buffer,
            ui_bind_group: Some(ui_bind_group),
            globals_buffer,
            textures: HashMap::new(),
            nearest_sampler,
            font_manager,
            textures_cache: HashMap::new(),
            bind_group_cache: HashMap::new(),
            depth_view,
            depth_texture,
            quad_buffer,
            instance_buffer,
            instance_buffer_capacity: INITIAL_INSTANCE_CAPACITY,
        }
    }

    /// Creates a new renderer synchronously (blocking).
    pub fn new(window: Arc<Window>, vsync: bool) -> Self {
        pollster::block_on(Self::new_async(window, vsync))
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    pub fn create_render_target(&self, size: (u32, u32)) -> RenderTarget {
        Self::build_render_target(&self.device, size, self.surface_config.format)
    }

    /// Resizes the surface and recreates the depth texture.
    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
        let depth = Self::create_depth_texture(
            &self.device,
            (self.surface_config.width, self.surface_config.height),
        );
        self.depth_texture = depth.0;
        self.depth_view = depth.1;

        // Update UI uniforms
        let ui_uniforms = UIUniforms {
            screen_width: self.surface_config.width as f32,
            screen_height: self.surface_config.height as f32,
        };
        self.queue
            .write_buffer(&self.ui_uniform_buffer, 0, bytemuck::bytes_of(&ui_uniforms));
    }

    /// Renders the current frame using the provided render queue and camera matrix.
    pub fn draw(&mut self, queue: &RenderQueue, camera_matrix: glam::Mat4, virtual_size: Vec2) {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            _ => return, // Surface lost, timeout, etc.
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: None,
                ..Default::default()
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let depth_view = self.depth_view.clone();
        self.encode_render_passes(
            &mut encoder,
            &view,
            &depth_view,
            (self.surface_config.width, self.surface_config.height),
            queue,
            camera_matrix,
            virtual_size,
        );
        self.queue.submit(Some(encoder.finish()));
        let _ = self.device.poll(wgpu::PollType::Poll);
        surface_texture.present();
    }

    pub fn draw_to_target(
        &mut self,
        target: &RenderTarget,
        queue: &RenderQueue,
        camera_matrix: glam::Mat4,
        virtual_size: Vec2,
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Offscreen Render Encoder"),
            });
        self.encode_render_passes(
            &mut encoder,
            target.color_view(),
            &target.depth_view,
            target.size(),
            queue,
            camera_matrix,
            virtual_size,
        );
        self.queue.submit(Some(encoder.finish()));
    }

    fn encode_render_passes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &TextureView,
        depth_view: &TextureView,
        target_size: (u32, u32),
        queue: &RenderQueue,
        camera_matrix: glam::Mat4,
        virtual_size: Vec2,
    ) {
        let target_aspect = target_size.0.max(1) as f32 / target_size.1.max(1) as f32;
        self.queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::bytes_of(&Globals {
                view_proj: camera_matrix.to_cols_array_2d(),
                aspect: (virtual_size.x / virtual_size.y) / target_aspect,
                _padding: [0.0; 7],
            }),
        );

        let ui_uniforms = UIUniforms {
            screen_width: target_size.0.max(1) as f32,
            screen_height: target_size.1.max(1) as f32,
        };
        self.queue
            .write_buffer(&self.ui_uniform_buffer, 0, bytemuck::bytes_of(&ui_uniforms));

        let mut all_instances = Vec::new();
        let mut batches = Vec::new();
        let mut ui_vertices = Vec::new();
        let mut ui_text_vertices = Vec::new();

        for cmd in &queue.commands {
            match cmd {
                RenderCommands::Mesh3D { .. } => {}
                RenderCommands::Sprite {
                    texture,
                    position,
                    rotation,
                    scale,
                } => {
                    let tex_width = texture.width as f32;
                    let tex_height = texture.height as f32;
                    let world_scale_x = scale.x * (tex_width / 16.0);
                    let world_scale_y = scale.y * (tex_height / 16.0);

                    let instance = InstanceData {
                        position: [position.x, position.y, position.z],
                        rotation: rotation.z,
                        scale: [world_scale_x, world_scale_y, 1.0],
                        uv_offset: [0.0, 0.0],
                        uv_size: [1.0, 1.0],
                        flip: 0,
                        _pad: 0.0,
                    };

                    let key = Arc::as_ptr(texture) as usize;
                    if !self.textures_cache.contains_key(&key) {
                        self.textures_cache.insert(key, texture.clone());
                    }

                    let offset = all_instances.len();
                    all_instances.push(instance);
                    batches.push((key, offset, 1));
                }
                RenderCommands::Tile {
                    texture,
                    position,
                    size,
                    uv_rect,
                    flip_x,
                    flip_y,
                    color: _,
                } => {
                    let instance = InstanceData {
                        position: [position.x, position.y, position.z],
                        rotation: 0.0,
                        scale: [size.x as f32, size.y as f32, 1.0],
                        uv_offset: [uv_rect[0], uv_rect[1]],
                        uv_size: [uv_rect[2], uv_rect[3]],
                        flip: ((*flip_x) as u32) | (((*flip_y) as u32) << 1),
                        _pad: 0.0,
                    };

                    let key = Arc::as_ptr(texture) as usize;
                    if !self.textures_cache.contains_key(&key) {
                        self.textures_cache.insert(key, texture.clone());
                    }

                    let offset = all_instances.len();
                    all_instances.push(instance);
                    batches.push((key, offset, 1));
                }
                RenderCommands::DebugRect {
                    position,
                    size,
                    color,
                } => {
                    let left = position.x - size.x / 2.0;
                    let top = position.y - size.y / 2.0;
                    let right = left + size.x;
                    let bottom = top + size.y;

                    ui_vertices.extend_from_slice(&[
                        UIVertex {
                            position: [left, top],
                            color: *color,
                        },
                        UIVertex {
                            position: [right, top],
                            color: *color,
                        },
                        UIVertex {
                            position: [left, bottom],
                            color: *color,
                        },
                        UIVertex {
                            position: [left, bottom],
                            color: *color,
                        },
                        UIVertex {
                            position: [right, top],
                            color: *color,
                        },
                        UIVertex {
                            position: [right, bottom],
                            color: *color,
                        },
                    ]);
                }
                RenderCommands::Text {
                    text,
                    position,
                    color,
                    size,
                } => {
                    let (char_width, char_height) = self.font_manager.char_size();
                    let char_w = *size * char_width as f32;
                    let char_h = *size * char_height as f32;
                    let mut x = position.x;
                    let y = position.y;

                    for ch in text.chars() {
                        if ch == ' ' {
                            x += char_w;
                            continue;
                        }

                        if let Some(char_uv) = self.font_manager.get_char_uv(ch) {
                            let left = x;
                            let top = y;
                            let right = x + char_w;
                            let bottom = y + char_h;

                            ui_text_vertices.extend_from_slice(&[
                                UITexturedVertex {
                                    position: [left, top],
                                    tex_coords: [char_uv.u, char_uv.v],
                                    color: *color,
                                },
                                UITexturedVertex {
                                    position: [right, top],
                                    tex_coords: [char_uv.u + char_uv.u_width, char_uv.v],
                                    color: *color,
                                },
                                UITexturedVertex {
                                    position: [left, bottom],
                                    tex_coords: [char_uv.u, char_uv.v + char_uv.v_height],
                                    color: *color,
                                },
                                UITexturedVertex {
                                    position: [left, bottom],
                                    tex_coords: [char_uv.u, char_uv.v + char_uv.v_height],
                                    color: *color,
                                },
                                UITexturedVertex {
                                    position: [right, top],
                                    tex_coords: [char_uv.u + char_uv.u_width, char_uv.v],
                                    color: *color,
                                },
                                UITexturedVertex {
                                    position: [right, bottom],
                                    tex_coords: [
                                        char_uv.u + char_uv.u_width,
                                        char_uv.v + char_uv.v_height,
                                    ],
                                    color: *color,
                                },
                            ]);
                        }

                        x += char_w;
                    }
                }
                RenderCommands::UiRect {
                    rect,
                    color,
                    z_index,
                } => todo!(),
                RenderCommands::UiImage {
                    texture,
                    rect,
                    tint,
                    uv_rect,
                    z_index,
                } => todo!(),
                RenderCommands::UiText {
                    text,
                    rect,
                    color,
                    font_size,
                    z_index,
                } => todo!(),
            }
        }

        if all_instances.len() > self.instance_buffer_capacity {
            let new_capacity = (all_instances.len() * 3 / 2).max(1000);
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (std::mem::size_of::<InstanceData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_buffer_capacity = new_capacity;
        }

        if !all_instances.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&all_instances),
            );
        }

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        for cmd in &queue.commands {
            if let RenderCommands::Mesh3D {
                vertices,
                indices,
                model_matrix,
                color,
            } = cmd
            {
                let mvp_matrix = camera_matrix * model_matrix;
                let mesh_uniforms = MeshUniforms {
                    view_proj: mvp_matrix.to_cols_array_2d(),
                    view: glam::Mat4::IDENTITY.to_cols_array_2d(),
                    color: *color,
                    _padding: [0.0; 28],
                };
                let mesh_uniform_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Mesh Uniform Buffer"),
                            contents: bytemuck::bytes_of(&mesh_uniforms),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });

                let mesh_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.mesh_pipeline.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: mesh_uniform_buffer.as_entire_binding(),
                    }],
                    label: Some("Mesh Bind Group"),
                });

                let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Vertex Buffer"),
                    size: (vertices.len() * std::mem::size_of::<runa_render_api::Vertex3D>())
                        as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue
                    .write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(vertices));

                let index_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Index Buffer"),
                    size: (indices.len() * 4) as u64,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.queue
                    .write_buffer(&index_buffer, 0, bytemuck::cast_slice(indices));

                rpass.set_pipeline(&self.mesh_pipeline.pipeline);
                rpass.set_bind_group(0, &mesh_bind_group, &[]);
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
        }

        for (texture_key, instance_offset, instance_count) in batches {
            if !self.textures.contains_key(&texture_key) {
                let texture = self.textures_cache.get(&texture_key).unwrap();
                let gpu_tex = Arc::new(GpuTexture::from_asset(&self.device, &self.queue, texture));
                self.textures.insert(texture_key, gpu_tex);
            }
            let gpu_texture = self.textures.get(&texture_key).unwrap().clone();

            let bind_group = self.bind_group_cache.entry(texture_key).or_insert_with(|| {
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.nearest_sampler),
                        },
                    ],
                    label: Some("BindGroup"),
                })
            });

            rpass.set_pipeline(&self.sprite_pipeline.pipeline);
            rpass.set_bind_group(0, &*bind_group, &[]);
            rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.draw(
                0..6,
                instance_offset as u32..(instance_offset + instance_count) as u32,
            );
        }

        if !ui_vertices.is_empty() {
            let ui_vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Vertex Buffer"),
                size: (std::mem::size_of::<UIVertex>() * ui_vertices.len()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue
                .write_buffer(&ui_vertex_buffer, 0, bytemuck::cast_slice(&ui_vertices));

            rpass.set_pipeline(&self.ui_pipeline.pipeline);
            rpass.set_bind_group(0, self.ui_bind_group.as_ref().unwrap(), &[]);
            rpass.set_vertex_buffer(0, ui_vertex_buffer.slice(..));
            rpass.draw(0..ui_vertices.len() as u32, 0..1);
        }

        if !ui_text_vertices.is_empty() {
            let ui_text_vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UI Text Vertex Buffer"),
                size: (std::mem::size_of::<UITexturedVertex>() * ui_text_vertices.len()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.queue.write_buffer(
                &ui_text_vertex_buffer,
                0,
                bytemuck::cast_slice(&ui_text_vertices),
            );

            if let Some(atlas_tex) = self.font_manager.get_atlas_texture() {
                let text_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.ui_pipeline.textured_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.ui_uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&atlas_tex.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.nearest_sampler),
                        },
                    ],
                    label: Some("Text Bind Group"),
                });

                rpass.set_pipeline(&self.ui_pipeline.textured_pipeline);
                rpass.set_bind_group(0, &text_bind_group, &[]);
                rpass.set_vertex_buffer(0, ui_text_vertex_buffer.slice(..));
                rpass.draw(0..ui_text_vertices.len() as u32, 0..1);
            }
        }
    }

    fn build_render_target(
        device: &wgpu::Device,
        size: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> RenderTarget {
        let (width, height) = (size.0.max(1), size.1.max(1));
        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Color Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let (depth_texture, depth_view) = Self::create_depth_texture(device, (width, height));

        RenderTarget {
            _color_texture: color_texture,
            color_view,
            _depth_texture: depth_texture,
            depth_view,
            size: (width, height),
            _format: format,
        }
    }

    fn create_depth_texture(device: &wgpu::Device, size: (u32, u32)) -> (Texture, TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Depth Texture"),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
