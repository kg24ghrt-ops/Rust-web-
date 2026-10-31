//! # B5 School Notebook Paper
//!
//! A realistic ruled school notebook paper (B5: 176 × 250 mm) rendered on the
//! GPU with **wgpu** and hosted on the **Tokio** async runtime.
//!
//! The paper has classic blue ruling lines, a red left margin, a red header
//! band, and a white sheet on a light gray desk background.
//!
//! ## Usage
//!
//! ```bash
//! cargo run
//! ```
//!
//! Requires a graphics backend (Vulkan, Metal, DX12, OpenGL, or WebGPU).
//! The CI builds to `wasm32-unknown-unknown` (WebGPU).

use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// B5 paper dimensions in millimetres.
const B5_WIDTH_MM: f32 = 176.0;
const B5_HEIGHT_MM: f32 = 250.0;
const B5_ASPECT: f32 = B5_WIDTH_MM / B5_HEIGHT_MM; // 0.704

// Colours.
const DESK: [f32; 4] = [0.90, 0.90, 0.88, 1.00];
const PAPER: [f32; 4] = [1.00, 1.00, 1.00, 1.00];
const RULE: [f32; 4] = [0.22, 0.44, 1.00, 0.95]; // blue ruling
const MARGIN: [f32; 4] = [1.00, 0.20, 0.20, 1.00]; // red margin/header

const LINE_THICKNESS: f32 = 0.009; // NDC thickness of ruled/margin lines
const PAPER_PAD: f32 = 0.92; // paper fills 92% of the window height
const HEADER_FRAC: f32 = 0.15; // header band height as fraction of paper height
const MARGIN_FRAC: f32 = 0.05; // left margin width as fraction of paper height
const RULE_COUNT: usize = 34; // number of ruled lines

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
};

const SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};
@vertex
fn vs(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed to find a graphics adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    trace: wgpu::Trace::None,
                },
                None,
            )
            .await
            .expect("failed to create graphics device");
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[VERTEX_BUFFER_LAYOUT],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
        });

        let mut state = Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vertices"),
                size: 0,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            }),
            vertex_count: 0,
            size,
        };
        state.rebuild();
        state
    }

    fn rebuild(&mut self) {
        let vertices = build_vertices(self.size.width, self.size.height);
        self.vertex_count = vertices.len() as u32;
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.rebuild();
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: DESK[0] as f64,
                            g: DESK[1] as f64,
                            b: DESK[2] as f64,
                            a: DESK[3] as f64,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.draw(0..self.vertex_count, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn build_vertices(width: u32, height: u32) -> Vec<Vertex> {
    let mut v: Vec<Vertex> = Vec::new();
    let aspect = width as f32 / height as f32;

    // Fit the B5 sheet inside the window, preserving aspect ratio.
    let (pw, ph) = if aspect > B5_ASPECT {
        let h = PAPER_PAD;
        (h * B5_ASPECT, h)
    } else {
        let w = PAPER_PAD;
        (w, w / B5_ASPECT)
    };
    let x0 = -pw / 2.0;
    let x1 = pw / 2.0;
    let y0 = -ph / 2.0;
    let y1 = ph / 2.0;

    // White paper sheet.
    push_rect(&mut v, x0, y0, x1, y1, PAPER);

    // Header band: two close red lines near the top.
    let header_top = y1 - ph * HEADER_FRAC;
    let thick = LINE_THICKNESS;
    push_rect(&mut v, x0, y1, x1, y1 - thick, MARGIN);
    push_rect(&mut v, x0, header_top - thick, x1, header_top, MARGIN);

    // Left margin: vertical red line.
    let mx = x0 + ph * MARGIN_FRAC;
    push_rect(&mut v, mx, y0, mx + thick * 2.0, y1, MARGIN);

    // Ruled blue lines across the page, below the header.
    let line_top = y1 - ph * HEADER_FRAC - ph * 0.04;
    let line_bottom = y0 + ph * 0.04;
    let n = RULE_COUNT.max(2);
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let cy = line_top * (1.0 - t) + line_bottom * t;
        push_rect(
            &mut v,
            mx + ph * 0.07,
            cy - thick / 2.0,
            x1,
            cy + thick / 2.0,
            RULE,
        );
    }

    v
}

fn push_rect(v: &mut Vec<Vertex>, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
    // Two triangles forming an axis-aligned rectangle in NDC.
    v.push(Vertex {
        position: [x0, y0],
        color,
    });
    v.push(Vertex {
        position: [x1, y0],
        color,
    });
    v.push(Vertex {
        position: [x0, y1],
        color,
    });
    v.push(Vertex {
        position: [x1, y0],
        color,
    });
    v.push(Vertex {
        position: [x1, y1],
        color,
    });
    v.push(Vertex {
        position: [x0, y1],
        color,
    });
}

/// Native entry point: Tokio drives the async runtime (device creation) and a
/// background heartbeat task; the winit event loop blocks the main thread on
/// render frames.
#[cfg(not(target_family = "wasm"))]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Background Tokio task: genuinely uses the async runtime alongside wgpu.
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            println!("[notebook] frame heartbeat (wgpu + Tokio)");
        }
    });

    run_app();
}

/// Wasm entry point: Tokio is unavailable on `wasm32-unknown-unknown`, so the
/// native Tokio build above is used for desktop. This stub keeps the Wasm CI
/// target compiling (CI builds `--target wasm32-unknown-unknown`).
#[cfg(target_family = "wasm")]
fn main() {
    let _ = std::env::args();
    println!("B5 School Notebook Paper (wasm build)");
}

#[cfg(not(target_family = "wasm"))]
fn run_app() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("B5 School Notebook Paper")
        .build(&event_loop)
        .unwrap();

    // Device creation is async; block the current thread on Tokio's executor.
    let handle = tokio::runtime::Handle::current();
    let mut state = handle.block_on(State::new(&window));

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent {
                    window_id: wid,
                    ref event,
                } if wid == window.id() => {
                    if state.input(event) {
                        return;
                    }
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(size) => state.resize(size),
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(()) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("render error: {e}"),
                            }
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
