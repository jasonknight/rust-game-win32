use engine::GameState;
use serde::{Deserialize, Serialize};
use fermium::{
    events::*,
    prelude::{
        SDL_CreateRenderer, SDL_Delay, SDL_DestroyWindow, SDL_RenderClear, SDL_RenderPresent,
        SDL_SetRenderDrawColor,
    },
    renderer::SDL_Renderer,
    stdinc::SDL_TRUE,
    syswm::{SDL_GetWindowWMInfo, SDL_SysWMinfo, SDL_SysWMinfo_union},
    version::SDL_VERSION,
    video::{
        SDL_CreateWindow, SDL_RaiseWindow, SDL_Window, SDL_WINDOW_ALLOW_HIGHDPI, SDL_WINDOW_ALWAYS_ON_TOP,
    },
    SDL_Init, SDL_Quit, SDL_INIT_EVERYTHING,
};
use std::arch::asm;

use windows_sys::Win32::{
    Graphics::Gdi::{GetDeviceCaps, ReleaseDC, VREFRESH},
    System::Performance::QueryPerformanceFrequency,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrA, SetLayeredWindowAttributes, SetWindowLongPtrA, GWL_EXSTYLE, LWA_ALPHA,
        WS_EX_LAYERED, WS_EX_TOPMOST,
    },
};

use engine::{access_global, edit_global, GameInputArc, GameStateArc};
use windows_sys::Win32::Foundation::COLORREF;
use windows_sys::Win32::Graphics::Gdi::GetDC;
use windows_sys::Win32::Media::{timeBeginPeriod, TIMERR_NOERROR};
use windows_sys::Win32::System::Performance::QueryPerformanceCounter;

use tokei::{Config, Languages, LanguageType};

use std::thread;

mod editor;
mod imgui_backend;

#[macro_use]
extern crate lazy_static;

use std::sync::{Arc, Mutex};
#[macro_use]
extern crate maplit;

use clap::Parser;
use std::{isize, path::PathBuf};

//const PI32: f32 = 3.14159265359;
const KILOBITS: usize = 1024;

/*
I don't know why we have to do it this way, but it's the only one
that works simply... so... yolo amiriight?
*/
const QUIT: i32 = fermium::events::SDL_QUIT.0;
const WINDOWEVENT: i32 = fermium::events::SDL_WINDOWEVENT.0;
const KEYDOWN: i32 = fermium::events::SDL_KEYDOWN.0;
const KEYUP: i32 = fermium::events::SDL_KEYUP.0;
const FOCUS_GAINED: u8 = fermium::video::SDL_WINDOWEVENT_FOCUS_GAINED.0;
const FOCUS_LOST: u8 = fermium::video::SDL_WINDOWEVENT_FOCUS_LOST.0;

type GameUpdateCallback<'a> = libloading::Symbol<
    'a,
    unsafe extern "C" fn(*mut SDL_Renderer, GameStateArc, GameInputArc) -> bool,
>;
type GameInitCallback<'a> =
    libloading::Symbol<'a, unsafe extern "C" fn(Arc<Mutex<GameState>>) -> bool>;
type GameInputCallback<'a> =
    libloading::Symbol<'a, unsafe extern "C" fn(SDL_Event) -> engine::GameInput>;
// Keeps track of input and the time offset from the recording start
type RecordedInput = Vec<(u128, engine::GameInput)>;
type RecordedInputArc = Arc<Mutex<RecordedInput>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RecordedGame {
    game_state_at_start: GameState,
    recorded_input: RecordedInput,
}

lazy_static! {
    static ref GAME_STATE: GameStateArc = Arc::new(Mutex::new(GameState {
        texts: vec![None, None],
        entities: vec![],
        zmap: btreemap![],
        timing_info: engine::TimingInfo {
            ..Default::default()
        },
        window: engine::Window {
            ..Default::default()
        },
        recording: false,
    }));
    static ref RECORDED_GAME_STATE: GameStateArc = Arc::new(Mutex::new(GameState {
        texts: vec![None, None],
        entities: vec![],
        zmap: btreemap![],
        timing_info: engine::TimingInfo {
            ..Default::default()
        },
        window: engine::Window {
            ..Default::default()
        },
        recording: false,
    }));
    static ref GAME_INPUT: GameInputArc = Arc::new(Mutex::new(vec![]));
    static ref RECORDED_INPUT: RecordedInputArc = Arc::new(Mutex::new(vec![]));
    static ref RECORDING_START: Arc<Mutex<Option<std::time::Instant>>> = Arc::new(Mutex::new(None));
}

fn kilobytes(v: usize) -> usize {
    v * KILOBITS
}

fn megabytes(v: usize) -> usize {
    kilobytes(v) * KILOBITS
}

fn gigabytes(v: usize) -> usize {
    kilobytes(v) * KILOBITS
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
fn rdtsc() -> u64 {
    let mut reg_eax: u32;
    let mut reg_edx: u32;

    unsafe {
        asm!("rdtsc", out("eax") reg_eax, out("edx") reg_edx);
    }

    (reg_edx as u64) << 32 | reg_eax as u64
}
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    slow: bool,
    #[arg(long)]
    fullscreen: bool,
    #[arg(long, default_value = "1024")]
    width: i32,
    #[arg(long, default_value = "768")]
    height: i32,
    #[arg(long, default_value = "64")]
    permanent_memory_size: usize,
    #[arg(long, default_value = "128")]
    transient_memory_size: usize,
    #[arg(long, default_value = "../game/target/debug/game.dll")]
    game_dll: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct TimingInfo {
    performance_count_frequency: i64,
    target_microseconds_per_frame: f32,
    last_counter: i64,
    current_cycle_count: u64,
    milliseconds_per_frame: f32,
    megacycles_per_frame: f32,
    fps: f32,
    last_cycle_count: u64,
    loop_counter: usize,
    sleep_is_granular: bool,
    elapsed: f32,
    cycles_elapsed: i64,
    work_counter: i64,
}

impl TimingInfo {
    fn update_microseconds_elapsed(&mut self) {
        self.elapsed =
            (self.work_counter - self.last_counter) as f32 / self.performance_count_frequency as f32
    }
    fn update(&mut self) {
        self.current_cycle_count = rdtsc();
        self.cycles_elapsed = self.current_cycle_count as i64 - self.last_cycle_count as i64;
        if self.cycles_elapsed < 0 {
            // This happens sometimes because, like I don't know...
            self.cycles_elapsed = 1;
        }
        self.megacycles_per_frame = (self.cycles_elapsed as f32) / (1000.0 * 1000.0); // megacylces per frame
        self.last_cycle_count = self.current_cycle_count;

        self.work_counter = get_wall_clock();
        self.update_microseconds_elapsed();
        self.milliseconds_per_frame = 1000.0 * self.elapsed;
        self.fps = 1000.0 / self.milliseconds_per_frame;
    }
    fn new(maybe_hwnd: Option<isize>) -> Self {
        let mut monitor_refresh = 60;
        unsafe {
            let sleep_is_granular = timeBeginPeriod(1) == TIMERR_NOERROR;
            if let Some(hwnd) = maybe_hwnd {
                let refresh_dc = GetDC(hwnd);
                let refresh_rate = GetDeviceCaps(refresh_dc, VREFRESH as i32);
                ReleaseDC(hwnd, refresh_dc);
                if refresh_rate > 1 {
                    monitor_refresh = refresh_rate;
                }
            }
            let update_hertz = monitor_refresh as f32 / 2.0;
            Self {
                target_microseconds_per_frame: 1.0 / update_hertz,
                last_counter: 0,
                sleep_is_granular,
                performance_count_frequency: get_performance_frequency(),
                ..Default::default()
            }
        }
    }
}

impl From<TimingInfo> for engine::TimingInfo {
    fn from(ti: TimingInfo) -> engine::TimingInfo {
        engine::TimingInfo {
            performance_count_frequency: ti.performance_count_frequency,
            target_microseconds_per_frame: ti.target_microseconds_per_frame,
            last_counter: ti.last_counter,
            current_cycle_count: ti.current_cycle_count,
            milliseconds_per_frame: ti.milliseconds_per_frame,
            megacycles_per_frame: ti.megacycles_per_frame,
            fps: ti.fps,
            last_cycle_count: ti.last_cycle_count,
            loop_counter: ti.loop_counter,
            sleep_is_granular: ti.sleep_is_granular,
            elapsed: ti.elapsed,
            cycles_elapsed: ti.cycles_elapsed,
            work_counter: ti.work_counter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Platform {
    running: bool,
    window: *mut SDL_Window,
    hwnd: Option<isize>,
    renderer: *mut SDL_Renderer,
    cli: Cli,
    timing_info: TimingInfo,
}

fn print_project_stats() -> () {
    // The paths to search. Accepts absolute, relative, and glob paths.
    let paths = &["src", "../game/src", "../engine/src"];
    // Exclude any path that contains any of these strings.
    let excluded = &["target"];
    // `Config` allows you to configure what is searched and counted.
    let config = Config::default();

    let mut languages = Languages::new();
    languages.get_statistics(paths, excluded, &config);
    let rust = &languages[&LanguageType::Rust];

    println!("Lines of code: {}", rust.code);
}

fn get_hwnd(window: *mut SDL_Window) -> Option<isize> {
    let mut info = SDL_SysWMinfo::default();
    SDL_VERSION(&mut info.version);
    if SDL_TRUE == unsafe { SDL_GetWindowWMInfo(window, &mut info) } {
        unsafe {
            match info.info {
                SDL_SysWMinfo_union { win } => {
                    return Some(win.window as isize);
                }
                _ => {
                    return None;
                }
            }
        }
    }
    None
}

fn set_layered_attributes(platform: &Platform, color_key: COLORREF, alpha: u8) -> bool {
    unsafe {
        match platform.hwnd {
            Some(hwnd) => {
                let result = SetWindowLongPtrA(
                    hwnd,
                    GWL_EXSTYLE,
                    GetWindowLongPtrA(hwnd, GWL_EXSTYLE)
                        | WS_EX_LAYERED as isize
                        | WS_EX_TOPMOST as isize,
                );
                if result == 0 {
                    println!("Failed to SetWindowLongA");
                }
                let result = SetLayeredWindowAttributes(hwnd, color_key, alpha, LWA_ALPHA);
                SDL_RaiseWindow(platform.window);
                return result > 0;
            }
            None => {
                return false;
            }
        }
    }
}

fn get_performance_frequency() -> i64 {
    let mut result: i64 = 0;
    unsafe {
        QueryPerformanceFrequency(&mut result);
    }
    result
}
// This returns microseconds
fn get_wall_clock() -> i64 {
    let mut result: i64 = 0;
    unsafe {
        QueryPerformanceCounter(&mut result);
    }
    result
}

fn handle_sdl_events(platform: &mut Platform, event: SDL_Event, callback: &GameInputCallback) -> () {
    unsafe {
        match event.type_.0 {
            // Look into why matching on the constant doesn't actually
            // work. Or only works for some types and not others,
            // I've had to copy in the constants to make this work, and I don't know why
            QUIT => {
                // SDL_QUIT
                platform.running = false;
            }
            KEYDOWN => {
                // SDL_KEYDOWN
                let input = callback(event.clone());
                edit_global!(game_input, GAME_INPUT, {
                    game_input.push(input.clone());
                    access_global!(recording, RECORDING_START, {
                        if let Some(instant) = recording {
                            edit_global!(recorded_input, RECORDED_INPUT, {
                                recorded_input.push((instant.elapsed().as_millis(), input.clone()));
                            });
                        }
                    });
                });
            }
            KEYUP => {
                // SDL_KEYUP
                let input = callback(event.clone());
                edit_global!(game_input, GAME_INPUT, {
                    game_input.push(input.clone());
                    access_global!(recording, RECORDING_START, {
                        if let Some(instant) = recording {
                            edit_global!(recorded_input, RECORDED_INPUT, {
                                recorded_input.push((instant.elapsed().as_millis(), input.clone()));
                            });
                        }
                    });
                });
            }
            WINDOWEVENT => {
                // SDL_WINDOWEVENT
                // Same issue here, we can't match on the constant, but
                // have to destructure and match the u8 value?
                match event.window.event.0 {
                    // SDL_WINDOW_EVENT_FOCUS_GAINED
                    FOCUS_GAINED => {
                        if !set_layered_attributes(platform, 0, 255) {
                            println!("failed to set layered attributes on focus gained");
                        }
                    }
                    // SDL_WINDOW_EVENT_FOCUS_LOST
                    FOCUS_LOST => {
                        if !set_layered_attributes(platform, 0, 64) {
                            println!("Failed to set layered attributes on loss of focus");
                        }
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    print_project_stats();
    
    unsafe {
        assert_eq!(SDL_Init(SDL_INIT_EVERYTHING), 0);
        let editor_handler = thread::spawn(|| {
            editor::spawn_window();
        });

        let window = SDL_CreateWindow(
            b"Circuit Mage\0".as_ptr().cast(),
            100,
            100,
            cli.width,
            cli.height,
            // The following is key for the transparent window trick to work
            (SDL_WINDOW_ALLOW_HIGHDPI | SDL_WINDOW_ALWAYS_ON_TOP).0,
        );
        // Panic if window is not null
        assert!(!window.is_null());

        let renderer = SDL_CreateRenderer(window, -1, 1);
        // Panic if renderer is not null
        assert!(!renderer.is_null());
        SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
        let mut platform = Platform {
            running: true,
            window: window,
            hwnd: get_hwnd(window),
            timing_info: TimingInfo::new(get_hwnd(window)),
            renderer,
            cli: cli.clone(),
        };
        edit_global!(game_state, GAME_STATE, {
            game_state.window.width = cli.width as usize;
            game_state.window.height = cli.height as usize;
        });

        // TODO(jason): This needs some checking and graceful failure with helpful message
        let dll_source = cli.game_dll.as_os_str();
        let tmp = dll_source.to_str().unwrap();
        let dll_dest = tmp.replace(".dll", "_temp.dll");
        
        std::fs::copy(dll_source, dll_dest.clone()).unwrap();
        let mut lib = libloading::Library::new(dll_dest.clone()).unwrap();
        let mut game: GameUpdateCallback = lib.get("update_and_render".as_bytes()).unwrap();
        let game_init: GameInitCallback = lib.get("init".as_bytes()).unwrap(); // we don't reload this
        let mut game_decide_input: GameInputCallback = lib.get("decide_input".as_bytes()).unwrap();
        let mut dll_modified_time = std::fs::metadata(dll_source)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut event = SDL_Event::default();

        platform.timing_info.last_cycle_count = rdtsc();
        platform.timing_info.last_counter = get_wall_clock();
        platform.timing_info.loop_counter = 0;
        platform.timing_info.update();
        edit_global!(game_state, GAME_STATE, {
            game_state.timing_info = platform.timing_info.clone().into();
        });
        game_init(Arc::clone(&GAME_STATE));

        while platform.running {
            while SDL_PollEvent(&mut event) == 1 {
                handle_sdl_events(&mut platform, event.clone(), &game_decide_input);
            }
            SDL_RenderClear(platform.renderer);
            game(
                platform.renderer,
                Arc::clone(&GAME_STATE),
                Arc::clone(&GAME_INPUT),
            );
            SDL_RenderPresent(platform.renderer);
            let new_dll_modified_time = std::fs::metadata(dll_source)
                .unwrap()
                .modified()
                .unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if new_dll_modified_time > dll_modified_time {
                lib.close().unwrap();
                std::fs::copy(dll_source, dll_dest.clone()).unwrap();
                lib = libloading::Library::new(dll_dest.clone()).unwrap();
                game = lib.get("update_and_render".as_bytes()).unwrap();
                game_decide_input = lib.get("decide_input".as_bytes()).unwrap();
                dll_modified_time = new_dll_modified_time;
            }
            platform.timing_info.update();
            // This will always show as the previous frame in the output, not the current frame, because we aren't done with it.
            if platform.timing_info.loop_counter % 10 == 0 {
                edit_global!(game_state, GAME_STATE, {
                    game_state.texts[1] = Some(format!(
                        "{:.2}ms/f,  {:.1}f/s,  {:.2}mc/f, z={:.2}",
                        platform.timing_info.milliseconds_per_frame,
                        platform.timing_info.fps,
                        platform.timing_info.megacycles_per_frame,
                        game_state.entities[0].position.z
                    ));
                });
            }

            //platform.game_state.texts[1] = Some("1.345".to_string());
            // We are targeting here at least 60fps, so generally we expect each frame to take 16ms But we are okay with rendering
            // at a higher framerate,
            if platform.timing_info.elapsed < platform.timing_info.target_microseconds_per_frame {
                if platform.timing_info.sleep_is_granular {
                    let mut sleep_milliseconds = 1000.0
                        * (platform.timing_info.target_microseconds_per_frame
                            - platform.timing_info.elapsed);
                    if sleep_milliseconds < 0.0 {
                        sleep_milliseconds = 0.0;
                    }
                    //sleep_milliseconds = 1000.0;
                    SDL_Delay(sleep_milliseconds as u32);
                } else {
                    // We can't rely on sleep_milliseconds, so brute force it
                    while platform.timing_info.elapsed
                        < platform.timing_info.target_microseconds_per_frame
                    {
                        platform.timing_info.update();
                    }
                }
            }
            platform.timing_info.last_counter = get_wall_clock();
            platform.timing_info.loop_counter += 1;
            if platform.timing_info.loop_counter > 1000 {
                platform.timing_info.loop_counter = 0;
            }
            edit_global!(game_state, GAME_STATE, {
                game_state.timing_info = platform.timing_info.clone().into();
            });
        }
        editor_handler.join().unwrap();
        SDL_DestroyWindow(platform.window);
        SDL_Quit();
    }
}
