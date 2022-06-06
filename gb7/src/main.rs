use std::{env, path::Path, time::Instant};

use gb7_core::{cartridge, gameboy::Gameboy, lcd::Lcd};
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let cart_path = Path::new(&args[1]);
    let cartridge = cartridge::load_from_path(&cart_path);

    let mut gameboy = Gameboy::new_dmg(cartridge);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("RGBL")
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

    let mut frame_start = Instant::now();

    event_loop.run(move |event, _, control_flow| {
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

        // Wait to conserve framerate
        let elapsed_time = Instant::now().duration_since(frame_start).as_millis() as u32;
        let wait_millis = match 1000 / TARGET_FPS >= elapsed_time {
            true => 1000 / TARGET_FPS - elapsed_time,
            false => 0,
        };
        let new_inst = frame_start + std::time::Duration::from_millis(wait_millis as u64);
        *control_flow = ControlFlow::WaitUntil(new_inst);
        frame_start = Instant::now();

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        };
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
