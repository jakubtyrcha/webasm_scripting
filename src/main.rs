#![cfg_attr(
    not(any(
        feature = "vulkan",
        feature = "dx11",
        feature = "dx12",
        feature = "metal"
    )),
    allow(dead_code, unused_extern_crates, unused_imports)
)]

#[cfg(feature = "dx11")]
extern crate gfx_backend_dx11 as back;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;

use nalgebra_glm as glm;
use hal::format::{AsFormat, ChannelType, Rgba8Srgb as ColorFormat, Swizzle};
use hal::pass::Subpass;
use hal::pso::{PipelineStage, ShaderStageFlags, VertexInputRate};
use hal::queue::Submission;
use hal::{
    buffer,
    command,
    format as f,
    image as i,
    memory as m,
    pass,
    pool,
    pso,
    window::Extent2D,
};
use hal::{DescriptorPool, Primitive, SwapchainConfig};
use hal::{Device, Instance, PhysicalDevice, Surface, Swapchain};

use std::io::Cursor;
use std::fs;
use std::time::{Duration, Instant};

#[cfg_attr(rustfmt, rustfmt_skip)]
const DIMS: Extent2D = Extent2D { width: 1024,height: 768 };

const ENTRY_NAME: &str = "main";

struct WorldState
{
    camera_position : glm::Vec3,
    camera_lookat : glm::Vec3,
    camera_up : glm::Vec3,
}

impl WorldState
{
    fn set_camera(&mut self, position : &[f32; 3])
    {
        self.camera_position = glm::Vec3::from_row_slice(position);
    }

    fn add_particle(&mut self, position : &[f32; 3])
    {

    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Vertex {
    a_Pos: [f32; 4],
    a_Uv: [f32; 2],
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD: [Vertex; 6] = [
    Vertex { a_Pos: [ -0.5, 0.33, 0.0, 1.0 ], a_Uv: [0.0, 1.0] },
    Vertex { a_Pos: [  0.5, 0.33, 0.0, 1.0 ], a_Uv: [1.0, 1.0] },
    Vertex { a_Pos: [  0.5,-0.33, 0.0, 1.0 ], a_Uv: [1.0, 0.0] },

    Vertex { a_Pos: [ -0.5, 0.33, 0.0, 1.0 ], a_Uv: [0.0, 1.0] },
    Vertex { a_Pos: [  0.5,-0.33, 0.0, 1.0 ], a_Uv: [1.0, 0.0] },
    Vertex { a_Pos: [ -0.5,-0.33, 0.0, 1.0 ], a_Uv: [0.0, 0.0] },
];

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: f::Aspects::COLOR,
    levels: 0 .. 1,
    layers: 0 .. 1,
};

#[cfg(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal"
))]
fn main() {
    env_logger::init();

    let mut events_loop = winit::EventsLoop::new();
    let wb = winit::WindowBuilder::new()
        .with_min_dimensions(winit::dpi::LogicalSize::new(1.0, 1.0))
        .with_dimensions(winit::dpi::LogicalSize::new(
            DIMS.width as _,
            DIMS.height as _,
        ))
        .with_title("quad".to_string());
    // instantiate backend
    let (_window, _instance, mut adapters, mut surface) = {
        let window = wb.build(&events_loop).unwrap();
        let instance = back::Instance::create("gfx-rs quad", 1);
        let surface = instance.create_surface(&window);
        let adapters = instance.enumerate_adapters();
        (window, instance, adapters, surface)
    };

    for adapter in &adapters {
        println!("{:?}", adapter.info);
    }

    let mut adapter = adapters.remove(0);
    let memory_types = adapter.physical_device.memory_properties().memory_types;
    let limits = adapter.physical_device.limits();

    // Build a new device and associated command queues
    let (device, mut queue_group) = adapter
        .open_with::<_, hal::Graphics>(1, |family| surface.supports_queue_family(family))
        .unwrap();

    let mut command_pool = unsafe {
        device.create_command_pool_typed(&queue_group, pool::CommandPoolCreateFlags::empty())
    }
    .expect("Can't create command pool");

    // Setup renderpass and pipeline
    let set_layout = unsafe {
        device.create_descriptor_set_layout(
            &[
                pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: pso::DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: ShaderStageFlags::VERTEX,
                    immutable_samplers: false,
                },
            ],
            &[],
        )
    }
    .expect("Can't create descriptor set layout");

    // Define maximum number of frames we want to be able to be "in flight" (being computed
    // simultaneously) at once
    const FRAMES_IN_FLIGHT : usize = 3;

    type Buffer = <back::Backend as hal::Backend>::Buffer;
    type Memory = <back::Backend as hal::Backend>::Memory;

    struct Frame
    {
        desc_set : Option<<back::Backend as hal::Backend>::DescriptorSet>,
        ubuffer : Option<Buffer>,
        ubuffer_mem : Option<Memory>
    }

    impl Frame
    {
        fn new() -> Frame 
        {
            Frame { desc_set : None, ubuffer : None, ubuffer_mem : None }
        }
    }

    let mut frames : [Frame; FRAMES_IN_FLIGHT] = [Frame::new(), Frame::new(), Frame::new(), ];

    // Descriptors
    let mut desc_pool = unsafe {
        device.create_descriptor_pool(
            FRAMES_IN_FLIGHT, // sets
            &[
                pso::DescriptorRangeDesc {
                    ty: pso::DescriptorType::UniformBuffer,
                    count: FRAMES_IN_FLIGHT,
                },
            ],
            pso::DescriptorPoolCreateFlags::empty(),
        )
    }
    .expect("Can't create descriptor pool");

    for i in 0..FRAMES_IN_FLIGHT {
        frames[i].desc_set = Some(unsafe { desc_pool.allocate_set(&set_layout) }.unwrap());
        println!("allocated {}", i);
    }

    // Buffer allocations
    println!("Memory types: {:?}", memory_types);

    let buffer_stride = std::mem::size_of::<Vertex>() as u64;
    let buffer_len = QUAD.len() as u64 * buffer_stride;

    assert_ne!(buffer_len, 0);
    let mut vertex_buffer =
        unsafe { device.create_buffer(buffer_len, buffer::Usage::VERTEX) }.unwrap();

    for i in 0..FRAMES_IN_FLIGHT {
        frames[i].ubuffer = Some(unsafe { device.create_buffer(64, buffer::Usage::UNIFORM) }.unwrap());
    }

    fn allocate_ubuffer_mem(device : & <back::Backend as hal::Backend>::Device, heaps : &hal::adapter::MemoryProperties, mut buffer : &mut Buffer) -> Memory
    {
        let buffer_req = unsafe { device.get_buffer_requirements(buffer) };

        let upload_type = heaps.memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
            // type_mask is a bit field where each bit represents a memory type. If the bit is set
            // to 1 it means we can use that type for our buffer. So this code finds the first
            // memory type that has a `1` (or, is allowed), and is visible to the CPU.
            buffer_req.type_mask & (1 << id) != 0
                && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

        let buffer_memory = unsafe { device.allocate_memory(upload_type, buffer_req.size) }.unwrap();

        unsafe { device.bind_buffer_memory(&buffer_memory, 0, &mut buffer) }.unwrap();

        buffer_memory
    }

    for i in 0..FRAMES_IN_FLIGHT {
        let mem = allocate_ubuffer_mem(&device, &adapter.physical_device.memory_properties(), frames[i].ubuffer.as_mut().unwrap());
        frames[i].ubuffer_mem = Some(mem);
    }

    let vbuffer_memory = {
        let buffer_req = unsafe { device.get_buffer_requirements(&vertex_buffer) };

        let upload_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, mem_type)| {
            // type_mask is a bit field where each bit represents a memory type. If the bit is set
            // to 1 it means we can use that type for our buffer. So this code finds the first
            // memory type that has a `1` (or, is allowed), and is visible to the CPU.
            buffer_req.type_mask & (1 << id) != 0
                && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
        })
        .unwrap()
        .into();

        let buffer_memory = unsafe { device.allocate_memory(upload_type, buffer_req.size) }.unwrap();

        unsafe { device.bind_buffer_memory(&buffer_memory, 0, &mut vertex_buffer) }.unwrap();

        // TODO: check transitions: read/write mapping and vertex buffer read
        unsafe {
            let mut vertices = device
                .acquire_mapping_writer::<Vertex>(&buffer_memory, 0 .. buffer_req.size)
                .unwrap();
            vertices[0 .. QUAD.len()].copy_from_slice(&QUAD);
            device.release_mapping_writer(vertices).unwrap();
        };

        buffer_memory
    };

    for i in 0..FRAMES_IN_FLIGHT {
        unsafe {
            device.write_descriptor_sets(vec![
                pso::DescriptorSetWrite {
                    set: frames[i].desc_set.as_ref().unwrap(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Buffer(frames[i].ubuffer.as_ref().unwrap(), None..None)),
                },
            ]);
        }
    }

    let (caps, formats, _present_modes) = surface.compatibility(&mut adapter.physical_device);
    println!("formats: {:?}", formats);
    let format = formats.map_or(f::Format::Rgba8Srgb, |formats| {
        formats
            .iter()
            .find(|format| format.base_format().1 == ChannelType::Srgb)
            .map(|format| *format)
            .unwrap_or(formats[0])
    });

    let swap_config = SwapchainConfig::from_caps(&caps, format, DIMS);
    println!("{:?}", swap_config);
    let extent = swap_config.extent.to_extent();

    let (mut swap_chain, mut backbuffer) =
        unsafe { device.create_swapchain(&mut surface, swap_config, None) }
            .expect("Can't create swapchain");

    let render_pass = {
        let attachment = pass::Attachment {
            format: Some(format),
            samples: 1,
            ops: pass::AttachmentOps::new(
                pass::AttachmentLoadOp::Clear,
                pass::AttachmentStoreOp::Store,
            ),
            stencil_ops: pass::AttachmentOps::DONT_CARE,
            layouts: i::Layout::Undefined .. i::Layout::Present,
        };

        let subpass = pass::SubpassDesc {
            colors: &[(0, i::Layout::ColorAttachmentOptimal)],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        };

        let dependency = pass::SubpassDependency {
            passes: pass::SubpassRef::External .. pass::SubpassRef::Pass(0),
            stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT
                .. PipelineStage::COLOR_ATTACHMENT_OUTPUT,
            accesses: i::Access::empty()
                .. (i::Access::COLOR_ATTACHMENT_READ | i::Access::COLOR_ATTACHMENT_WRITE),
        };

        unsafe { device.create_render_pass(&[attachment], &[subpass], &[dependency]) }
            .expect("Can't create render pass")
    };

    let (mut frame_images, mut framebuffers) = {
        let pairs = backbuffer
            .into_iter()
            .map(|image| unsafe {
                let rtv = device
                    .create_image_view(
                        &image,
                        i::ViewKind::D2,
                        format,
                        Swizzle::NO,
                        COLOR_RANGE.clone(),
                    )
                    .unwrap();
                (image, rtv)
            })
            .collect::<Vec<_>>();
        let fbos = pairs
            .iter()
            .map(|&(_, ref rtv)| unsafe {
                device
                    .create_framebuffer(&render_pass, Some(rtv), extent)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        (pairs, fbos)
    }; 

    // Number of image acquisition semaphores is based on the number of swapchain images, not frames in flight,
    // plus one extra which we can guarantee is unused at any given time by swapping it out with the ones
    // in the rest of the queue.
    let mut image_acquire_semaphores = Vec::with_capacity(frame_images.len());
    let mut free_acquire_semaphore = device
        .create_semaphore()
        .expect("Could not create semaphore");

    // The number of the rest of the resources is based on the frames in flight.
    let mut submission_complete_semaphores = Vec::with_capacity(FRAMES_IN_FLIGHT);
    let mut submission_complete_fences = Vec::with_capacity(FRAMES_IN_FLIGHT);
    // Note: We don't really need a different command pool per frame in such a simple demo like this,
    // but in a more 'real' application, it's generally seen as optimal to have one command pool per
    // thread per frame. There is a flag that lets a command pool reset individual command buffers
    // which are created from it, but by default the whole pool (and therefore all buffers in it)
    // must be reset at once. Furthermore, it is often the case that resetting a whole pool is actually
    // faster and more efficient for the hardware than resetting individual command buffers, so it's
    // usually best to just make a command pool for each set of buffers which need to be reset at the
    // same time (each frame). In our case, each pool will only have one command buffer created from it,
    // though.
    let mut cmd_pools = Vec::with_capacity(FRAMES_IN_FLIGHT);
    let mut cmd_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);

    cmd_pools.push(command_pool);
    for _ in 1 .. FRAMES_IN_FLIGHT {
        unsafe {
            cmd_pools.push(
                device
                    .create_command_pool_typed(&queue_group, pool::CommandPoolCreateFlags::empty())
                    .expect("Can't create command pool"),
            );
        }
    }

    for _ in 0 .. frame_images.len() {
        image_acquire_semaphores.push(
            device
                .create_semaphore()
                .expect("Could not create semaphore"),
        );
    }

    for i in 0 .. FRAMES_IN_FLIGHT {
        submission_complete_semaphores.push(
            device
                .create_semaphore()
                .expect("Could not create semaphore"),
        );
        submission_complete_fences.push(
            device
                .create_fence(true)
                .expect("Could not create semaphore"),
        );
        cmd_buffers.push(cmd_pools[i].acquire_command_buffer::<command::MultiShot>());
    }

    let pipeline_layout = unsafe {
        device.create_pipeline_layout(
            std::iter::once(&set_layout),
            &[(pso::ShaderStageFlags::VERTEX, 0 .. 8)],
        )
    }
    .expect("Can't create pipeline layout");
    let pipeline = {
        // let vs_module = {
        //     let spirv =
        //         hal::read_spirv(Cursor::new(&include_bytes!("data/quad.vert.spv")[..])).unwrap();
        //     unsafe { device.create_shader_module(&spirv) }.unwrap()
        // };
        // let fs_module = {
        //     let spirv =
        //         hal::read_spirv(Cursor::new(&include_bytes!("./data/quad.frag.spv")[..])).unwrap();
        //     unsafe { device.create_shader_module(&spirv) }.unwrap()
        // };

        let vs_module = {
            let glsl = fs::read_to_string("data/quad.vert").unwrap();
            let file = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Vertex).unwrap();
            let spirv: Vec<u32> = hal::read_spirv(file).unwrap();
            let shader = unsafe { device.create_shader_module(&spirv) }.unwrap();

            shader
        };

        let fs_module = {
            let glsl = fs::read_to_string("data/quad.frag").unwrap();
            let file = glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Fragment).unwrap();
            let spirv: Vec<u32> = hal::read_spirv(file).unwrap();
            let shader = unsafe { device.create_shader_module(&spirv) }.unwrap();

            shader
        };

        let pipeline = {
            let (vs_entry, fs_entry) = (
                pso::EntryPoint {
                    entry: ENTRY_NAME,
                    module: &vs_module,
                    specialization: hal::spec_const_list![0.8f32],
                },
                pso::EntryPoint {
                    entry: ENTRY_NAME,
                    module: &fs_module,
                    specialization: pso::Specialization::default(),
                },
            );

            let shader_entries = pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let subpass = Subpass {
                index: 0,
                main_pass: &render_pass,
            };

            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                shader_entries,
                Primitive::TriangleList,
                pso::Rasterizer::FILL,
                &pipeline_layout,
                subpass,
            );
            pipeline_desc.blender.targets.push(pso::ColorBlendDesc(
                pso::ColorMask::ALL,
                pso::BlendState::ALPHA,
            ));
            pipeline_desc.vertex_buffers.push(pso::VertexBufferDesc {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as u32,
                rate: VertexInputRate::Vertex,
            });

            pipeline_desc.attributes.push(pso::AttributeDesc {
                location: 0,
                binding: 0,
                element: pso::Element {
                    format: f::Format::Rgba32Sfloat,
                    offset: 0,
                },
            });
            pipeline_desc.attributes.push(pso::AttributeDesc {
                location: 1,
                binding: 0,
                element: pso::Element {
                    format: f::Format::Rg32Sfloat,
                    offset: 8,
                },
            });

            unsafe { device.create_graphics_pipeline(&pipeline_desc, None) }
        };

        unsafe {
            device.destroy_shader_module(vs_module);
        }
        unsafe {
            device.destroy_shader_module(fs_module);
        }

        pipeline.unwrap()
    };

    // Rendering setup
    let mut viewport = pso::Viewport {
        rect: pso::Rect {
            x: 0,
            y: 0,
            w: extent.width as _,
            h: extent.height as _,
        },
        depth: 0.0 .. 1.0,
    };

    //
    let mut running = true;
    let mut recreate_swapchain = false;
    let mut resize_dims = Extent2D {
        width: 0,
        height: 0,
    };
    let now = Instant::now();
    let mut frame: u64 = 0;
    while running {
        running = true;
        events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event, .. } = event {
                #[allow(unused_variables)]
                match event {
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | winit::WindowEvent::CloseRequested => running = false,
                    winit::WindowEvent::Resized(dims) => {
                        println!("resized to {:?}", dims);
                        recreate_swapchain = true;
                        resize_dims.width = dims.width as u32;
                        resize_dims.height = dims.height as u32;
                    }
                    _ => (),
                }
            }
        });

        // Window was resized so we must recreate swapchain and framebuffers
        if recreate_swapchain {
            device.wait_idle().unwrap();

            let (caps, formats, _present_modes) =
                surface.compatibility(&mut adapter.physical_device);
            // Verify that previous format still exists so we may reuse it.
            assert!(formats.iter().any(|fs| fs.contains(&format)));

            let swap_config = SwapchainConfig::from_caps(&caps, format, resize_dims);
            println!("{:?}", swap_config);
            let extent = swap_config.extent.to_extent();

            let (new_swap_chain, new_backbuffer) =
                unsafe { device.create_swapchain(&mut surface, swap_config, Some(swap_chain)) }
                    .expect("Can't create swapchain");

            unsafe {
                // Clean up the old framebuffers, images and swapchain
                for framebuffer in framebuffers {
                    device.destroy_framebuffer(framebuffer);
                }
                for (_, rtv) in frame_images {
                    device.destroy_image_view(rtv);
                }
            }

            backbuffer = new_backbuffer;
            swap_chain = new_swap_chain;

            let (new_frame_images, new_framebuffers) = {
                let pairs = backbuffer
                    .into_iter()
                    .map(|image| unsafe {
                        let rtv = device
                            .create_image_view(
                                &image,
                                i::ViewKind::D2,
                                format,
                                Swizzle::NO,
                                COLOR_RANGE.clone(),
                            )
                            .unwrap();
                        (image, rtv)
                    })
                    .collect::<Vec<_>>();
                let fbos = pairs
                    .iter()
                    .map(|&(_, ref rtv)| unsafe {
                        device
                            .create_framebuffer(&render_pass, Some(rtv), extent)
                            .unwrap()
                    })
                    .collect();
                (pairs, fbos)
            };

            framebuffers = new_framebuffers;
            frame_images = new_frame_images;
            viewport.rect.w = extent.width as _;
            viewport.rect.h = extent.height as _;
            recreate_swapchain = false;
        }

        // Use guaranteed unused acquire semaphore to get the index of the next frame we will render to
        // by using acquire_image
        let swap_image = unsafe {
            match swap_chain.acquire_image(!0, Some(&free_acquire_semaphore), None) {
                Ok((i, _)) => i as usize,
                Err(_) => {
                    recreate_swapchain = true;
                    continue;
                }
            }
        };

        // Swap the acquire semaphore with the one previously associated with the image we are acquiring
        core::mem::swap(
            &mut free_acquire_semaphore,
            &mut image_acquire_semaphores[swap_image],
        );

        // Compute index into our resource ring buffers based on the frame number
        // and number of frames in flight. Pay close attention to where this index is needed
        // versus when the swapchain image index we got from acquire_image is needed.
        let frame_idx = frame as usize % FRAMES_IN_FLIGHT;

        let elapsed_sec = now.elapsed().as_micros() as f32 / 1000000.;
        let t = elapsed_sec;

        let proj = glm::perspective(1.0, glm::half_pi::<f32>() * 0.8, 1.0 / 16.0, 1024.);
        let lookat = glm::look_at(&glm::vec3(t.sin() * 10.0, 0.0, t.cos() * 10.0), &glm::vec3(0.0, 0.0, 0.0), &glm::vec3(0.0, 1.0, 0.0));
        let view_proj =  proj * lookat;

        let uniform_mvp: [[f32; 4]; 4] = view_proj.into();

        unsafe {
            let mut constants = device
                .acquire_mapping_writer::<[[f32; 4]; 4]>(frames[frame_idx].ubuffer_mem.as_ref().unwrap(), 0 .. 64)
                .unwrap();

            constants[0] = uniform_mvp;
                
            device.release_mapping_writer(constants).unwrap();
        }

        // Wait for the fence of the previous submission of this frame and reset it; ensures we are
        // submitting only up to maximum number of FRAMES_IN_FLIGHT if we are submitting faster than
        // the gpu can keep up with. This would also guarantee that any resources which need to be
        // updated with a CPU->GPU data copy are not in use by the GPU, so we can perform those updates.
        // In this case there are none to be done, however.
        unsafe {
            device
                .wait_for_fence(&submission_complete_fences[frame_idx], !0)
                .expect("Failed to wait for fence");
            device
                .reset_fence(&submission_complete_fences[frame_idx])
                .expect("Failed to reset fence");
            cmd_pools[frame_idx].reset(false);
        }

        // Rendering
        let cmd_buffer = &mut cmd_buffers[frame_idx];
        unsafe {
            cmd_buffer.begin(false);

            cmd_buffer.set_viewports(0, &[viewport.clone()]);
            cmd_buffer.set_scissors(0, &[viewport.rect]);
            cmd_buffer.bind_graphics_pipeline(&pipeline);
            cmd_buffer.bind_vertex_buffers(0, Some((&vertex_buffer, 0)));
            cmd_buffer.bind_graphics_descriptor_sets(&pipeline_layout, 0, frames[frame_idx].desc_set.as_ref(), &[]);

            {
                let mut encoder = cmd_buffer.begin_render_pass_inline(
                    &render_pass,
                    &framebuffers[swap_image],
                    viewport.rect,
                    &[command::ClearValue::Color(command::ClearColor::Sfloat([
                        0.0, 0.0, 0.0, 0.0,
                    ]))],
                );
                encoder.draw(0 .. 6, 0 .. 1);
            }

            cmd_buffer.finish();

            let submission = Submission {
                command_buffers: Some(&*cmd_buffer),
                wait_semaphores: Some((
                    &image_acquire_semaphores[swap_image],
                    PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                )),
                signal_semaphores: Some(&submission_complete_semaphores[frame_idx]),
            };
            queue_group.queues[0].submit(submission, Some(&submission_complete_fences[frame_idx]));

            // present frame
            if let Err(_) = swap_chain.present(
                &mut queue_group.queues[0],
                swap_image as hal::SwapImageIndex,
                Some(&submission_complete_semaphores[frame_idx]),
            ) {
                recreate_swapchain = true;
            }
        }
        // Increment our frame
        frame += 1;
    }

    // cleanup!
    device.wait_idle().unwrap();
    unsafe {
        device.destroy_descriptor_pool(desc_pool);
        device.destroy_descriptor_set_layout(set_layout);

        device.destroy_buffer(vertex_buffer);
        device.destroy_semaphore(free_acquire_semaphore);
        for p in cmd_pools {
            device.destroy_command_pool(p.into_raw());
        }
        for s in image_acquire_semaphores {
            device.destroy_semaphore(s);
        }
        for s in submission_complete_semaphores {
            device.destroy_semaphore(s);
        }
        for f in submission_complete_fences {
            device.destroy_fence(f);
        }
        device.destroy_render_pass(render_pass);
        device.free_memory(vbuffer_memory);
        //device.free_memory(cbuffer_memory);
        device.destroy_graphics_pipeline(pipeline);
        device.destroy_pipeline_layout(pipeline_layout);
        for framebuffer in framebuffers {
            device.destroy_framebuffer(framebuffer);
        }
        for (_, rtv) in frame_images {
            device.destroy_image_view(rtv);
        }

        device.destroy_swapchain(swap_chain);
    }
}

#[cfg(not(any(
    feature = "vulkan",
    feature = "dx11",
    feature = "dx12",
    feature = "metal"
)))]
fn main() {
    println!("You need to enable the native API feature (vulkan/metal/dx11/dx12) in order to test the LL");
}
