use nanoview::{Camera, Renderer, Scene, PointLight};
use nanoview::ultraviolet::{Rotor3, Vec3};
use nanomesh::Vector3;
use std::mem::size_of;

use std::f32::consts::PI;

use winit::{
    event_loop::{ControlFlow, EventLoop},
    event::{self, WindowEvent, MouseScrollDelta},
};

fn main() {

    let event_loop = EventLoop::new();

    let title = "decimation-benchmark";

    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_title(title);

    nanoview::futures::executor::block_on(run_async(event_loop, window));
}

async fn run_async(event_loop: EventLoop<()>, window: winit::window::Window) {
    let instance = nanoview::wgpu::Instance::new(nanoview::wgpu::Backends::all());

    let initial_screen_size = window.inner_size();
    let surface = unsafe { instance.create_surface(&window) };
    let needed_extensions = nanoview::wgpu::Features::empty();

    let adapter = instance.request_adapter(
        &nanoview::wgpu::RequestAdapterOptions {
            power_preference: nanoview::wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    ).await.unwrap();
    let adapter_features = adapter.features();

    let (device, queue) = adapter.request_device(
        &nanoview::wgpu::DeviceDescriptor {
            label: None,
            features: adapter_features & needed_extensions,
            limits: nanoview::wgpu::Limits::default(),
        },
        None,
    ).await.unwrap();

    let swapchain_format = surface.get_preferred_format(&adapter).unwrap();
    let mut surface_config = nanoview::wgpu::SurfaceConfiguration {
        usage: nanoview::wgpu::TextureUsages::RENDER_ATTACHMENT, // | nanoview::wgpu::TextureUsages::COPY_DST,
        //format: wgpu::TextureFormat::Bgra8UnormSrgb,
        //format: wgpu::TextureFormat::Bgra8Unorm,
        format: nanoview::wgpu::TextureFormat::Bgra8UnormSrgb,
        width: 512,//initial_screen_size.width,
        height: 512,//initial_screen_size.height,
        present_mode: nanoview::wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &surface_config);

    ////////////////////////////////////

    let mut camera = Camera::new(surface_config.width as f32 / surface_config.height as f32);
    // camera.set_projection(nanoview::ultraviolet::projection::rh_yup::orthographic_gl(
    //     -1000.0, 1000.0, -1000.0, 1000.0, 0.001, 1000.0,
    // ));

    let mut scene = Scene::new(camera);
    let mut renderer = Renderer::new(&surface_config, device, queue);

    let mesh = renderer.mesh_from_file("cases/helmet/helmet_original.glb", true);
    let bbox = mesh.bbox;
    let mesh_id = scene.add_mesh(mesh);

    // Unnecessary but perhaps educational?
    scene.mesh(mesh_id).position = Vec3::zero();
    scene.mesh(mesh_id).scale = Vec3::broadcast(1.0);

    // We'll position these lights down in the render loop
    let light0 = scene.add_point_light(PointLight {
        pos: [0.0; 3],
        color: [1.0, 0.3, 0.3],
        intensity: 800.0,
    });

    let light1 = scene.add_point_light(PointLight {
        pos: [0.0; 3],
        color: [0.3, 1.0, 0.3],
        intensity: 800.0,
    });

    let light2 = scene.add_point_light(PointLight {
        pos: [0.0; 3],
        color: [0.3, 0.3, 1.0],
        intensity: 800.0,
    });

    let winit::dpi::PhysicalSize { width: win_w, height: win_h } = window.inner_size();
    let win_center_x = win_w / 2;
    let win_center_y = win_h / 2;
    let _ignore_error = window
        .set_cursor_position(winit::dpi::LogicalPosition::new(win_center_x, win_center_y))
        .map_err(|_| eprintln!("unable to set cursor position"));
    window.set_maximized(true);

    let mut player_rot_x: f32 = 0.0;
    let mut player_rot_y: f32 = 0.0;
    let mut player_rot = Rotor3::identity();
    let mut camera_distance: f32 = 5.0;
    let mut prev_mouse_x: f64 = 0.0;
    let mut prev_mouse_y: f64 = 0.0;

    camera_distance = 1.3 * bbox.diagonal() as f32;

    let mut timer = timer::Timer::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                //surface_config.width = size.width;
                //surface_config.height = size.height;
                surface.configure(&renderer.device, &surface_config);

                scene.camera.resize(surface_config.width as f32 / surface_config.height as f32);
                renderer.mesh_pass.resize(&surface_config, &mut renderer.device);
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }

                _ => { }
            }
            event::Event::RedrawRequested(_) => {
                let elapsed = timer.get_elapsed_micros();
                let elapsed_seconds = elapsed as f32 / 1_000_000.0;

                // Orbit them lights
                scene.point_light(light0).pos = [
                    10.0 * f32::cos(elapsed_seconds + 0.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 0.0 / 3.0 * 2.0 * PI),
                ];
                scene.point_light(light1).pos = [
                    10.0 * f32::cos(elapsed_seconds + 1.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 1.0 / 3.0 * 2.0 * PI),
                ];
                scene.point_light(light2).pos = [
                    10.0 * f32::cos(elapsed_seconds + 2.0 / 3.0 * 2.0 * PI),
                    10.0,
                    10.0 * f32::sin(elapsed_seconds + 2.0 / 3.0 * 2.0 * PI),
                ];

                // Update camera
                let mut cam_offset = Vec3::new(0.0, 0.0, -camera_distance);
                player_rot.rotate_vec(&mut cam_offset);
                scene.camera.look_at(
                    cam_offset,
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                );

                // Render scene
                let frame = surface.get_current_texture().expect("output frame");
                let mut encoder = renderer.device.create_command_encoder(&nanoview::wgpu::CommandEncoderDescriptor {
                    label: None,
                });

                let texture_desc = nanoview::wgpu::TextureDescriptor {
                    size: nanoview::wgpu::Extent3d {
                        width: surface_config.width,
                        height: surface_config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: nanoview::wgpu::TextureDimension::D2,
                    format: nanoview::wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: nanoview::wgpu::TextureUsages::COPY_SRC | nanoview::wgpu::TextureUsages::RENDER_ATTACHMENT,
                    label: None,
                };

                let buffer_dimensions = BufferDimensions::new(surface_config.width as usize, surface_config.height as usize);

                let output_buffer_desc = nanoview::wgpu::BufferDescriptor {
                    size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
                    usage: nanoview::wgpu::BufferUsages::COPY_DST | nanoview::wgpu::BufferUsages::MAP_READ,
                    label: None,
                    mapped_at_creation: false,
                };
                let output_buffer = renderer.device.create_buffer(&output_buffer_desc);

                // Render to framebuffer
                let fb_texture = renderer.device.create_texture(&texture_desc);
                let fb_view = fb_texture.create_view(&nanoview::wgpu::TextureViewDescriptor::default());
                renderer.render(&fb_view, &mut encoder, &scene);

                // Render to surface
                let texture_view = frame.texture.create_view(&nanoview::wgpu::TextureViewDescriptor::default());
                renderer.render(&texture_view, &mut encoder, &scene);

                encoder.copy_texture_to_buffer(
                    nanoview::wgpu::ImageCopyTexture {
                        aspect: nanoview::wgpu::TextureAspect::All,
                            texture: &fb_texture,
                        mip_level: 0,
                        origin: nanoview::wgpu::Origin3d::ZERO,
                    },
                    nanoview::wgpu::ImageCopyBuffer {
                        buffer: &output_buffer,
                        layout: nanoview::wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: std::num::NonZeroU32::new(buffer_dimensions.padded_bytes_per_row as u32),
                            rows_per_image: std::num::NonZeroU32::new(surface_config.height),
                        },
                    },
                    texture_desc.size,
                );

                renderer.queue.submit(Some(encoder.finish()));
                frame.present();

                // We need to scope the mapping variables so that we can
                // unmap the buffer
                {
                    let buffer_slice = output_buffer.slice(..);

                    // NOTE: We have to create the mapping THEN device.poll() before await
                    // the future. Otherwise the application will freeze.
                    let mapping = buffer_slice.map_async(nanoview::wgpu::MapMode::Read);
                    renderer.device.poll(nanoview::wgpu::Maintain::Wait);
                    pollster::block_on(mapping).unwrap();

                    let data = buffer_slice.get_mapped_range();

                    image::save_buffer("/Users/oginiaux/Projects/nanolabo/decimation-benchmark/image.jpg", 
                        data.as_ref(),
                        surface_config.width,
                        surface_config.height,
                        image::ColorType::Bgra8).unwrap();
                }
                output_buffer.unmap();
            }
            _ => (),
        }
    });
}

mod timer {
    use std::time::Instant;

    pub struct Timer {
        last: Instant,
    }

    impl Timer {
        pub fn new() -> Timer {
            let now = Instant::now();
            Timer {
                last: now,
            }
        }

        pub fn get_elapsed_micros(&mut self) -> u64 {
            let now = Instant::now();
            let duration = now.duration_since(self.last);
            let interval = duration.as_micros() as u64;

            interval
        }

        pub fn clear(&mut self) {
            let now = Instant::now();
            self.last = now;
        }
    }
}

struct BufferDimensions {
    width: usize,
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = nanoview::wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}