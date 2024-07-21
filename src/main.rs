use std::{borrow::Cow, iter};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

mod logic;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 2],
    _color: [f32; 4],
}

fn create_bundle(
    device: &wgpu::Device,
    swapchain_format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    sample_count: u32,
    vertex_buffer: &wgpu::Buffer,
    vertex_count: u32,
) -> wgpu::RenderBundle {
    log::info!("sample_count: {}", sample_count);

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    });

    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[Some(swapchain_format.into())],
        depth_stencil: None,
        sample_count,
        multiview: None,
    });
    encoder.set_pipeline(&pipeline);
    encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
    encoder.draw(0..vertex_count, 0..1);
    encoder.finish(&wgpu::RenderBundleDescriptor {
        label: Some("main"),
    })
}

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    swapchain_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: swapchain_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };

    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let surface = instance.create_surface(&window).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .unwrap();

    surface.configure(&device, &config);

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let sample_flags = adapter.get_texture_format_features(swapchain_format).flags;

    let sample_count = {
        if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X16) {
            16
        } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
            8
        } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
            4
        } else if sample_flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
            2
        } else {
            1
        }
    };

    let mut vertex_data = vec![];
    let max = 50;
    for i in 0..max {
        let percent = i as f32 / max as f32;
        let (sin, cos) = (percent * 2.0 * std::f32::consts::PI).sin_cos();
        vertex_data.push(Vertex {
            _pos: [0.0, 0.0],
            _color: [1.0, -sin, cos, 1.0],
        });
        vertex_data.push(Vertex {
            _pos: [cos, sin],
            _color: [sin, -cos, 1.0, 1.0],
        });
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let vertex_count = vertex_data.len() as u32;

    let bundle = create_bundle(
        &device,
        swapchain_format,
        &shader,
        &pipeline_layout,
        sample_count,
        &vertex_buffer,
        vertex_count,
    );

    let window = &window;
    event_loop
        .run(move |event, target| {
            let _ = (&instance, &adapter, &shader, &pipeline_layout);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(new_size) => {
                        config.width = new_size.width.max(1);
                        config.height = new_size.height.max(1);

                        surface.configure(&device, &config);
                        window.request_redraw();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::RedrawRequested => {
                        let frame = surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let view = if sample_count == 1 {
                            frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default())
                        } else {
                            create_multisampled_framebuffer(
                                &device,
                                &config,
                                swapchain_format,
                                sample_count,
                            )
                        };

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        {
                            let rpass_color_attachment = wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: if sample_count == 1 {
                                        wgpu::StoreOp::Store
                                    } else {
                                        wgpu::StoreOp::Discard
                                    },
                                },
                            };
                            encoder
                                .begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: None,
                                    color_attachments: &[Some(rpass_color_attachment)],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                })
                                .execute_bundles(iter::once(&bundle));
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                    }
                    _ => {}
                }
            }
        })
        .unwrap();
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let builder = winit::window::WindowBuilder::new();

    let window = builder.build(&event_loop).unwrap();

    env_logger::init();
    pollster::block_on(run(event_loop, window));
}
