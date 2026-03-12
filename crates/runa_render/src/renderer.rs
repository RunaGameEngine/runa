//use std::time::Instant;
use std::{collections::HashMap, sync::Arc};

use crate::{
    font::FontManager, pipelines::MeshPipeline, pipelines::SpritePipeline,
    resources::texture::GpuTexture,
};
use glam::Vec2;
use runa_asset::TextureAsset;
use runa_render_api::{RenderCommands, RenderQueue};
use wgpu::util::DeviceExt;
use wgpu::{MemoryHints::Performance, Trace};
use wgpu::{Texture, TextureView};
use winit::window::Window;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub position: [f32; 3],  // x, y, z
    pub rotation: f32,       // radians
    pub scale: [f32; 3],     // x, y, z
    pub uv_offset: [f32; 2], // left-bottom UV
    pub uv_size: [f32; 2],   // size UV-quad
    pub flip: u32,           // 0 = flip_x, 1 = flip_y
    pub _pad: f32,
}

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

    quad_buffer: wgpu::Buffer,     // базовый квад (6 вершин, не меняется)
    instance_buffer: wgpu::Buffer, // буфер инстансов
    max_instances: usize,
}

impl<'window> Renderer<'window> {
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

        let surface_config: wgpu::SurfaceConfiguration;
        if vsync {
            surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.get_capabilities(&adapter).formats[0],
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
                format: surface.get_capabilities(&adapter).formats[0],
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::AutoNoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
        }

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

        // Инстанс-буфер (максимум 1000 спрайтов)
        const MAX_INSTANCES: usize = 1000;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceData>() * MAX_INSTANCES) as u64,
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
            quad_buffer,
            instance_buffer,
            max_instances: MAX_INSTANCES,
        }
    }

    pub fn new(window: Arc<Window>, vsync: bool) -> Self {
        pollster::block_on(Self::new_async(window, vsync))
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

        //let t0 = Instant::now();

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
        let mut all_instances = Vec::new();
        let mut batches = Vec::new();

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
                    let world_scale_x = scale.x * (tex_width / 16.0);
                    let world_scale_y = scale.y * (tex_height / 16.0);

                    let instance = InstanceData {
                        position: [position.x, position.y, position.z],
                        rotation: rotation.z,
                        scale: [world_scale_x, world_scale_y, 1.0],

                        uv_offset: [0.0, 0.0], // начинаем с левого-нижнего угла
                        uv_size: [1.0, 1.0],   // используем полный размер
                        flip: 0,               // без флипа
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
                    uv_rect, // [x, y, w, h] в 0..1
                    flip_x,
                    flip_y,
                    color: _,
                } => {
                    let instance = InstanceData {
                        position: [position.x, position.y, position.z],
                        rotation: 0.0, // тайлы обычно не вращаются
                        scale: [size.x as f32, size.y as f32, 1.0],

                        // UV-данные для тайла
                        uv_offset: [uv_rect[0], uv_rect[1]],
                        uv_size: [uv_rect[2], uv_rect[3]],
                        flip: ((*flip_x) as u32) | (((*flip_y) as u32) << 1),
                        _pad: 0.0,
                    };

                    let key = Arc::as_ptr(texture) as usize;
                    if !self.textures_cache.contains_key(&key) {
                        self.textures_cache.insert(key, texture.clone());
                    }

                    let offset = all_instances.len(); // тот же вектор, что и для спрайтов!
                    all_instances.push(instance);
                    batches.push((key, offset, 1)); // 1 инстанс = 1 тайл
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

        // Записываем ВСЕ инстансы ОДИН раз
        if !all_instances.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&all_instances[..all_instances.len().min(self.max_instances)]),
            );
        }

        //let t2 = Instant::now();

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

        // Устанавливаем бинд-группу (берём первую текстуру из кэша)
        // ===== ШАГ 2: РЕНДЕРИНГ СПРАЙТОВ (ИНСТАНСИНГ) =====
        for (texture_key, instance_offset, instance_count) in batches {
            let gpu_texture = self.textures.entry(texture_key).or_insert_with(|| {
                let texture = self.textures_cache.get(&texture_key).unwrap();
                GpuTexture::from_asset(&self.device, &self.queue, texture)
            });

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
            // ← КЛЮЧЕВОЕ: рендерим инстансы с правильным смещением
            rpass.draw(
                0..6,
                instance_offset as u32..(instance_offset + instance_count) as u32,
            );
        }

        //let t3 = Instant::now();

        // println!(
        //     "Prep: {:.2}ms | Write (cmd): {:.2}ms | Render: {:.2}ms",
        //     (t1 - t0).as_secs_f32() * 1000.,
        //     (t2 - t1).as_secs_f32() * 1000.,
        //     (t3 - t2).as_secs_f32() * 1000.
        // );

        drop(rpass);
        self.queue.submit(Some(encoder.finish()));
        let _ = self.device.poll(wgpu::PollType::Poll);
        surface_texture.present();
    }
}
