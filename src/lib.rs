#![allow(dead_code, unused_variables)] // quieter :^)
mod apu;
mod cart;
mod cpu;
mod ppu;
mod sdl;
mod system;
mod video;

pub use cpu::CPU;
use sdl::SDL;
pub use system::System;

const WINDOW_WIDTH: i32 = 600;

pub fn run() {
    let mut sdl = SDL::construct();
    sdl.init_video(WINDOW_WIDTH, WINDOW_WIDTH);
    video::draw_frame(&sdl, WINDOW_WIDTH);
    sdl.quit();
}
