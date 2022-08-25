use std::{env, path::Path, time::{Instant, Duration}};

use gb7_core::{cartridge, gameboy::Gameboy, lcd::Lcd, joypad::JoypadButton};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::VirtualKeyCode,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

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
    let mut input = WinitInputHelper::new();
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

    let target_frame_duration: Duration = Duration::from_secs(1) / TARGET_FPS;

    event_loop.run(move |event, _, control_flow| {
        let frame_start = Instant::now();

        // Execute one gameboy frame
        gameboy.execute_frame();

        // Draw the current frame to screen
        draw_lcd(&gameboy.lcd, pixels.get_frame());
        if pixels
            .render()
            .map_err(|e| panic!("pixels.render() failed: {}", e))
            .is_err()
        {
            *control_flow = ControlFlow::Exit;
            return;
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Button press/release
            for ctr in CONTROLS {
                if input.key_pressed(ctr) { 
                    gameboy.joypad.press(control(ctr));
                }
                if input.key_released(ctr) { 
                    gameboy.joypad.release(control(ctr));
                }
            }
        };

        // Wait to conserve framerate
        let elapsed_time = frame_start.elapsed();
        
        // Show FPS
        let fps = 1e9f64 / (elapsed_time.as_nanos() as f64);
        window.set_title(format!("gb7 - FPS: {:.2}", fps).as_str());

        if target_frame_duration > elapsed_time {
            *control_flow = ControlFlow::WaitUntil(frame_start + target_frame_duration);
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
