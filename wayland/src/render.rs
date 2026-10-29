use bytemuck::{Pod, Zeroable};
use snowhush::simulation::{COLUMN_WIDTH, SnowSimulation, Sprite};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Uniforms {
    pub view_size: [f32; 2],
    pub scale: f32,
    pub time: f32,
    pub column_width: f32,
    pub column_count: i32,
    pub pile_top: f32,
    pub occluder_count: i32,
    pub origin: [f32; 2],
    pub pile_bottom: f32,
    pub height_offset: i32,
    pub light: [f32; 4],
    pub shadow_tint: [f32; 4],
    pub occluders: [[f32; 4]; 16],
    pub motion_blur: [f32; 4],
}

#[derive(Clone, Copy)]
pub struct Ambient {
    pub light: [f32; 3],
    pub shadow_tint: [f32; 4],
}

impl Default for Ambient {
    fn default() -> Self {
        Self {
            light: [1.0, 1.0, 1.0],
            shadow_tint: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub sprite_pipeline: wgpu::RenderPipeline,
    pub pile_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_stride: usize,
    pub height_buffer: wgpu::Buffer,
    pub height_capacity: usize,
    pub sprite_buffer: wgpu::Buffer,
    pub sprite_capacity: usize,
    pub bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find suitable GPU adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Snowhush Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let alpha_mode = caps
            .alpha_modes
            .iter()
            .copied()
            .find(|&m| m == wgpu::CompositeAlphaMode::PreMultiplied)
            .unwrap_or(caps.alpha_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Snowhush Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Snowhush Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(std::mem::size_of::<Uniforms>() as u64).unwrap(),
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Snowhush Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let blend_state = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let sprite_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Sprite>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 20,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 24,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 28,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 32,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 44,
                    shader_location: 7,
                },
            ],
        };

        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("sprite_vertex"),
                buffers: &[sprite_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("sprite_fragment"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let pile_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pile Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("pile_vertex"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("pile_fragment"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let align = device.limits().min_uniform_buffer_offset_alignment as usize;
        let uniform_size = std::mem::size_of::<Uniforms>();
        let uniform_stride = uniform_size.div_ceil(align) * align;
        let max_uniform_slots = 256;
        let uniform_buffer_size = (uniform_stride * max_uniform_slots) as u64;

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let height_capacity = 4096;
        let height_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Height Buffer"),
            size: (height_capacity * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sprite_capacity = 4096;
        let sprite_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Buffer"),
            size: (sprite_capacity * std::mem::size_of::<Sprite>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(uniform_size as u64).unwrap(),
                        ),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: height_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            device,
            queue,
            surface,
            config,
            sprite_pipeline,
            pile_pipeline,
            bind_group_layout,
            uniform_buffer,
            uniform_stride,
            height_buffer,
            height_capacity,
            sprite_buffer,
            sprite_capacity,
            bind_group,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 && (self.config.width != width || self.config.height != height) {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn draw(&mut self, simulation: &SnowSimulation, ambient: Ambient, time: f32) -> Result<(), wgpu::SurfaceError> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Timeout) => {
                return Ok(());
            }
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Prepare sprites
        let sprite_count = simulation.sprite_count();
        if sprite_count > self.sprite_capacity {
            self.sprite_capacity = (sprite_count + 1024).next_power_of_two();
            self.sprite_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sprite Buffer"),
                size: (self.sprite_capacity * std::mem::size_of::<Sprite>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let mut sprite_data = vec![Sprite::default(); sprite_count];
        let written_sprites = simulation.write_sprites(&mut sprite_data);
        if written_sprites > 0 {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    sprite_data.as_ptr() as *const u8,
                    written_sprites * std::mem::size_of::<Sprite>(),
                )
            };
            self.queue.write_buffer(&self.sprite_buffer, 0, bytes);
        }

        // Prepare heightfield layers
        let col_count = simulation.column_count();
        if col_count > self.height_capacity {
            self.height_capacity = (col_count + 1024).next_power_of_two();
            self.height_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Height Buffer"),
                size: (self.height_capacity * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let uniform_size = std::mem::size_of::<Uniforms>();
            self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Main Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.uniform_buffer,
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(uniform_size as u64).unwrap(),
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.height_buffer.as_entire_binding(),
                    },
                ],
            });
        }

        let mut height_data = vec![0.0f32; self.height_capacity];
        let piles = simulation.write_layers(&mut height_data);
        if !piles.is_empty() {
            let bytes = bytemuck::cast_slice(&height_data);
            self.queue.write_buffer(&self.height_buffer, 0, bytes);
        }

        // Pack uniforms for all piles + falling sprites
        let total_slots = piles.len() + 1;
        let mut uniform_bytes = vec![0u8; total_slots * self.uniform_stride];

        for (i, pile) in piles.iter().enumerate() {
            let mut uniforms = Uniforms {
                view_size: [simulation.width(), simulation.height()],
                scale: 1.0,
                time,
                column_width: COLUMN_WIDTH,
                column_count: pile.count as i32,
                pile_top: pile.top + 2.0,
                occluder_count: pile.occluders.len().min(16) as i32,
                origin: [pile.origin.x, pile.origin.y],
                pile_bottom: pile.bottom,
                height_offset: pile.offset as i32,
                light: [ambient.light[0], ambient.light[1], ambient.light[2], 1.0],
                shadow_tint: ambient.shadow_tint,
                occluders: [[0.0; 4]; 16],
                motion_blur: [0.0; 4],
            };
            for (j, o) in pile.occluders.iter().take(16).enumerate() {
                uniforms.occluders[j] = *o;
            }
            let slice = bytemuck::bytes_of(&uniforms);
            let start = i * self.uniform_stride;
            uniform_bytes[start..start + slice.len()].copy_from_slice(slice);
        }

        if written_sprites > 0 {
            let sprite_uniforms = Uniforms {
                view_size: [simulation.width(), simulation.height()],
                scale: 1.0,
                time,
                column_width: COLUMN_WIDTH,
                column_count: 0,
                pile_top: 0.0,
                occluder_count: 0,
                origin: [0.0; 2],
                pile_bottom: 0.0,
                height_offset: 0,
                light: [ambient.light[0], ambient.light[1], ambient.light[2], 1.0],
                shadow_tint: ambient.shadow_tint,
                occluders: [[0.0; 4]; 16],
                motion_blur: [simulation.motion_blur, 0.0, 0.0, 0.0],
            };
            let slice = bytemuck::bytes_of(&sprite_uniforms);
            let start = piles.len() * self.uniform_stride;
            uniform_bytes[start..start + slice.len()].copy_from_slice(slice);
        }

        self.queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Snow Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw snow piles on window edges and ground
            if !piles.is_empty() {
                rpass.set_pipeline(&self.pile_pipeline);
                for i in 0..piles.len() {
                    let offset = (i * self.uniform_stride) as u32;
                    rpass.set_bind_group(0, &self.bind_group, &[offset]);
                    rpass.draw(0..4, 0..1);
                }
            }

            // Draw falling snow particles and motes
            if written_sprites > 0 {
                rpass.set_pipeline(&self.sprite_pipeline);
                let offset = (piles.len() * self.uniform_stride) as u32;
                rpass.set_bind_group(0, &self.bind_group, &[offset]);
                rpass.set_vertex_buffer(0, self.sprite_buffer.slice(..));
                rpass.draw(0..4, 0..written_sprites as u32);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
