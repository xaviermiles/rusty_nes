mod apu;
mod cart;
mod ppu;
mod sdl;
mod system;
mod video;

pub use cart::{load_to_cart, CartLoadStatus};
use sdl::SDL;

const WINDOW_WIDTH: i32 = 600;

pub fn run() {
    let mut sdl = SDL::construct();
    sdl.init_video(WINDOW_WIDTH, WINDOW_WIDTH);
    video::draw_frame(&sdl, WINDOW_WIDTH);
    sdl.quit();
}
