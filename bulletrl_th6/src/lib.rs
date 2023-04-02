#![feature(abi_thiscall)]
#![feature(let_chains)]

use log::info;
use windows::{
    s,
    Win32::{
        System::SystemServices::DLL_PROCESS_ATTACH,
        UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR},
    },
};

#[cfg(feature = "console")]
use windows::Win32::System::Console::{
    AllocConsole, GetConsoleMode, GetStdHandle, SetConsoleMode, SetConsoleTitleA, CONSOLE_MODE,
    ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
};

mod th6;

#[no_mangle]
pub extern "system" fn dummy_export() {}

#[no_mangle]
pub extern "system" fn DllMain(_instance: usize, reason: u32, _reserved: usize) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        init();
    }
    1
}

#[cfg(feature = "console")]
fn init_console() {
    // Spawn a console with ANSI support
    unsafe {
        AllocConsole();
        let console = GetStdHandle(STD_OUTPUT_HANDLE).expect("getting console handle");

        let mut mode = CONSOLE_MODE::default();
        GetConsoleMode(console, &mut mode).expect("getting console mode");

        mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        SetConsoleMode(console, mode).expect("setting console mode");

        SetConsoleTitleA(s!("bulletrl"));
    }

    // Initialize Rust's logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
}

fn init() {
    // Set up panic handling as soon as possible!!!
    std::panic::set_hook(Box::new(|info| unsafe {
        println!("OH SHIT!!!!");
        MessageBoxA(
            None,
            windows::core::PCSTR(
                format!("Something has gone horribly wrong!\n\n{}\0", info).as_ptr(),
            ),
            s!("bulletrl error"),
            MB_ICONERROR,
        );
        std::process::abort();
    }));

    #[cfg(feature = "console")]
    init_console();

    info!("Hi!");

    unsafe {
        th6::offsets::init_offsets();
        th6::patch::apply_patches();
        th6::patch::install_hooks().expect("installing hooks");
    }
}
