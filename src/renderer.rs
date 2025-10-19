use rand::Rng;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    wgt::CommandEncoderDescriptor,
    *,
};

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Debug)]
pub struct Renderer {
    width: u32,
    height: u32,
}

struct App {
    window: Option<Window>,
    renderer_state: Option<RendererState>,
    width: u32,
    height: u32,
}

struct RendererState {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    texture1: Texture,
    texture2: Texture,
    compute_shader_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    current_texture: bool, // false = texture1 is current, true = texture2 is current
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    fn create_texture(device: &Device, width: u32, height: u32, label: &str) -> Texture {
        let texture_desc = TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Uint,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING
                | TextureUsages::COPY_DST,
            view_formats: &[TextureFormat::R32Uint],
        };
        device.create_texture(&texture_desc)
    }

    fn init_texture(device: &Device, queue: &Queue, texture: &mut Texture, data: &[u32]) {
        let bytes_per_pixel = 4; // e.g. RGBA8 format
        let bytes_per_row = ((bytes_per_pixel * texture.width() + 255) / 256) * 256;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Temp Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Init Encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(texture.height()),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: texture.width(),
                height: texture.height(),
                depth_or_array_layers: 1,
            },
        );

        queue.submit([encoder.finish()]);
    }

    async fn init_wgpu(window: &Window, width: u32, height: u32) -> RendererState {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(window).unwrap())
        }
        .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to request adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .expect("Failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut texture1 = Self::create_texture(&device, width, height, "Texture 1");
        let texture2 = Self::create_texture(&device, width, height, "Texture 2");

        let compute_shader_module = device.create_shader_module(include_wgsl!("conways.wgsl"));
        let compute_shader_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: None,
            module: &compute_shader_module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let mut r = rand::rng();

        let data = (0..(width * height))
            .into_iter()
            .map(|_| r.random_bool(0.5) as u32)
            .collect::<Vec<_>>();

        Self::init_texture(&device, &queue, &mut texture1, data.as_slice());

        // Create render pipeline for displaying the texture
        let display_shader_module = device.create_shader_module(include_wgsl!("display.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Display Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Uint,
                        },
                        count: None,
                    }],
                }),
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &display_shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &display_shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        RendererState {
            surface,
            device,
            queue,
            texture1,
            texture2,
            compute_shader_pipeline,
            render_pipeline,
            current_texture: false,
        }
    }

    fn redraw(&mut self, renderer_state: &mut RendererState) {
        let (src_texture, dst_texture) = if renderer_state.current_texture {
            (&renderer_state.texture2, &renderer_state.texture1)
        } else {
            (&renderer_state.texture1, &renderer_state.texture2)
        };

        let src_view = src_texture.create_view(&Default::default());
        let dst_view = dst_texture.create_view(&Default::default());

        let dimensions_data = [self.width, self.height];
        let dimensions_buffer = renderer_state
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Dimensions Buffer"),
                contents: bytemuck::cast_slice(&dimensions_data),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let bind_group = renderer_state
            .device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Compute Bind Group"),
                layout: &renderer_state
                    .compute_shader_pipeline
                    .get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: dimensions_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });

        let output = renderer_state
            .surface
            .get_current_texture()
            .expect("Failed to get current texture");
        let output_view = output.texture.create_view(&Default::default());

        let mut encoder = renderer_state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&renderer_state.compute_shader_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let wg_x = self.width.div_ceil(8);
            let wg_y = self.height.div_ceil(8);
            compute_pass.dispatch_workgroups(wg_x, wg_y, 1);
        }

        // Render pass - display the computed texture
        {
            let current_tex = if renderer_state.current_texture {
                &renderer_state.texture1
            } else {
                &renderer_state.texture2
            };

            let current_tex_view = current_tex.create_view(&TextureViewDescriptor {
                label: Some("Current Texture View"),
                format: Some(TextureFormat::R32Uint),
                dimension: Some(TextureViewDimension::D2),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
                usage: Some(TextureUsages::TEXTURE_BINDING),
            });

            let display_bind_group =
                renderer_state
                    .device
                    .create_bind_group(&BindGroupDescriptor {
                        label: Some("Display Bind Group"),
                        layout: &renderer_state.render_pipeline.get_bind_group_layout(0),
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&current_tex_view),
                        }],
                    });

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&renderer_state.render_pipeline);
            render_pass.set_bind_group(0, &display_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Full-screen triangle
        }

        renderer_state.queue.submit([encoder.finish()]);
        output.present();

        // Swap textures for next frame
        renderer_state.current_texture = !renderer_state.current_texture;
    }

    pub fn run(self) {
        env_logger::init();

        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App {
            window: None,
            renderer_state: None,
            width: self.width,
            height: self.height,
        };

        event_loop
            .run_app(&mut app)
            .expect("Failed to run event loop");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Conway's Game of Life")
                .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            let renderer_state =
                pollster::block_on(Renderer::init_wgpu(&window, self.width, self.height));

            self.window = Some(window);
            self.renderer_state = Some(renderer_state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(renderer_state)) =
                    (&self.window, &mut self.renderer_state)
                {
                    let mut renderer = Renderer::new(self.width, self.height);
                    renderer.redraw(renderer_state);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            // Add a small delay to control animation speed
            std::thread::sleep(std::time::Duration::from_millis(100));
            window.request_redraw();
        }
    }
}
