use vulkano::{
    instance::{
        Instance, PhysicalDevice
    },
    device::{
        Device, DeviceExtensions
    },
    buffer::{
        BufferUsage, CpuAccessibleBuffer
    },
    framebuffer::{
        Framebuffer, Subpass, FramebufferAbstract
    },
    pipeline::{
        viewport::Viewport, GraphicsPipeline
    },
    command_buffer::{
        AutoCommandBufferBuilder, DynamicState
    },
    swapchain::{
        Swapchain, SurfaceTransform, PresentMode, acquire_next_image
    },
    sync::{
        now, GpuFuture
    }
};

use vulkano_win::VkSurfaceBuild;

use winit::{
    EventsLoop,
    Event,
    WindowEvent,
    ControlFlow,
    WindowBuilder
};

use std::sync::Arc;

mod imagegen;

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2]
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r#"
        #version 450

        layout(location = 0) in vec2 position;
        layout(location = 1) in vec2 tex_coord;
        //layout(location = 2) in int paletteIndex;

        layout(location = 0) out vec2 texCoordOut;
        //layout(location = 1) out int paletteIndexOut;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
            texCoordOut = tex_coord;
            //paletteIndexOut = paletteIndex;
        }"#
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: r#"
        #version 450

        layout(location = 0) in vec2 texCoord;
        //layout(location = 1) in int paletteIndex;

        layout(location = 0) out vec4 outColor;

        void main() {
            //int texel = texture(texSampler, texCoord);
            //float pixel = palette[texel];
            //outColor = vec4(vec3(pixel), 1.0);
            outColor = vec4(texCoord.x, texCoord.y, 0.0, 0.0);
        }"#
    }
}

vulkano::impl_vertex!(Vertex, position, tex_coord);

fn main() {
    // Make instance with window extensions.
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("Failed to create vulkan instance")
    };

    // Get graphics device.
    let physical = PhysicalDevice::enumerate(&instance).next()
        .expect("No device available");

    // Get graphics command queue family from graphics device.
    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Could not find a graphical queue family");

    // Make software device and queue iterator of the graphics family.
    let (device, mut queues) = {
        let device_ext = DeviceExtensions{
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };
        
        Device::new(physical, physical.supported_features(), &device_ext,
                    [(queue_family, 0.5)].iter().cloned())
            .expect("Failed to create device")
    };

    // Get a queue from the iterator.
    let queue = queues.next().unwrap();

    // Make an events loop and a window.
    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

    // Get a swapchain and images for use with the swapchain.
    let (swapchain, images) = {
        let caps = surface.capabilities(physical)
            .expect("Failed to get surface capabilities");
        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        Swapchain::new(device.clone(), surface.clone(),
            caps.min_image_count, format, dimensions, 1, caps.supported_usage_flags, &queue,
            SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None)
            .expect("Failed to create swapchain")
    };

    // Make vertices (4 squares.)
    let vertex_buffer = {
        // Triangle list with grid of 4 squares...
        // 1 2 3
        // 4 5 6
        // 7 8 9
        // ...where 1-2-4-5 is the top-left square.
        let vertices = imagegen::generate_vertices(2, 2);

        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::vertex_buffer(),
            vertices.into_iter()).unwrap()
    };

    // Make the render pass to insert into the command queue.
    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),//Format::R8G8B8A8Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

    // State that may change during pipeline execution (?)
    let mut dynamic_state = DynamicState{
        viewports: Some(vec![Viewport{
            origin: [0.0, 0.0],
            dimensions: [1024.0, 1024.0],
            depth_range: 0.0 .. 1.0,
        }]),
        .. DynamicState::none()
    };

    // Make frame buffers from images (i.e. attach viewports).
    let framebuffers = {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0 .. 1.0,
        };

        dynamic_state.viewports = Some(vec![viewport]);

        images.iter().map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .build().unwrap()
            ) as Arc<FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>()
    };

    // Assemble
    let vs = vs::Shader::load(device.clone()).expect("failed to create vertex shader");
    let fs = fs::Shader::load(device.clone()).expect("failed to create fragment shader");

    // Make pipeline.
    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    // Future foor previous frame completion.
    let mut previous_frame_future = Box::new(now(device.clone())) as Box<GpuFuture>;

    events_loop.run_forever(|event| {
        // Get current framebuffer index from the swapchain.
        let (image_num, acquire_future) = acquire_next_image(swapchain.clone(), None)
            .expect("Didn't get next image");
        
        // Make and submit command buffer using pipeline and current framebuffer.
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue_family).unwrap()
            .begin_render_pass(framebuffers[image_num].clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()]).unwrap()
            .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        // Wait until previous frame is done.
        let mut now_future = Box::new(now(device.clone())) as Box<GpuFuture>;
        std::mem::swap(&mut previous_frame_future, &mut now_future);

        // Wait until previous frame is done _and_ the framebuffer has been acquired.
        let future = now_future.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()                   // Run the commands (pipeline and render)
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)    // Present newly rendered image.
            .then_signal_fence_and_flush();                                         // Signal done and flush the pipeline.

        match future {
            Ok(future) => previous_frame_future = Box::new(future) as Box<_>,
            Err(e) => println!("Err: {:?}", e),
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                ControlFlow::Break
            },
            _ => ControlFlow::Continue,
        }
    });
}