// Adapted from graphics-ipc v2 ipc module, cannot use as it needs libstd
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Damage {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub const VERSION: u64 = 0;
#[repr(C, packed)]
pub struct Version {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patchlevel: u32,
    // FIXME allow variable sized fields
    pub name_len: usize,
    pub name: [u8; 16],
    pub desc_len: usize,
    pub desc: [u8; 16],
}

pub const GET_CAP: u64 = 0x0C;
pub const CAP_DUMB_BUFFER: u64 = 0x1;
#[repr(C, packed)]
pub struct GetCap {
    pub capability: u64,
    pub value: u64,
}

pub const SET_CLIENT_CAP: u64 = 0x0D;
pub const CLIENT_CAP_CURSOR_PLANE_HOTSPOT: u64 = 6;
#[repr(C, packed)]
pub struct SetClientCap {
    pub capability: u64,
    pub value: u64,
}

// FIXME replace these with proper drm interfaces
pub const DISPLAY_COUNT: u64 = 1;
#[repr(C, packed)]
pub struct DisplayCount {
    pub count: usize,
}

pub const DISPLAY_SIZE: u64 = 2;
#[repr(C, packed)]
pub struct DisplaySize {
    pub display_id: usize,

    pub width: u32,
    pub height: u32,
}

pub const CREATE_DUMB_FRAMEBUFFER: u64 = 3;
#[repr(C, packed)]
pub struct CreateDumbFramebuffer {
    pub width: u32,
    pub height: u32,

    pub fb_id: usize,
}

pub const DUMB_FRAMEBUFFER_MAP_OFFSET: u64 = 4;
#[repr(C, packed)]
pub struct DumbFramebufferMapOffset {
    pub fb_id: usize,

    pub offset: usize,
}

pub const DESTROY_DUMB_FRAMEBUFFER: u64 = 5;
#[repr(C, packed)]
pub struct DestroyDumbFramebuffer {
    pub fb_id: usize,
}

pub const UPDATE_PLANE: u64 = 6;
#[repr(C, packed)]
pub struct UpdatePlane {
    pub display_id: usize,
    pub fb_id: usize,
    pub damage: Damage,
}
