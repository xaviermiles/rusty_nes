use crate::sdl::{Event, Key, SDL};

pub fn draw_frame(sdl: &SDL, window_width: i32) {
    let mut user_x = 100;
    let mut user_y = 100;

    sdl.set_render_draw_color(0, 0, 0, 0);
    sdl.render_clear();
    sdl.set_render_draw_color(255, 0, 0, 255);
    for i in 0..window_width {
        sdl.render_draw_point(i, window_width - i);
    }

    sdl.render_present();

    loop {
        let event = sdl.poll_event();
        match event {
            Event::KeyDown(key) => {
                match key {
                    Key::Up => user_y -= 1,
                    Key::Down => user_y += 1,
                    Key::Left => user_x -= 1,
                    Key::Right => user_x += 1,
                }
                sdl.render_draw_point(user_x, user_y);
                sdl.render_present();
            }
            Event::Quit => break,
            _ => {}
        }
    }
}
