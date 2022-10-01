use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

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
                wgpu::Features::MULTI_DRAW_INDIRECT
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
                    min_binding_size: wgpu::BufferSize::new(1),
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
            // ???
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
    
}
