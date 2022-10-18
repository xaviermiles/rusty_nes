mod sdl;
mod video;

use sdl::SDL;
use video::draw_frame;

const WINDOW_WIDTH: i32 = 600;

pub fn run() {
    let mut sdl = SDL::construct();
    sdl.init_video(WINDOW_WIDTH, WINDOW_WIDTH);
    draw_frame(&sdl, WINDOW_WIDTH);
    sdl.quit();
}
