use std::{env, path::Path, time::{Instant, Duration}};
use std::cmp::min;

use gb7_core::{cartridge, gameboy::Gameboy, lcd::Lcd, joypad::JoypadButton};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit::event::{ElementState, Event, WindowEvent};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const TARGET_FPS: u32 = 60;

static CONTROLS: [VirtualKeyCode; 8] = [VirtualKeyCode::Z, VirtualKeyCode::X, VirtualKeyCode::Return, VirtualKeyCode::RShift,
                    VirtualKeyCode::Left, VirtualKeyCode::Right, VirtualKeyCode::Up, VirtualKeyCode::Down];

fn control(key: VirtualKeyCode) -> JoypadButton {
    match key {
        VirtualKeyCode::Z => JoypadButton::A,
        VirtualKeyCode::X => JoypadButton::B,
        VirtualKeyCode::Return => JoypadButton::Start,
        VirtualKeyCode::RShift => JoypadButton::Select,
        VirtualKeyCode::Left => JoypadButton::Left,
        VirtualKeyCode::Right => JoypadButton::Right,
        VirtualKeyCode::Up => JoypadButton::Up,
        VirtualKeyCode::Down => JoypadButton::Down,
        _ => unreachable!("invalid control keycode")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cart_path = Path::new(&args[1]);
    let cartridge = cartridge::load_from_path(&cart_path);

    let mut gameboy = Gameboy::new_dmg(cartridge);

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64 * 3f64, HEIGHT as f64 * 3f64);
        WindowBuilder::new()
            .with_title("gb7")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let active_target_fps: u32 = TARGET_FPS;
    let target_frame_duration: Duration = Duration::from_secs(1) / active_target_fps;
    let mut turbo_enabled: bool = false;

    event_loop.run(move |main_event, _, control_flow| {
        // Handle input events
        match main_event {
            Event::WindowEvent { ref event, .. } => {
                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(keycode) = input.virtual_keycode {
                            if CONTROLS.contains(&keycode) {
                                match input.state {
                                    ElementState::Pressed => gameboy.joypad.press(control(keycode)),
                                    ElementState::Released => gameboy.joypad.release(control(keycode)),
                                }
                            } else if keycode == VirtualKeyCode::Grave {
                                match input.state {
                                    ElementState::Pressed => {
                                        turbo_enabled = true;
                                    }
                                    ElementState::Released => {
                                        turbo_enabled = false;
                                    }
                                }
                            }
                        }
                    },
                    WindowEvent::Resized(size) => {
                        pixels.resize_surface(size.width, size.height)
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::ExitWithCode(0);
                    },
                    _ => (),
                }
                *control_flow = ControlFlow::Poll;
            },
            Event::MainEventsCleared => {
                let frame_start = Instant::now();

                // Execute one gameboy frame
                match turbo_enabled {
                    true => (0..10).for_each(|_| gameboy.execute_frame()),
                    false => gameboy.execute_frame()
                }

                // Wait to conserve framerate
                let elapsed_time = frame_start.elapsed();

                // Show FPS
                let fps = 1e9f64 / (elapsed_time.as_nanos() as f64);


                window.set_title(format!("gb7 - FPS: {:.2}", min(active_target_fps, fps as u32)).as_str());

                if target_frame_duration > elapsed_time {
                    *control_flow = ControlFlow::WaitUntil(frame_start + target_frame_duration);
                }

                window.request_redraw()
            },
            Event::RedrawRequested(_) => {
                // Draw the current frame to screen
                draw_lcd(&gameboy.lcd, pixels.get_frame_mut());
                if pixels
                    .render()
                    .map_err(|e| panic!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            _ => ()
        }
    });
}

fn draw_lcd(lcd: &Lcd, frame: &mut [u8]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let c = match lcd.pixels[i] {
            3 => [0, 0, 0, 255],
            2 => [100, 100, 100, 255],
            1 => [175, 175, 175, 255],
            0 => [255, 255, 255, 255],
            _ => panic!("invalid color code"),
        };

        pixel.copy_from_slice(&c);
    }
}
