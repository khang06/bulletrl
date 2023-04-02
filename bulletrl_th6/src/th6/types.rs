use std::ffi::c_void;

use static_assertions::const_assert_eq;

// Low-level game stuff
bitflags::bitflags! {
    pub struct TouhouInput: u16 {
        const SHOOT = 0x1;
        const BOMB = 0x2;
        const FOCUS = 0x4;
        const PAUSE = 0x8;
        const UP = 0x10;
        const DOWN = 0x20;
        const LEFT = 0x40;
        const RIGHT = 0x80;
        const SKIP_DIALOGUE = 0x100;
        const Q = 0x200;
        const S = 0x400;
        const ENTER = 0x800;
    }
}

#[repr(C)]
pub struct CEngine {
    gap_0: [u8; 0x114],
    pub config: [u8; 0x38],
    gap_14c: [u8; 0x3C],
    pub last_state: i32,
    pub state: i32,
    gap_190: [u8; 0x348],
}
const_assert_eq!(std::mem::size_of::<CEngine>(), 0x4D8);

#[repr(C)]
pub struct CGame {
    pub visible_score: u32,
    pub score: u32,
    pub visible_score_increment: u32,
    pub high_score: u32,
    pub difficulty: u32,
    gap_14: [u8; 0x1806],
    pub lives: u8,
    pub bombs: u8,
    gap_181c: [u8; 0x4],
    pub game_over: bool,
    gap_1821: [u8; 0x213],
    pub stage: u32,
    gap_1a38: [u8; 0x48],
}
const_assert_eq!(std::mem::size_of::<CGame>(), 0x1A80);

// Game UI
#[repr(C)]
pub struct CGameUI {
    gap_0: [u8; 0x4],
    pub inner: *mut CGameUIInner,
    gap_8: [u8; 0x24],
}
const_assert_eq!(std::mem::size_of::<CGameUI>(), 0x2C);

#[repr(C)]
pub struct CGameUIInner {
    gap_0: [u8; 0x2BDC],
    pub show_results: u32,
    gap_2be0: [u8; 0x64],
}
const_assert_eq!(std::mem::size_of::<CGameUIInner>(), 0x2C44);

// Enemies
#[repr(C)]
pub struct CEnemyManager {
    gap_0: [u8; 0xED0],
    pub enemies: [CEnemy; 256],
    gap_ed6d0: [u8; 0xF1C],
}
const_assert_eq!(std::mem::size_of::<CEnemyManager>(), 0xEE5EC);

#[repr(C)]
pub struct CEnemy {
    gap_0: [u8; 0xC6C],
    pub pos: bulletrl_common::Vector2,
    pos_z: f32,
    pub size: bulletrl_common::Vector2,
    size_z: f32,
    gap_c84: [u8; 0x1CC],
    pub enemy_type: u8,
    gap_e51: u8,
    pub flags: u8,
    gap_e53: [u8; 0x75],
}
const_assert_eq!(std::mem::size_of::<CEnemy>(), 0xEC8);

// Enemy projectiles
#[repr(C)]
pub struct CEnemyBulletManager {
    gap_0: [u8; 0x5600],
    pub bullets: [CEnemyBullet; 640],
    pub lasers: [CEnemyLaser; 64],
    gap_f5c00: [u8; 0x18],
}
const_assert_eq!(std::mem::size_of::<CEnemyBulletManager>(), 0xF5C18);

#[repr(C)]
pub struct CEnemyLaser {
    gap_0: [u8; 0x220],
    pub fire_point: bulletrl_common::Vector2,
    fire_point_z: f32,
    pub angle: f32,
    pub start_offset: f32,
    pub end_offset: f32,
    pub length: f32,
    pub width: f32,
    gap_240: [u8; 0x18],
    pub active: u32,
    gap_25c: [u8; 0x14],
}
const_assert_eq!(std::mem::size_of::<CEnemyLaser>(), 0x270);

#[repr(C)]
pub struct CEnemyBullet {
    gap_0: [u8; 0x550],
    pub size: bulletrl_common::Vector2,
    size_z: f32,
    gap_55c: [u8; 0x4],
    pub pos: bulletrl_common::Vector2,
    pos_z: f32,
    gap_56c: [u8; 0x52],
    pub shot_type: i16,
    gap_5c0: [u8; 4],
}
const_assert_eq!(std::mem::size_of::<CEnemyBullet>(), 0x5C4);

// Items
#[repr(C)]
pub struct CItemManager {
    pub items: [CItem; 512],
    gap_28800: [u8; 0x14C],
}
const_assert_eq!(std::mem::size_of::<CItemManager>(), 0x2894C);

#[repr(C)]
pub struct CItem {
    gap_0: [u8; 0x110],
    pub pos: bulletrl_common::Vector2,
    pos_z: f32,
    gap_11c: [u8; 0x25],
    pub active: bool,
    gap_142: [u8; 0x2],
}
const_assert_eq!(std::mem::size_of::<CItem>(), 0x144);

// Other important stuff
#[repr(C)]
pub struct CPlayer {
    gap_0: [u8; 0x458],
    pub pos: bulletrl_common::Vector2,
    pos_z: f32,
    gap_464: [u8; 0x57C],
    pub death_state: u8,
    gap_9e1: [u8; 0x8F0F],
}
const_assert_eq!(std::mem::size_of::<CPlayer>(), 0x98F0);

#[repr(C)]
#[allow(dead_code)]
pub struct CChainManager;

#[repr(C)]
pub struct CChainNode {
    pub priority: u16,
    pub flags: u16,
    pub update: unsafe extern "cdecl" fn(*mut c_void) -> i32,
    pub init: unsafe extern "cdecl" fn(*mut c_void) -> i32,
    pub destroy: unsafe extern "cdecl" fn(*mut c_void) -> i32,
    pub back: *mut CChainNode,
    pub next: *mut CChainNode,
    unk: u32,
    pub data: *mut c_void,
}
const_assert_eq!(std::mem::size_of::<CChainNode>(), 0x20);
