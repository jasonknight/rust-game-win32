use glow::HasContext;
use imgui::Context;
#[path = "imgui_renderer.rs"]
mod imgui_renderer;
#[path = "imgui_backend.rs"]
pub mod imgui_backend;
use fermium::{
    error::SDL_GetErrorMsg, events::*, video::{SDL_GL_CreateContext, SDL_GL_GetProcAddress, SDL_GL_MakeCurrent, SDL_GL_SetSwapInterval, SDL_GL_SwapWindow, SDL_GLprofile, SDL_Window, SDL_GL_CONTEXT_PROFILE_CORE, SDL_WINDOW_OPENGL}
};
use std::ffi::CStr;

pub fn get_error() -> String {
    unsafe {
        let mut errstr: Vec<i8> = Vec::with_capacity(255);
        let finalstr = SDL_GetErrorMsg(errstr.as_mut_ptr(), 255);
        CStr::from_ptr(finalstr as *const _).to_str().unwrap().to_owned()
    }
}
// Create a new glow context.
fn glow_context(window: *mut SDL_Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| SDL_GL_GetProcAddress(s.as_ptr() as _))
    }
}
macro_rules! gl_set_attribute {
    ($attr:ident, $value:expr) => {{
        let result = unsafe { fermium::video::SDL_GL_SetAttribute(fermium::video::$attr, $value) };

        if result != 0 {
            // Panic and print the attribute that failed.
            panic!(
                "couldn't set attribute {}, {}",
                stringify!($attr),
                get_error()
            );
        }
    }};
}
macro_rules! gl_get_attribute {
    ($attr:ident) => {{
        let mut value = 0;
        let result = unsafe { fermium::video::SDL_GL_GetAttribute(fermium::video::$attr, &mut value) };
        if result != 0 {
            // Panic and print the attribute that failed.
            panic!(
                "couldn't get attribute {}, {}",
                stringify!($attr),
                get_error()
            );
        }
        value
    }};
}

pub fn spawn_window() {
    let window: *mut SDL_Window;
    unsafe {

        window = fermium::video::SDL_CreateWindow(
            b"Editor\0".as_ptr().cast(),
            100,
            100,
            200,
            500,
            // The following is key for the transparent window trick to work
            (fermium::video::SDL_WINDOW_ALLOW_HIGHDPI | fermium::video::SDL_WINDOW_ALWAYS_ON_TOP | fermium::video::SDL_WINDOW_OPENGL).0,
        );
        assert!(!window.is_null(), "Error: {}", get_error());
        gl_set_attribute!(SDL_GL_CONTEXT_MAJOR_VERSION,3);
        gl_set_attribute!(SDL_GL_CONTEXT_MINOR_VERSION,3);
        gl_set_attribute!(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE.0 as i32);
        let gl_context = SDL_GL_CreateContext(window);
        SDL_GL_MakeCurrent(window, gl_context);
        SDL_GL_SetSwapInterval(1);
    }

    /* create new glow and imgui contexts */
    let gl = glow_context(window);

    /* create context */
    let mut imgui = Context::create();

    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    /* create platform and renderer */
    let mut platform = imgui_backend::ImguiBackend::init(&mut imgui);
    let mut renderer = imgui_renderer::AutoRenderer::initialize(gl, &mut imgui).unwrap();

    /* start main loop */
    let mut event: SDL_Event = Default::default();
    'main: loop {
        unsafe {
            while SDL_PollEvent(&mut event) == 1 {
                if event.type_.0 == SDL_QUIT.0 {
                    break 'main;
                }
                platform.handle_event(&mut imgui, &event);
            }
        }
        

        /* call prepare_frame before calling imgui.new_frame() */
        platform.prepare_frame(&mut imgui, window);

        let ui = imgui.new_frame();
        /* create imgui UI here */
        ui.show_demo_window(&mut true);

        /* render */
        let draw_data = imgui.render();

        unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
        renderer.render(draw_data).unwrap();
        unsafe {
            SDL_GL_SwapWindow(window);
        }
    }
}