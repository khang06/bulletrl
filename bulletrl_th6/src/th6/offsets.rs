use std::ffi::c_void;

use null_fn::null_fn;

use super::types::{
    CChainManager, CChainNode, CEnemyBulletManager, CEnemyManager, CEngine, CGame, CGameUI,
    CItemManager, CPlayer,
};

// Offsets are for Embodiment of Scarlet Devil v1.02h

// Function pointers that are called have to be initialized at runtime to work around this:
// error[E0080]: it is undefined behavior to use this value
//  --> src\th6\native.rs:3:1
//   |
// 3 | / pub const create_window: unsafe extern "cdecl" fn(HINSTANCE) -> HWND =
// 4 | |     unsafe { std::mem::transmute(0x00420C10) };
//   | |_______________________________________________^ type validation failed: encountered 0x00420c10, but expected a function pointer
//   |
//   = note: The rules on what exactly is undefined behavior aren't clear, so this check might be overzealous. Please open an issue on the rustc repository if you believe it should not be considered undefined behavior.
//   = note: the raw bytes of the constant (size: 4, align: 4) {
//               10 0c 42 00                                     │ ..B.
//           }

// Stuff that only gets hooked can just be defined as usual

pub const RESIZABLE_WINDOW_PATCH_ADDR: *const c_void = 0x00420D09 as _;
pub const FOCUS_PATCH_ADDR: *const c_void = 0x004206F0 as _;
pub const DISABLE_FRAME_LIMITER_PATCH_ADDR: *const c_void = 0x004208ED as _;
pub const DONT_LOAD_DEMO_REPLAY_PATCH_ADDR: *const c_void = 0x0041C143 as _;
pub const PREVENT_SLOWDOWN_PATCH_ADDR: *const c_void = 0x00420A21 as _;
pub const PRESENT_INTERVAL_PATCH_ADDR: *const c_void = 0x00420F4D as _;

pub const CHECK_IF_ALREADY_RUNNING: *const c_void = 0x00421900 as _;
pub const INIT_ENGINE_CHAINS: *const c_void = 0x0042386B as _;
pub const WRITE_TO_FILE: *const c_void = 0x0041E460 as _;
pub const CENGINE_LOADCONFIG: *const c_void = 0x0042464D as _;
pub const GET_INPUT: *const c_void = 0x0041D820 as _;

pub const CHAIN_MANAGER: *mut CChainManager = 0x0069D918 as _;
pub const ENGINE: *mut CEngine = 0x006C6D18 as _;
pub const GAME: *mut CGame = 0x0069BCA0 as _;
pub const GAME_UI: *mut CGameUI = 0x0069BC30 as _;
pub const ENEMY_MANAGER: *mut CEnemyManager = 0x004B79C8 as _;
pub const ENEMY_BULLETS: *mut CEnemyBulletManager = 0x005A5FF8 as _;
pub const ITEM_MANAGER: *mut CItemManager = 0x0069E268 as _;
pub const PLAYER: *mut CPlayer = 0x006CA628 as _;

#[null_fn]
pub static mut CREATE_CHAIN_NODE: unsafe extern "stdcall" fn(
    calc: unsafe extern "cdecl" fn(*mut c_void) -> i32,
) -> *mut CChainNode = std::ptr::null();

#[null_fn]
pub static mut ADD_CALC_CHAIN: unsafe extern "thiscall" fn(
    this: *mut CChainManager,
    node: *mut CChainNode,
    priority: i16,
) -> i32 = std::ptr::null();

#[null_fn]
pub static mut INIT_GAME_CHAINS: unsafe extern "cdecl" fn() -> i32 = std::ptr::null();

#[null_fn]
pub static mut DESTROY_GAME_CHAINS: unsafe extern "cdecl" fn() -> i32 = std::ptr::null();

pub fn init_offsets() {
    unsafe {
        CREATE_CHAIN_NODE = std::mem::transmute(0x0041CD40);
        ADD_CALC_CHAIN = std::mem::transmute(0x0041C860);
        INIT_GAME_CHAINS = std::mem::transmute(0x0041BA6A);
        DESTROY_GAME_CHAINS = std::mem::transmute(0x0041C269);
    }
}
