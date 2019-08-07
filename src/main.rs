use vulkano::{
    instance::{
        Instance, PhysicalDevice
    },
    device::{
        Device, DeviceExtensions
    },
    buffer::{
        BufferUsage,
        CpuBufferPool,
        immutable::ImmutableBuffer
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
    sampler::{
        Filter,
        MipmapMode,
        Sampler,
        SamplerAddressMode
    },
    swapchain::{
        Swapchain, SurfaceTransform, PresentMode, acquire_next_image
    },
    sync::{
        now, GpuFuture
    },
    descriptor::descriptor_set::FixedSizeDescriptorSetsPool
};

use vulkano_win::VkSurfaceBuild;

use winit::{
    EventsLoop,
    Event,
    WindowEvent,
    DeviceEvent,
    KeyboardInput,
    ElementState,
    ControlFlow,
    WindowBuilder
};

use cgmath::{
    Matrix4,
    Vector4
};

use std::sync::Arc;

mod imagegen;
mod keystate;
mod vertexgrid;

const TILE_SIZE: usize = 8;     // In pixels
const ATLAS_SIZE: usize = 2;    // In tiles

#[derive(Copy, Clone)]
struct PaletteUniformBufferObject {
    _colours: [Matrix4<f32>; 4]
}

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
    palette_index: u32
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r#"
        #version 450

        layout(location = 0) in vec2 position;
        layout(location = 1) in vec2 tex_coord;
        layout(location = 2) in uint palette_index;

        layout(location = 0) out vec2 texCoordOut;
        layout(location = 1) out uint paletteIndexOut;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
            texCoordOut = tex_coord;
            paletteIndexOut = palette_index;
        }"#
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: r#"
        #version 450

        layout(location = 0) in vec2 texCoord;
        layout(location = 1) flat in uint paletteIndex;

        layout(set = 0, binding = 0) uniform usampler2D atlas;

        layout(set = 1, binding = 0) uniform paletteVals {
            mat4 colours[4];
        } palette;

        layout(location = 0) out vec4 outColor;

        void main() {
            uint texel = texture(atlas, texCoord).x;
            outColor = palette.colours[paletteIndex][texel];
        }"#
    }
}

vulkano::impl_vertex!(Vertex, position, tex_coord, palette_index);

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
    let mut vertex_grid = {
        // Triangle list with grid of 16 squares (4x4), with atlas size 2x2.
        let mut vertex_grid = vertexgrid::VertexGrid::new(4, 4, ATLAS_SIZE);

        // Pick a random tex and palette combo for each tile.
        for y in 0..4 {
            for x in 0..4 {
                vertex_grid.set_tile_texture(x, y, rand::random::<usize>() & 1, rand::random::<usize>() & 1);
                vertex_grid.set_tile_palette(x, y, rand::random::<u32>() & 0b11);
            }
        }

        vertex_grid
    };

    // Make buffer pool for uploading vertices.
    let vertex_buffer_pool = CpuBufferPool::vertex_buffer(device.clone());

    // Make texture atlas.
    // 2x2 textures, textures of size 8x8, texel of size 2 bits.
    let mut texture_atlas = {
        let mut texture_atlas = imagegen::TextureAtlas::new(ATLAS_SIZE, TILE_SIZE);

        texture_atlas.generate_tile_tex(0, 0);
        texture_atlas.generate_tile_tex(1, 0);
        texture_atlas.generate_tile_tex(0, 1);
        texture_atlas.generate_tile_tex(1, 1);

        texture_atlas
    };

    // Make sampler for texture.
    let sampler = Sampler::new(
        device.clone(),
        Filter::Nearest,
        Filter::Nearest,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0, 1.0, 0.0, 0.0
    ).expect("Couldn't create sampler!");

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
            dimensions: [500.0, 500.0],
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

    // Make descriptor set pools.
    let mut set_0_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 0);
    let mut set_1_pool = FixedSizeDescriptorSetsPool::new(pipeline.clone(), 1);

    // Future foor previous frame completion.
    let mut previous_frame_future = Box::new(now(device.clone())) as Box<GpuFuture>;

    // Initial command state.
    let mut state = keystate::KeyState::new();

    events_loop.run_forever(|event| {
        // Get current framebuffer index from the swapchain.
        let (image_num, acquire_future) = acquire_next_image(swapchain.clone(), None)
            .expect("Didn't get next image");

        // Make vertex buffer with current tex coords.
        // TODO: only re-create the buffer when the data has changed.
        // TODO: investigate reducing data copies.
        let vertex_buffer = vertex_buffer_pool.chunk(vertex_grid.vertices.iter().cloned()).unwrap();

        // Make image with current texture.
        // TODO: only re-create the image when the data has changed.
        let (image, write_future) = texture_atlas.make_image(queue.clone());

        // Make palette buffer.
        // TODO: only recreate buffer when the data has changed.
        let (palette_buffer, palette_future) = ImmutableBuffer::from_data(
            PaletteUniformBufferObject{
                _colours: [
                    Matrix4::from_cols(
                        Vector4::new(1.0, 0.0, 0.0, 1.0),
                        Vector4::new(0.8, 0.4, 0.1, 1.0),
                        Vector4::new(1.0, 1.0, 0.0, 1.0),
                        Vector4::new(0.8, 0.2, 0.0, 1.0)
                    ),
                    Matrix4::from_cols(
                        Vector4::new(0.0, 1.0, 0.0, 1.0),
                        Vector4::new(0.0, 0.8, 0.8, 1.0),
                        Vector4::new(0.1, 0.9, 0.3, 1.0),
                        Vector4::new(0.5, 1.0, 0.1, 1.0)
                    ),
                    Matrix4::from_cols(
                        Vector4::new(0.0, 0.0, 1.0, 1.0),
                        Vector4::new(0.3, 0.3, 0.8, 1.0),
                        Vector4::new(0.7, 0.2, 0.9, 1.0),
                        Vector4::new(0.4, 0.0, 0.9, 1.0)
                    ),
                    Matrix4::from_cols(
                        Vector4::new(1.0, 1.0, 1.0, 1.0),
                        Vector4::new(0.6, 0.6, 0.6, 1.0),
                        Vector4::new(0.3, 0.3, 0.3, 1.0),
                        Vector4::new(0.0, 0.0, 0.0, 1.0)
                    )
                ]
            },
            BufferUsage::uniform_buffer(),
            queue.clone()
        ).expect("Couldn't create palette buffer.");

        // Make descriptor set to bind texture atlas.
        let set0 = set_0_pool.next()
            .add_sampled_image(image.clone(), sampler.clone()).unwrap()
            .build().unwrap();

        // Make descriptor set for palettes.
        let set1 = set_1_pool.next()
            .add_buffer(palette_buffer.clone()).unwrap()
            .build().unwrap();
        
        // Make and submit command buffer using pipeline and current framebuffer.
        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue_family).unwrap()
            .begin_render_pass(framebuffers[image_num].clone(), false, vec![[1.0, 1.0, 1.0, 1.0].into()]).unwrap()
            .draw(pipeline.clone(), &dynamic_state, vertex_buffer, (set0, set1), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        // Wait until previous frame is done.
        let mut now_future = Box::new(now(device.clone())) as Box<GpuFuture>;
        std::mem::swap(&mut previous_frame_future, &mut now_future);

        // Wait until previous frame is done,
        // _and_ the framebuffer has been acquired,
        // _and_ the texture has been uploaded,
        // _and_ the palettes have been uploaded.
        let future = now_future.join(acquire_future)
            .join(write_future)
            .join(palette_future)
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
            Event::WindowEvent {
                // Handle keyboard input.
                event: WindowEvent::KeyboardInput{
                    input: KeyboardInput{
                        state: ElementState::Pressed,
                        virtual_keycode: Some(k),
                        .. },
                    .. },
                .. } => {
                let (new_state, command) = state.process_key(k);
                state = new_state;
                if let Some(c) = command {
                    use keystate::Command::*;
                    match c {
                        ModifyTilePalette{ palette: p, x, y }   => vertex_grid.set_tile_palette(x, y, p),
                        ModifyTileTexture{ tex_x, tex_y, x, y } => vertex_grid.set_tile_texture(x, y, tex_x, tex_y),
                        GenerateTexture{ tex_x: x, tex_y: y }   => texture_atlas.generate_tile_tex(x, y)
                    }
                }
                ControlFlow::Continue
            },
            _ => ControlFlow::Continue,
        }
    });
}