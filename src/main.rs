use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
struct Uniform {
    modelview: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
struct Vertex {
    pos: [f32; 3],
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
struct Object {
    color: [f32; 3],
    pos: [f32; 3],
    transform: [[f32; 3]; 3],
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    base_index: u32,
    vertex_offset: u32,
    base_instance: u32,
}

fn generate_mesh() -> (Object, Vec<Vertex>, Vec<u16>) {
    (
        Object {
            color: [1.0, 0.5, 0.0],
            pos: [0.0, 0.0, 0.5],
            transform: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        },
        vec![
            Vertex {
                pos: [-1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [0.0, 1.0, 0.0],
            },
        ],
        vec![0, 1, 2],
    )
}

fn main() {
    // enable multi draw if needed
    // TODO: proper args
    let multidraw_allowed = std::env::args().find(|x| *x == "--multidraw").is_some();

    println!("multidraw: {}", multidraw_allowed);

    // winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("wgpu multidraw test")
        .build(&event_loop)
        .unwrap();

    // wgpu
    let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
    let instance = wgpu::Instance::new(backends);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .enumerate_adapters(backends)
        .find(|x| {
            std::env::args()
                .find(|y| x.get_info().name.contains(y))
                .is_some()
                && x.is_surface_supported(&surface)
        })
        .unwrap_or(
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }))
            .unwrap(),
        );

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            features: if multidraw_allowed {
                wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::INDIRECT_FIRST_INSTANCE
            } else {
                wgpu::Features::empty()
            },
        },
        None,
    ))
    .unwrap();

    // shader
    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    // uniform bind group
    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<Uniform>() as wgpu::BufferAddress
                    ),
                },
            }],
        });

    // mesh layout
    let vertex_layout = wgpu::VertexBufferLayout {
        array_stride: 3 * 4,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
        ],
    };

    // per mesh (this is done in instance rate vertex buffers)
    let object_layout = wgpu::VertexBufferLayout {
        array_stride: 3 * 4,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            1 => Float32x3,
            2 => Float32x3,
            3 => Float32x3,
            4 => Float32x3,
            5 => Float32x3,
        ],
    };

    // set up pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    // pipeline
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vertex",
            buffers: &[vertex_layout, object_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fragment",
            targets: &[Some(wgpu::TextureFormat::Bgra8UnormSrgb.into())],
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // buffers
    let mut indirect_draw = Vec::new();
    let mut vertices = Vec::new();
    let mut objects = Vec::new();
    let mut indices = Vec::new();

    // generate some objects
    // TODO: tunable size
    for i in 0..1024 {
        // generate the mesh
        let (object, verts, tris) = generate_mesh();

        // append
        indirect_draw.push(DrawIndirect {
            vertex_count: verts.len() as u32,
            instance_count: 1,
            base_index: indices.len() as u32,
            vertex_offset: vertices.len() as u32,
            base_instance: objects.len() as u32,
        });

        // append
        vertices.extend(verts);
        indices.extend(tris);
        objects.push(object);
    }

    // create uniform
    let mut uniforms = Uniform {
        modelview: [[0.0; 4]; 4],
    };

    // make wgpu buffers
    let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniforms"),
        usage: wgpu::BufferUsages::UNIFORM,
        contents: bytemuck::cast_slice(&[uniforms]),
    });

    let indirect_draw_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indirect draw"),
        usage: wgpu::BufferUsages::INDIRECT,
        contents: bytemuck::cast_slice(&indirect_draw),
    });

    let vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertices"),
        usage: wgpu::BufferUsages::VERTEX,
        contents: bytemuck::cast_slice(&vertices),
    });

    let indices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indices"),
        usage: wgpu::BufferUsages::INDEX,
        contents: bytemuck::cast_slice(&indices),
    });

    let objects_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("objects"),
        usage: wgpu::BufferUsages::VERTEX,
        contents: bytemuck::cast_slice(&objects),
    });
    
    // bind group
    let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniforms bind group"),
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }
        ],
    });
    
    // config surface
    let mut surface_config = wgpu::SurfaceConfiguration {
        width: window.inner_size().width,
        height: window.inner_size().height,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        present_mode: wgpu::PresentMode::AutoNoVsync,
    };
    
    surface.configure(&device, &surface_config);

    window.request_redraw();
    
    // start the event loop
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => control_flow.set_exit(),
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                surface_config.width = window.inner_size().width;
                surface_config.height = window.inner_size().height;
                surface.configure(&device, &surface_config);
            }
            Event::RedrawRequested(_) => {
            
                // get the surface
                let texture = surface.get_current_texture().expect("Failed to get texture");
                let view = texture.texture.create_view(&Default::default());
                
                // render
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                
                // render pass
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &uniforms_bind_group, &[]);
                pass.set_vertex_buffer(0, vertices_buffer.slice(..));
                pass.set_vertex_buffer(1, objects_buffer.slice(..));
                pass.set_index_buffer(indices_buffer.slice(..), wgpu::IndexFormat::Uint16);
                
                // draw
                if multidraw_allowed {
                    pass.multi_draw_indexed_indirect(&indirect_draw_buf, 0, 1024);
                } else {
                    for i in 0..1024 {
                        pass.draw_indexed_indirect(&indirect_draw_buf, i * std::mem::size_of::<DrawIndirect>() as u64);
                    }
                }
                
                // stop
                std::mem::drop(pass);
                
                // submit
                queue.submit(Some(encoder.finish()));
                
                // present
                texture.present();
                
                // and again
                window.request_redraw();
            }
            _ => (),
        }
    });
}
