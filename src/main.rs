use assets_manager::asset::Png;

use frenderer::{
    input::{self, Key},
    sprites::{Camera2D, SheetRegion, Transform},
    wgpu,
};

struct State {
    x: f32,
    y: f32,
    frame: usize,
}

fn main() {
    let mut input = input::Input::default();

    #[cfg(not(target_arch = "wasm32"))]
    let source =
        assets_manager::source::FileSystem::new("content").expect("Couldn't load resources");
    #[cfg(target_arch = "wasm32")]
    let source = assets_manager::source::Embedded::from(assets_manager::source::embed!("content"));
    let cache = assets_manager::AssetCache::with_source(source);

    let img_handle = cache
        .load::<Png>("tilesheet")
        .expect("Couldn't load spritesheet img");
    let img = img_handle.read().0.to_rgba8();

    let driver = frenderer::Driver::new(
        winit::window::WindowBuilder::new()
            .with_title("demo")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0)),
        Some((1024, 768)),
    );
    const DT: f32 = 1.0 / 60.0;
    let mut acc = 0.0;
    let mut now = frenderer::clock::Instant::now();
    driver
        .run_event_loop::<(), _>(
            move |window, mut frenderer| {
                let img_tex = frenderer.create_array_texture(
                    &[&img],
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    img.dimensions(),
                    Some("tilesheet"),
                );
                let camera = Camera2D {
                    screen_pos: [0.0, 0.0],
                    screen_size: [1024.0, 768.0],
                };
                frenderer.sprite_group_add(
                    &img_tex,
                    vec![Transform {
                        x: 1024.0 / 2.0,
                        y: 768.0 / 2.0,
                        w: 32,
                        h: 32,
                        rot: 0.0,
                    }],
                    vec![SheetRegion::new(0, 0, 578, 0, 16, 16)],
                    camera,
                );
                (
                    window,
                    frenderer,
                    State {
                        x: 1024.0 / 2.0,
                        y: 768.0 / 2.0,
                        frame: 0,
                    },
                )
            },
            move |event, target, (window, frenderer, state)| {
                use winit::event::{Event, WindowEvent};
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        target.exit();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        ..
                    } => {
                        if !frenderer.gpu.is_web() {
                            frenderer.resize_surface(size.width, size.height);
                        }
                        window.request_redraw();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        // compute elapsed time since last frame
                        let elapsed = now.elapsed().as_secs_f32();
                        acc += elapsed;
                        now = frenderer::clock::Instant::now();
                        // While we have time to spend
                        while acc >= DT {
                            // simulate a frame
                            acc -= DT;
                            state.x += input.key_axis(Key::ArrowLeft, Key::ArrowRight) * 2.0;
                            state.y += input.key_axis(Key::ArrowDown, Key::ArrowUp) * 2.0;
                            state.frame += 1;
                            input.next_frame();
                        }

                        frenderer.sprite_group_resize(0, 2);
                        let (trfs, uvs) = frenderer.sprites_mut(0, 0..2);
                        trfs[0].x = state.x;
                        trfs[0].y = state.y;
                        if (state.frame / 10) % 2 == 0 {
                            trfs[1] = Transform {
                                x: 0.5 * 1024.0 / 2.0,
                                y: 0.5 * 768.0 / 2.0,
                                w: 64,
                                h: 64,
                                rot: state.frame as f32 * std::f32::consts::PI * DT,
                            };
                            uvs[1] = SheetRegion::new(0, 0, 646, 0, 16, 16);
                        } else {
                            trfs[1] = Transform::ZERO;
                        }

                        frenderer.render();
                        window.request_redraw();
                    }
                    event => {
                        input.process_input_event(&event);
                    }
                }
            },
        )
        .unwrap();
}
