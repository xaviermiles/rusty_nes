use fermium::{
    prelude::{SDL_Event, SDL_PollEvent, SDL_KEYDOWN, SDL_QUIT},
    renderer::{
        SDL_CreateRenderer, SDL_DestroyRenderer, SDL_RenderClear, SDL_RenderDrawPoint,
        SDL_RenderPresent, SDL_Renderer, SDL_SetRenderDrawColor,
    },
    scancode::{SDL_SCANCODE_DOWN, SDL_SCANCODE_LEFT, SDL_SCANCODE_RIGHT, SDL_SCANCODE_UP},
    video::{
        SDL_CreateWindow, SDL_DestroyWindow, SDL_Window, SDL_WINDOWPOS_CENTERED,
        SDL_WINDOW_ALLOW_HIGHDPI, SDL_WINDOW_OPENGL,
    },
    SDL_Init, SDL_Quit, SDL_INIT_VIDEO,
};

pub enum Key {
    Up,
    Down,
    Left,
    Right,
}

pub enum Event {
    #[allow(dead_code)] // TODO: Will KeyUp be necessary?
    KeyUp(Key),
    KeyDown(Key),
    Quit,
}

#[allow(clippy::upper_case_acronyms)]
pub struct SDL {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
}

impl SDL {
    pub fn construct() -> Self {
        Self {
            window: 0 as *mut SDL_Window,
            renderer: 0 as *mut SDL_Renderer,
        }
    }

    // TODO: can this
    pub fn init_video(&mut self, width: i32, height: i32) {
        unsafe {
            SDL_Init(SDL_INIT_VIDEO);
            self.window = SDL_CreateWindow(
                b"rusty-nes".as_ptr().cast(),
                SDL_WINDOWPOS_CENTERED,
                SDL_WINDOWPOS_CENTERED,
                width,
                height,
                (SDL_WINDOW_OPENGL | SDL_WINDOW_ALLOW_HIGHDPI).0,
            );
            self.renderer = SDL_CreateRenderer(self.window, 0, 0);
        }
    }

    pub fn set_render_draw_color(&self, r: u8, g: u8, b: u8, a: u8) {
        unsafe {
            SDL_SetRenderDrawColor(self.renderer, r, g, b, a);
        }
    }

    pub fn render_clear(&self) {
        unsafe {
            SDL_RenderClear(self.renderer);
        }
    }

    pub fn render_draw_point(&self, x: i32, y: i32) {
        unsafe {
            SDL_RenderDrawPoint(self.renderer, x, y);
        }
    }

    pub fn render_present(&self) {
        unsafe {
            SDL_RenderPresent(self.renderer);
        }
    }

    pub fn poll_event(&self) -> Event {
        unsafe {
            let mut event: SDL_Event = SDL_Event::default();
            loop {
                SDL_PollEvent(&mut event);
                match event.type_ {
                    SDL_KEYDOWN => {
                        let potential_key = match event.key.keysym.scancode {
                            SDL_SCANCODE_UP => Some(Key::Up),
                            SDL_SCANCODE_DOWN => Some(Key::Down),
                            SDL_SCANCODE_LEFT => Some(Key::Left),
                            SDL_SCANCODE_RIGHT => Some(Key::Right),
                            _ => None,
                        };
                        if let Some(key) = potential_key {
                            return Event::KeyDown(key);
                        }
                    }
                    SDL_QUIT => return Event::Quit,
                    _ => {}
                }
            }
        }
    }

    pub fn quit(&self) {
        unsafe {
            SDL_DestroyRenderer(self.renderer);
            SDL_DestroyWindow(self.window);
            SDL_Quit();
        }
    }
}
