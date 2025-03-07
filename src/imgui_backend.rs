//! This crate provides a SDL 2 based backend platform for imgui-rs.
//!
//! A backend platform handles window/input device events and manages their
//! state.
//!
//! # Using the library
//!
//! There are three things you need to do to use this library correctly:
//!
//! 1. Initialize a `ImguiBackend` instance
//! 2. Pass events to the platform (every frame)
//! 3. Call frame preparation callback (every frame)
//!
//! For a complete example, take a look at the imgui-rs' GitHub repository.

use std::time::Instant;

use imgui::{BackendFlags, ConfigFlags, Context, Io, MouseCursor};
use fermium::{
    events::{SDL_Event, SDL_KeyboardEvent, SDL_MouseButtonEvent, SDL_KEYDOWN, SDL_MOUSEBUTTONDOWN, SDL_MOUSEBUTTONUP, SDL_MOUSEWHEEL, SDL_PRESSED, SDL_TEXTINPUT}, keyboard::SDL_Keysym, keycode::{SDLK_a, SDL_Keycode, SDL_Keymod, KMOD_LALT, KMOD_LCTRL, KMOD_LGUI, KMOD_LSHIFT, KMOD_RALT, KMOD_RCTRL, KMOD_RGUI, KMOD_RSHIFT}, mouse::{SDL_CreateSystemCursor, SDL_Cursor, SDL_GetMouseState, SDL_SetCursor, SDL_ShowCursor, SDL_SystemCursor, SDL_WarpMouseGlobal, SDL_WarpMouseInWindow, SDL_BUTTON_LEFT, SDL_BUTTON_MIDDLE, SDL_BUTTON_X2}, scancode::SDL_Scancode, video::{SDL_GetWindowSize, SDL_Window}
};

macro_rules! imgui_key_map {
    ($var: ident, $event: ident, $sdl_key: ident, $imgui_key: ident) => {
        if $event.keysym.sym == fermium::keycode::$sdl_key {
            $var = imgui::Key::$imgui_key;
        }
    };
}
/// Handle changes in the key states.
fn handle_key(io: &mut Io, event: &SDL_KeyboardEvent) {
    let mut igkey: imgui::Key = imgui::Key::ModShortcut;
    imgui_key_map!(igkey, event, SDLK_a, A);
    imgui_key_map!(igkey, event, SDLK_b, B);
    imgui_key_map!(igkey, event, SDLK_c, C);
    imgui_key_map!(igkey, event, SDLK_d, D);
    imgui_key_map!(igkey, event, SDLK_e, E);
    imgui_key_map!(igkey, event, SDLK_f, F);
    imgui_key_map!(igkey, event, SDLK_g, G);
    imgui_key_map!(igkey, event, SDLK_h, H);
    imgui_key_map!(igkey, event, SDLK_i, I);
    imgui_key_map!(igkey, event, SDLK_j, J);
    imgui_key_map!(igkey, event, SDLK_k, K);
    imgui_key_map!(igkey, event, SDLK_l, L);
    imgui_key_map!(igkey, event, SDLK_m, M);
    imgui_key_map!(igkey, event, SDLK_n, N);
    imgui_key_map!(igkey, event, SDLK_o, O);
    imgui_key_map!(igkey, event, SDLK_p, P);
    imgui_key_map!(igkey, event, SDLK_q, Q);
    imgui_key_map!(igkey, event, SDLK_r, R);
    imgui_key_map!(igkey, event, SDLK_s, S);
    imgui_key_map!(igkey, event, SDLK_t, T);
    imgui_key_map!(igkey, event, SDLK_u, U);
    imgui_key_map!(igkey, event, SDLK_v, V);
    imgui_key_map!(igkey, event, SDLK_w, W);
    imgui_key_map!(igkey, event, SDLK_x, X);
    imgui_key_map!(igkey, event, SDLK_y, Y);
    imgui_key_map!(igkey, event, SDLK_z, Z);

    imgui_key_map!(igkey, event, SDLK_KP_1, Keypad1);
    imgui_key_map!(igkey, event, SDLK_KP_2, Keypad2);
    imgui_key_map!(igkey, event, SDLK_KP_3, Keypad3);
    imgui_key_map!(igkey, event, SDLK_KP_4, Keypad4);
    imgui_key_map!(igkey, event, SDLK_KP_5, Keypad5);
    imgui_key_map!(igkey, event, SDLK_KP_6, Keypad6);
    imgui_key_map!(igkey, event, SDLK_KP_7, Keypad7);
    imgui_key_map!(igkey, event, SDLK_KP_8, Keypad8);
    imgui_key_map!(igkey, event, SDLK_KP_9, Keypad9);
    imgui_key_map!(igkey, event, SDLK_KP_0, Keypad0);

    imgui_key_map!(igkey, event, SDLK_KP_DIVIDE, KeypadDivide);
    imgui_key_map!(igkey, event, SDLK_KP_DECIMAL, KeypadDecimal);
    imgui_key_map!(igkey, event, SDLK_KP_ENTER, KeypadEnter);
    imgui_key_map!(igkey, event, SDLK_KP_PLUS, KeypadAdd);
    imgui_key_map!(igkey, event, SDLK_KP_MULTIPLY, KeypadMultiply);
    imgui_key_map!(igkey, event, SDLK_KP_MINUS, KeypadSubtract);


    imgui_key_map!(igkey, event, SDLK_RETURN, Enter);
    imgui_key_map!(igkey, event, SDLK_ESCAPE, Escape);
    imgui_key_map!(igkey, event, SDLK_BACKSPACE, Backspace);
    imgui_key_map!(igkey, event, SDLK_TAB, Tab);
    imgui_key_map!(igkey, event, SDLK_SPACE, Space);
    imgui_key_map!(igkey, event, SDLK_MINUS, Minus);
    imgui_key_map!(igkey, event, SDLK_EQUALS, Equal);

    imgui_key_map!(igkey, event, SDLK_LEFTBRACKET, LeftBracket);
    imgui_key_map!(igkey, event, SDLK_RIGHTBRACKET, RightBracket);
    imgui_key_map!(igkey, event, SDLK_BACKSLASH, Backslash);

    imgui_key_map!(igkey, event, SDLK_SEMICOLON, Semicolon);
    imgui_key_map!(igkey, event, SDLK_QUOTE, Apostrophe);
    imgui_key_map!(igkey, event, SDLK_COMMA, Comma);

    imgui_key_map!(igkey, event, SDLK_PERIOD, Period);
    imgui_key_map!(igkey, event, SDLK_SLASH, Slash);

    imgui_key_map!(igkey, event, SDLK_CAPSLOCK, CapsLock);

    imgui_key_map!(igkey, event, SDLK_F1, F1);
    imgui_key_map!(igkey, event, SDLK_F2, F2);
    imgui_key_map!(igkey, event, SDLK_F3, F3);
    imgui_key_map!(igkey, event, SDLK_F4, F4);
    imgui_key_map!(igkey, event, SDLK_F5, F5);
    imgui_key_map!(igkey, event, SDLK_F6, F6);
    imgui_key_map!(igkey, event, SDLK_F7, F7);
    imgui_key_map!(igkey, event, SDLK_F8, F8);
    imgui_key_map!(igkey, event, SDLK_F9, F9);
    imgui_key_map!(igkey, event, SDLK_F10, F10);
    imgui_key_map!(igkey, event, SDLK_F11, F11);
    imgui_key_map!(igkey, event, SDLK_F12, F12);

    imgui_key_map!(igkey, event, SDLK_INSERT, Insert);
    imgui_key_map!(igkey, event, SDLK_HOME, Home);
    imgui_key_map!(igkey, event, SDLK_PAGEUP, PageUp);
    imgui_key_map!(igkey, event, SDLK_PAGEDOWN, PageDown);
    imgui_key_map!(igkey, event, SDLK_DELETE, Delete);
    imgui_key_map!(igkey, event, SDLK_END, End);

    imgui_key_map!(igkey, event, SDLK_UP, UpArrow);
    imgui_key_map!(igkey, event, SDLK_DOWN, DownArrow);
    imgui_key_map!(igkey, event, SDLK_LEFT, LeftArrow);
    imgui_key_map!(igkey, event, SDLK_RIGHT, RightArrow);

    imgui_key_map!(igkey, event, SDLK_LCTRL, LeftCtrl);
    imgui_key_map!(igkey, event, SDLK_RCTRL, RightCtrl);
    imgui_key_map!(igkey, event, SDLK_LSHIFT, LeftShift);
    imgui_key_map!(igkey, event, SDLK_RSHIFT, RightShift);
    imgui_key_map!(igkey, event, SDLK_LALT, LeftAlt);
    imgui_key_map!(igkey, event, SDLK_RALT, RightAlt);
    if igkey != imgui::Key::ModShortcut {
        io.add_key_event(igkey, (event.state | SDL_PRESSED) == event.state);
    }
    
}

/// Handle changes in the key modifier states.
fn handle_key_modifier(io: &mut Io, event: &SDL_KeyboardEvent) {
    let keymod = event.keysym.mod_;
    io.add_key_event(
        imgui::Key::ModShift,
        keymod | (KMOD_LSHIFT.0 as u16 | KMOD_RSHIFT.0 as u16) == keymod,
    );
    io.add_key_event(
        imgui::Key::ModCtrl,
        keymod | (KMOD_LCTRL.0 as u16 | KMOD_RCTRL.0 as u16) == keymod
    );
    io.add_key_event(
        imgui::Key::ModAlt,
        keymod | (KMOD_LALT.0 as u16 | KMOD_RALT.0 as u16) == keymod,
    );
    io.add_key_event(
        imgui::Key::ModSuper,
        keymod | (KMOD_LGUI.0 as u16 | KMOD_RGUI.0 as u16) == keymod,
    );
}

/// Map an imgui::MouseCursor to an equivalent sdl2::mouse::SystemCursor.
fn to_sdl_cursor(cursor: MouseCursor) -> SDL_SystemCursor {
    match cursor {
        MouseCursor::Arrow => fermium::mouse::SDL_SYSTEM_CURSOR_ARROW,
        MouseCursor::TextInput => fermium::mouse::SDL_SYSTEM_CURSOR_IBEAM,
        MouseCursor::ResizeAll => fermium::mouse::SDL_SYSTEM_CURSOR_SIZEALL,
        MouseCursor::ResizeNS => fermium::mouse::SDL_SYSTEM_CURSOR_SIZENS,
        MouseCursor::ResizeEW => fermium::mouse::SDL_SYSTEM_CURSOR_SIZEWE,
        MouseCursor::ResizeNESW => fermium::mouse::SDL_SYSTEM_CURSOR_SIZENESW,
        MouseCursor::ResizeNWSE => fermium::mouse::SDL_SYSTEM_CURSOR_SIZENWSE,
        MouseCursor::Hand => fermium::mouse::SDL_SYSTEM_CURSOR_HAND,
        MouseCursor::NotAllowed => fermium::mouse::SDL_SYSTEM_CURSOR_NO,
    }
}

pub struct ImguiBackend {
    cursor_instance: Option<*mut SDL_Cursor>, /* to avoid dropping cursor instances */
    last_frame: Instant,
}

impl ImguiBackend {

    pub fn init(imgui: &mut Context) -> ImguiBackend {
        let io = imgui.io_mut();

        io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
        io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);

        imgui.set_platform_name(Some(format!(
            "imgui-sdl2-support {}",
            env!("CARGO_PKG_VERSION")
        )));

        ImguiBackend {
            cursor_instance: None,
            last_frame: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, context: &mut Context, event: &SDL_Event) -> bool {
        let io = context.io_mut();
        unsafe {
            match event.type_ {
                SDL_MOUSEWHEEL  => {
                    io.add_mouse_wheel_event([ event.wheel.x as f32, event.wheel.y as f32]);
                    true
                }
    
                SDL_MOUSEBUTTONDOWN => {
                    self.handle_mouse_button(io, &event.button, true);
                    true
                }
    
                SDL_MOUSEBUTTONUP => {
                    self.handle_mouse_button(io, &event.button, false);
                    true
                }
    
                SDL_TEXTINPUT => {
                    event.text.text.iter().for_each(|c| io.add_input_character((*c as u8) as char));
                    true
                }
    
                SDL_KEYDOWN => {
                    handle_key_modifier(io, &event.key);
                    handle_key(io, &event.key);
                    true
                }
    
                SDL_KEYUP => {
                    handle_key_modifier(io, &event.key);
                    handle_key(io, &event.key);

                    true
                }

                _ => false,
            }
        }
        
    }

    pub fn prepare_frame(
        &mut self,
        context: &mut Context,
        window: *mut SDL_Window,
    ) {
        let mouse_cursor = context.mouse_cursor();
        let io = context.io_mut();

        // Update delta time
        let now = Instant::now();
        io.update_delta_time(now.duration_since(self.last_frame));
        self.last_frame = now;

        let mut mouse_x: i32 = 0;
        let mut mouse_y: i32 = 0;
        unsafe {
            let mouse_state = SDL_GetMouseState(&mut mouse_x, &mut mouse_y);
        }
        let mut window_height = 0;
        let mut window_width = 0;
        unsafe {
            SDL_GetWindowSize(window, &mut window_width, &mut window_height);
        }

        // Set display size and scale here, since SDL 2 doesn't have
        // any easy way to get the scale factor, and changes in said
        // scale factor
        io.display_size = [window_width as f32, window_height as f32];
        io.display_framebuffer_scale = [
            //(window_drawable_size.0 as f32) / (window_size.0 as f32),
            //(window_drawable_size.1 as f32) / (window_size.1 as f32),
            window_width as f32,
            window_height as f32,
        ];

        // Set mouse position if requested by imgui-rs
        if io.want_set_mouse_pos {
            unsafe {
                SDL_WarpMouseInWindow(window, io.mouse_pos[0] as i32, io.mouse_pos[1] as i32)
            }
        }

        // Update mouse cursor position
        io.mouse_pos = [mouse_x as f32, mouse_y as f32];

        // Update mouse cursor icon if requested
        if !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
        {
            match mouse_cursor {
                Some(mouse_cursor) if !io.mouse_draw_cursor => {
                    unsafe {
                        let cursor = SDL_CreateSystemCursor(to_sdl_cursor(mouse_cursor));
                        SDL_SetCursor(cursor);
                        SDL_ShowCursor(true as i32);
                        self.cursor_instance = Some(cursor);
                    }
                }

                _ => {
                    unsafe {
                        SDL_ShowCursor(false as i32);
                    }
                    self.cursor_instance = None;
                }
            }
        }
    }
}

impl ImguiBackend {
    fn handle_mouse_button(
        &mut self,
        io: &mut Io,
        event: &SDL_MouseButtonEvent,
        pressed: bool,
    ) {
        match event.button as u32 {
           SDL_BUTTON_LEFT => {
                io.add_mouse_button_event(imgui::MouseButton::Left, pressed)
            }
            SDL_BUTTON_RIGHT => {
                io.add_mouse_button_event(imgui::MouseButton::Right, pressed)
            }
            SDL_BUTTON_MIDDLE => {
                io.add_mouse_button_event(imgui::MouseButton::Middle, pressed)
            }
            SDL_BUTTON_X1 => {
                io.add_mouse_button_event(imgui::MouseButton::Extra1, pressed)
            }
            SDL_BUTTON_X2 => {
                io.add_mouse_button_event(imgui::MouseButton::Extra2, pressed)
            }
            _ => {}
        }
    }
}