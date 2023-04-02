use std::ffi::c_void;

use log::info;
use quick_detour::make_hook;
use windows::Win32::System::Memory::{
    VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
};

use crate::th6::{
    offsets::{self, ENGINE},
    types::CEngine,
};

unsafe fn apply_patch(name: &str, ptr: *const c_void, bytes: &[u8]) {
    info!("Applying patch: {} ({:p})", name, ptr);

    let mut old_prot = PAGE_PROTECTION_FLAGS::default();
    VirtualProtect(ptr, bytes.len(), PAGE_EXECUTE_READWRITE, &mut old_prot)
        .ok()
        .expect("setting page as rwx");
    std::ptr::copy(bytes.as_ptr(), ptr as _, bytes.len());
    VirtualProtect(ptr, bytes.len(), old_prot, &mut old_prot)
        .ok()
        .expect("restoring page permissions");
}

pub unsafe fn apply_patches() {
    // Makes it easier to view all of the games at once
    apply_patch(
        "Resizable window",
        offsets::RESIZABLE_WINDOW_PATCH_ADDR,
        &[0x68, 0x00, 0x00, 0x8F, 0x10], // push 100A0000
    );

    // Allows running all of the games at once and is just nice to have in general
    apply_patch(
        "Run without window focused",
        offsets::FOCUS_PATCH_ADDR,
        &[0xEB], // jmp
    );
}

pub unsafe fn apply_training_patches() {
    // Disable built-in frame limiter
    apply_patch(
        "Disable frame limiter",
        offsets::DISABLE_FRAME_LIMITER_PATCH_ADDR,
        // jmp 0x420990
        &[0xE9, 0x9E, 0x00, 0x00, 0x00],
    );

    // The "no input" timer for the title screen to load the demo replay after a while is still active, even if the player is already in-game
    // If it loads, it crashes the game
    apply_patch(
        "Don't load demo replay",
        offsets::DONT_LOAD_DEMO_REPLAY_PATCH_ADDR,
        &[0xEB], // jmp
    );

    // Prevent the game from randomly slowing down
    apply_patch(
        "Prevent random slowdown",
        offsets::PREVENT_SLOWDOWN_PATCH_ADDR,
        // mov dword ptr ds:[6C6EC0], 0x3F800000
        // jmp 0x420B2A
        &[
            0xC7, 0x05, 0xC0, 0x6E, 0x6C, 0x00, 0x00, 0x00, 0x80, 0x3F, 0xE9, 0xFA, 0x00, 0x00,
            0x00,
        ],
    );

    // Disable Direct3D's frame limiter
    apply_patch(
        "Use D3DPRESENT_INTERVAL_IMMEDIATE",
        offsets::PRESENT_INTERVAL_PATCH_ADDR,
        // mov dword ptr ss:[ebp-18], 0x80000000
        &[0xC7, 0x45, 0xE8, 0x00, 0x00, 0x00, 0x80],
    );
}

unsafe fn log_hook(name: &str) {
    info!("Installing hook: {}", name);
}

pub unsafe fn install_hooks() -> Result<(), i32> {
    // Initialize MinHook
    assert_eq!(minhook_sys::MH_Initialize(), minhook_sys::MH_OK);

    // Needed because some static constructors have to run before being able to safely call game functions
    // It runs really close to the start of WinMain, so further patching can still happen
    log_hook("Post-injection code entrypoint (check_if_already_running)");
    make_hook!(
        std::mem::transmute(offsets::CHECK_IF_ALREADY_RUNNING),
        unsafe extern "cdecl" fn() -> i32,
        |_hook| -> i32 {
            super::post_inject_init();

            // We want multiple games running at the same time anyway, so the original function won't be called
            0
        }
    )?;

    // Allow the agent to send inputs
    log_hook("Intercept input polling (get_input)");
    make_hook!(
        std::mem::transmute(offsets::GET_INPUT),
        unsafe extern "cdecl" fn() -> u16,
        |orig| -> u16 { super::get_input().unwrap_or_else(|| orig()) }
    )?;

    Ok(())
}

pub unsafe fn install_training_hooks() -> Result<(), i32> {
    // Allows randomizing stage, difficulty, etc
    log_hook("Modify game settings (init_game_chains)");
    make_hook!(
        std::mem::transmute(offsets::INIT_GAME_CHAINS),
        unsafe extern "cdecl" fn() -> i32,
        |orig| -> i32 {
            super::modify_game_settings();
            orig()
        }
    )?;

    // The config file and score.dat can get corrupted when there's multiple processes trying to write to them at once
    log_hook("Block file writes (write_to_file)");
    make_hook!(
        std::mem::transmute(offsets::WRITE_TO_FILE),
        unsafe extern "cdecl" fn(*const u8, *mut c_void, u32) -> i32,
        |_orig, _filename, _buffer, _size| -> i32 { 0 }
    )?;

    // This makes it so that you don't have to mash through a bunch of menus when training
    log_hook("Skip the title screen (init_engine_chains)");
    make_hook!(
        std::mem::transmute(offsets::INIT_ENGINE_CHAINS),
        unsafe extern "cdecl" fn() -> i32,
        |orig| -> i32 {
            let ret = orig();
            (*ENGINE).last_state = 1;
            (*ENGINE).state = 2;
            ret
        }
    )?;

    // Standardized config for training
    // Minimum graphics, no audio, no joypad, etc
    log_hook("Load standard config (CEngine::LoadConfig)");
    make_hook!(
        std::mem::transmute(offsets::CENGINE_LOADCONFIG),
        unsafe extern "thiscall" fn(*mut CEngine, *const u8) -> i32,
        |_hook, engine, _filename| -> i32 {
            let config = include_bytes!("../../training.cfg");
            assert_eq!(config.len(), 0x38);
            (*engine).config[..].clone_from_slice(config);
            0
        }
    )?;

    Ok(())
}
