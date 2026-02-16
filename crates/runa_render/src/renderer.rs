use std::{collections::HashMap, sync::Arc};

use crate::{
    font::FontManager, pipelines::MeshPipeline, pipelines::SpritePipeline,
    resources::texture::GpuTexture,
};
use glam::{Vec2, Vec3};
use runa_asset::TextureAsset;
use runa_render_api::{RenderCommands, RenderQueue};
use wgpu::util::DeviceExt;
use wgpu::{MemoryHints::Performance, Trace};
use wgpu::{Texture, TextureView};
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub view_proj: [[f32; 4]; 4],
    pub aspect: f32,
    pub _padding: [f32; 7],
}

pub struct Renderer<'window> {
    pub surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    sprite_pipeline: SpritePipeline,
    mesh_pipeline: MeshPipeline,

    vertex_buffer: wgpu::Buffer,
    max_vertices: usize,

    globals_buffer: wgpu::Buffer,

    textures: HashMap<usize, GpuTexture>,
    nearest_sampler: wgpu::Sampler,

    font_manager: FontManager,

    textures_cache: HashMap<usize, Arc<TextureAsset>>,
    bind_group_cache: HashMap<usize, wgpu::BindGroup>,

    depth_view: TextureView,
    depth_texture: Texture,
}

impl<'window> Renderer<'window> {
    pub async fn new_async(window: Arc<Window>) -> Self {
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

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            // ← КЛЮЧЕВОЕ ИЗМЕНЕНИЕ:
            present_mode: wgpu::PresentMode::Immediate, // вместо Fifo
            alpha_mode: surface.get_capabilities(&adapter).alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let sprite_pipeline = SpritePipeline::new(&device, surface_config.format);

        const MAX_SPRITES: usize = 1000;
        const VERTICES_PER_SPRITE: usize = 6;
        let max_vertices = MAX_SPRITES * VERTICES_PER_SPRITE;
        let vertex_buffer_size = (std::mem::size_of::<Vertex>() * max_vertices) as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            size: vertex_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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

        // 3D пайплайн
        let mesh_pipeline = MeshPipeline::new(
            &device,
            surface_config.format,
            wgpu::TextureFormat::Depth32Float,
        );

        Self {
            surface,
            surface_config,
            device,
            queue,
            sprite_pipeline,
            mesh_pipeline,
            vertex_buffer,
            max_vertices,
            globals_buffer,
            textures: HashMap::new(),
            nearest_sampler,
            font_manager,
            textures_cache: HashMap::new(),
            bind_group_cache: HashMap::new(),
            depth_view,
            depth_texture,
        }
    }

    pub fn new(window: Arc<Window>) -> Self {
        pollster::block_on(Self::new_async(window))
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);

        // Пересоздаём глубинный буфер
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
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
        self.depth_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn draw(&mut self, queue: &RenderQueue, camera_matrix: glam::Mat4, _virtual_size: Vec2) {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(_) => return,
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: None,
                ..Default::default()
            });

        // Обновляем глобальные данные
        self.queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::bytes_of(&Globals {
                view_proj: camera_matrix.to_cols_array_2d(),
                aspect: (_virtual_size.x / _virtual_size.y)
                    / (self.surface_config.width as f32 / self.surface_config.height as f32),
                _padding: [0.0; 7],
            }),
        );

        // ===== ШАГ 1: СБОРКА ВЕРШИН ПО ТЕКСТУРАМ =====
        let mut all_vertices = Vec::new();
        let mut draw_calls = Vec::new();

        for cmd in &queue.commands {
            match cmd {
                RenderCommands::Sprite {
                    texture,
                    position,
                    rotation,
                    scale,
                } => {
                    let tex_width = texture.width as f32;
                    let tex_height = texture.height as f32;

                    let world_width = (tex_width / 16.0) * scale.x;
                    let world_height = (tex_height / 16.0) * scale.y;

                    let sprite_vertices: [Vertex; 6] = [
                        Vertex {
                            position: [-world_width * 0.5, -world_height * 0.5, 0.0],
                            tex_coords: [0.0, 1.0],
                        },
                        Vertex {
                            position: [world_width * 0.5, -world_height * 0.5, 0.0],
                            tex_coords: [1.0, 1.0],
                        },
                        Vertex {
                            position: [-world_width * 0.5, world_height * 0.5, 0.0],
                            tex_coords: [0.0, 0.0],
                        },
                        Vertex {
                            position: [world_width * 0.5, -world_height * 0.5, 0.0],
                            tex_coords: [1.0, 1.0],
                        },
                        Vertex {
                            position: [world_width * 0.5, world_height * 0.5, 0.0],
                            tex_coords: [1.0, 0.0],
                        },
                        Vertex {
                            position: [-world_width * 0.5, world_height * 0.5, 0.0],
                            tex_coords: [0.0, 0.0],
                        },
                    ];

                    let transformed_vertices: Vec<Vertex> = sprite_vertices
                        .iter()
                        .map(|v| {
                            let pos_3d = Vec3::new(v.position[0], v.position[1], v.position[2]);
                            let scaled = pos_3d * scale;
                            let rotated = rotation * scaled;
                            let final_pos = Vec3::new(
                                position.x + rotated.x,
                                position.y + rotated.y,
                                position.z + rotated.z,
                            );

                            Vertex {
                                position: [final_pos.x, final_pos.y, final_pos.z],
                                tex_coords: v.tex_coords,
                            }
                        })
                        .collect();

                    let vertex_count = transformed_vertices.len();
                    let vertex_offset = all_vertices.len();
                    all_vertices.extend(transformed_vertices);

                    let key = Arc::as_ptr(&texture) as usize;
                    if !self.textures_cache.contains_key(&key) {
                        self.textures_cache.insert(key, texture.clone());
                    }
                    draw_calls.push((key, vertex_offset, vertex_count));
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
                    // Вычисляем углы текстуры с учётом флипа
                    let u0 = if *flip_x {
                        uv_rect[0] + uv_rect[2]
                    } else {
                        uv_rect[0]
                    };
                    let u1 = if *flip_x {
                        uv_rect[0]
                    } else {
                        uv_rect[0] + uv_rect[2]
                    };
                    let v_bottom = if *flip_y {
                        uv_rect[1]
                    } else {
                        uv_rect[1] + uv_rect[3]
                    };
                    let v_top = if *flip_y {
                        uv_rect[1] + uv_rect[3]
                    } else {
                        uv_rect[1]
                    };

                    let half_width = size.x as f32 * 0.5;
                    let half_height = size.y as f32 * 0.5;

                    let vertices: [Vertex; 6] = [
                        // Треугольник 1
                        Vertex {
                            position: [
                                position.x - half_width,
                                position.y + half_height,
                                position.z,
                            ],
                            tex_coords: [u0, v_top],
                        },
                        Vertex {
                            position: [
                                position.x + half_width,
                                position.y + half_height,
                                position.z,
                            ],
                            tex_coords: [u1, v_top],
                        },
                        Vertex {
                            position: [
                                position.x - half_width,
                                position.y - half_height,
                                position.z,
                            ],
                            tex_coords: [u0, v_bottom],
                        },
                        // Треугольник 2
                        Vertex {
                            position: [
                                position.x + half_width,
                                position.y - half_height,
                                position.z,
                            ],
                            tex_coords: [u1, v_bottom],
                        },
                        Vertex {
                            position: [
                                position.x + half_width,
                                position.y + half_height,
                                position.z,
                            ],
                            tex_coords: [u1, v_top],
                        },
                        Vertex {
                            position: [
                                position.x - half_width,
                                position.y - half_height,
                                position.z,
                            ],
                            tex_coords: [u0, v_bottom],
                        },
                    ];

                    let vertex_offset = all_vertices.len();
                    all_vertices.extend(vertices);
                    let key = Arc::as_ptr(&texture) as usize;
                    if !self.textures_cache.contains_key(&key) {
                        self.textures_cache.insert(key, texture.clone());
                    }
                    draw_calls.push((key, vertex_offset, vertices.len()));
                }
                RenderCommands::DebugRect {
                    position,
                    size,
                    color,
                } => {
                    // Создаем вершины для отладочного прямоугольника в 3D пространстве
                    let half_width = size.x * 0.5;
                    let half_height = size.y * 0.5;

                    let rect_vertices: [Vertex; 6] = [
                        // Треугольник 1
                        Vertex {
                            position: [
                                position.x - half_width,
                                position.y + half_height,
                                position.z,
                            ],
                            tex_coords: [0.0, 0.0],
                        },
                        Vertex {
                            position: [
                                position.x + half_width,
                                position.y + half_height,
                                position.z,
                            ],
                            tex_coords: [1.0, 0.0],
                        },
                        Vertex {
                            position: [
                                position.x - half_width,
                                position.y - half_height,
                                position.z,
                            ],
                            tex_coords: [0.0, 1.0],
                        },
                        // Треугольник 2
                        Vertex {
                            position: [
                                position.x + half_width,
                                position.y - half_height,
                                position.z,
                            ],
                            tex_coords: [1.0, 1.0],
                        },
                        Vertex {
                            position: [
                                position.x + half_width,
                                position.y + half_height,
                                position.z,
                            ],
                            tex_coords: [1.0, 0.0],
                        },
                        Vertex {
                            position: [
                                position.x - half_width,
                                position.y - half_height,
                                position.z,
                            ],
                            tex_coords: [0.0, 1.0],
                        },
                    ];

                    let vertex_offset = all_vertices.len();
                    all_vertices.extend(rect_vertices);

                    // Для отладочного рендеринга используем специальный вызов отрисовки
                    // Временно пропускаем этот вызов, так как у нас нет специальной шейдерной программы для отладочных примитивов
                    // draw_calls.push((0, vertex_offset, rect_vertices.len())); // 0 как специальный ключ для отладочных примитивов
                }
                RenderCommands::Text { .. } => {
                    // TODO: реализовать текст
                }
            }
        }

        if all_vertices.len() > self.max_vertices {
            eprintln!("Too many vertices! Max: {}", self.max_vertices);
            return;
        }

        if !all_vertices.is_empty() {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&all_vertices));
        }

        // ===== ШАГ 2: РЕНДЕРИНГ БАТЧЕЙ =====
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sprite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            ..Default::default()
        });

        let mut last_texture_key: Option<usize> = None;

        for (texture_key, vertex_offset, vertex_count) in draw_calls {
            // Получаем/создаём текстуру
            let gpu_texture = self.textures.entry(texture_key).or_insert_with(|| {
                let texture = self.textures_cache.get(&texture_key).unwrap();
                GpuTexture::from_asset(&self.device, &self.queue, texture)
            });

            // Кэшируем бинд-группу (создаётся ОДИН раз на текстуру)
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
                    label: Some("Cached BindGroup"),
                })
            });

            // ← ЯВНОЕ ПРЕОБРАЗОВАНИЕ для совместимости с set_bind_group
            let bind_group_ref: &wgpu::BindGroup = &*bind_group;

            // Оптимизация: избегаем лишних вызовов set_bind_group
            if last_texture_key != Some(texture_key) {
                rpass.set_pipeline(&self.sprite_pipeline.pipeline);
                rpass.set_bind_group(0, bind_group_ref, &[]);
                last_texture_key = Some(texture_key);
            }

            let vertex_offset_bytes = (vertex_offset * std::mem::size_of::<Vertex>()) as u64;
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(vertex_offset_bytes..));
            rpass.draw(0..vertex_count as u32, 0..1);
        }

        drop(rpass);
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}
