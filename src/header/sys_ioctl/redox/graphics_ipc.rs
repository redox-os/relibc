// Adapted from graphics-ipc v2 ipc module, cannot use as it needs libstd
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Damage {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

use alloc::vec::Vec;
use core::{
    cmp,
    ffi::{c_char, c_int},
    iter, mem, slice,
};

pub trait IoctlData {
    unsafe fn write(&self) -> Vec<u8>;
    unsafe fn read_from(&mut self, buf: &[u8]);
}

macro_rules! define_ioctl_data {
        (struct $ioctl_ty:ident, $mem_ty:ident {
            $($rest:tt)*
        }) => {
            define_ioctl_data!(
                struct $ioctl_ty, $mem_ty { $($rest)* } => (), (), ()
            );
        };
        (struct $ioctl_ty:ident, $mem_ty:ident {
            $field:ident: $ty:ty,
            $($rest:tt)*
        } =>
            ($($ioctl_fields:tt)*),
            ($($counted_fields:tt)*),
            ($($noncounted_fields:tt)*)
        ) => {
            define_ioctl_data!(
                struct $ioctl_ty, $mem_ty { $($rest)* } =>
                    ($($ioctl_fields)* $field: $ty,),
                    ($($counted_fields)*),
                    ($($noncounted_fields)* $field: $ty,)
            );
        };
        (struct $ioctl_ty:ident, $mem_ty:ident {
            $field:ident: $ty:ty [array<$el:ty, $counted_by:ident>],
            $($rest:tt)*
        } =>
            ($($ioctl_fields:tt)*),
            ($($counted_fields:tt)*),
            ($($noncounted_fields:tt)*)
        ) => {
            define_ioctl_data!(
                struct $ioctl_ty, $mem_ty { $($rest)* } =>
                    ($($ioctl_fields)* $field: $ty,),
                    ($($counted_fields)* $field: $ty [array<$el, $counted_by>],),
                    ($($noncounted_fields)*)
            );
        };
        (struct $ioctl_ty:ident, $mem_ty:ident {} =>
            ($($ioctl_field:ident: $ioctl_field_ty:ty,)*),
            ($($counted_field:ident: $counted_ty:ty [array<$el:ty, $counted_by:ident>],)*),
            ($($noncounted_field:ident: $noncounted_ty:ty,)*)
        ) => {
            pub use drm_sys::$ioctl_ty;

            // FIXME check ioctl_ty doesn't have padding
            const _: $ioctl_ty = $ioctl_ty {
                $($ioctl_field: unsafe { mem::zeroed::<$ioctl_field_ty>() },)*
            };

            #[repr(C)]
            pub struct ${concat(__, $mem_ty, Noncounted)} {
                $($noncounted_field: $noncounted_ty,)*
            }

            pub struct $mem_ty<'a> {
                noncounted_fields: &'a mut ${concat(__, $mem_ty, Noncounted)},
                $($counted_field: &'a mut [$el],)*
            }

            impl IoctlData for $ioctl_ty {
                unsafe fn write(&self) -> Vec<u8> {
                    let noncounted_fields = ${concat(__, $mem_ty, Noncounted)} {
                        $($noncounted_field: self.$noncounted_field,)*
                    };
                    // FIXME use Vec::with_capacity
                    let mut data = Vec::<u8>::new();
                    data.extend_from_slice(&unsafe {
                        mem::transmute::<
                            ${concat(__, $mem_ty, Noncounted)},
                            [u8; size_of::<${concat(__, $mem_ty, Noncounted)}>()],
                        >(noncounted_fields)
                    });
                    $(
                        let size = self.$counted_by as usize * size_of::<$el>();
                        if self.$counted_field as usize != 0 {
                            let $counted_field = unsafe {
                                slice::from_raw_parts(self.$counted_field as *const u8, size)
                            };
                            data.extend_from_slice(&$counted_field);
                        } else {
                            data.extend(iter::repeat(0u8).take(size));
                        };

                    )*
                    data
                }

                unsafe fn read_from(&mut self, mut buf: &[u8]) {
                    // FIXME be robust against malicious scheme implementations by returning an error
                    // when the buf is the wrong size
                    let noncounted_fields = buf.split_off(..size_of::<${concat(__, $mem_ty, Noncounted)}>()).unwrap();

                    $(
                        let size = self.$counted_by as usize * size_of::<$el>();
                        let $counted_field = buf.split_off(..size).unwrap();
                        if self.$counted_field as usize != 0 {
                            unsafe {
                                slice::from_raw_parts_mut(self.$counted_field as *mut u8, size).copy_from_slice($counted_field);
                            }
                        }
                    )*

                    assert!(buf.is_empty());

                    let noncounted_fields = unsafe { &*(noncounted_fields as *const _ as *const ${concat(__, $mem_ty, Noncounted)}) };
                    $(self.$noncounted_field = noncounted_fields.$noncounted_field;)*
                }
            }

            impl<'a> $mem_ty<'a> {
                pub fn with(
                    mut buf: &'a mut [u8],
                    f: impl FnOnce($mem_ty<'a>) -> syscall::Result<usize>,
                ) -> syscall::Result<usize> {
                    let noncounted_fields = buf.split_off_mut(..size_of::<${concat(__, $mem_ty, Noncounted)}>())
                        .ok_or(syscall::Error::new(syscall::EINVAL))?;
                    let noncounted_fields = unsafe { &mut *(noncounted_fields as *mut _ as *mut ${concat(__, $mem_ty, Noncounted)}) };

                    $(
                        let $counted_field = buf.split_off_mut(..noncounted_fields.$counted_by as usize * size_of::<$el>())
                            .ok_or(syscall::Error::new(syscall::EINVAL))?;
                        let $counted_field = unsafe {
                            slice::from_raw_parts_mut($counted_field as *mut _ as *mut $el, noncounted_fields.$counted_by as usize)
                        };
                    )*

                    if !buf.is_empty() {
                        return Err(syscall::Error::new(syscall::EINVAL));
                    }



                    Ok( f($mem_ty {
                        noncounted_fields,
                        $($counted_field,)*
                    })?)
                }

                $(
                    pub fn $noncounted_field(&self) -> $noncounted_ty {
                        self.noncounted_fields.$noncounted_field
                    }

                    /// Should not be called for fields used as array length
                    pub fn ${concat(set_, $noncounted_field)}(&mut self, data: $noncounted_ty) {
                        self.noncounted_fields.$noncounted_field = data;
                    }
                )*

                $(
                    pub fn $counted_field(&self) -> &[$el] {
                        self.$counted_field
                    }

                    pub fn ${concat(set_, $counted_field)}(&mut self, data: &[$el]) {
                        let copied_count = cmp::min(data.len(), self.$counted_field.len());
                        self.$counted_field[..copied_count].copy_from_slice(&data[..copied_count]);
                        self.noncounted_fields.$counted_by = data.len() as _;
                    }
                )*
            }
        };
    }

pub const VERSION: u64 = 0;
define_ioctl_data! {
    struct drm_version, DrmVersion {
        version_major: c_int,
        version_minor: c_int,
        version_patchlevel: c_int,
        name_len: drm_sys::__kernel_size_t,
        name: *mut c_char [array<c_char, name_len>],
        date_len: drm_sys::__kernel_size_t,
        date: *mut c_char [array<c_char, date_len>],
        desc_len: drm_sys::__kernel_size_t,
        desc: *mut c_char [array<c_char, desc_len>],
    }
}

pub const GET_CAP: u64 = 0x0C;
pub use drm_sys::DRM_CAP_DUMB_BUFFER;
define_ioctl_data! {
    struct drm_get_cap, DrmGetCap {
        capability: u64,
        value: u64,
    }
}

pub const SET_CLIENT_CAP: u64 = 0x0D;
pub use drm_sys::DRM_CLIENT_CAP_CURSOR_PLANE_HOTSPOT;
define_ioctl_data! {
    struct drm_set_client_cap, DrmSetClientCap {
        capability: u64,
        value: u64,
    }
}

pub const MODE_CARD_RES: u64 = 0xA0;
define_ioctl_data! {
    struct drm_mode_card_res, DrmModeCardRes {
        fb_id_ptr: u64 [array<u32, count_fbs>],
        crtc_id_ptr: u64 [array<u32, count_crtcs>],
        connector_id_ptr: u64 [array<u32, count_connectors>],
        encoder_id_ptr: u64 [array<u32, count_encoders>],
        count_fbs: u32,
        count_crtcs: u32,
        count_connectors: u32,
        count_encoders: u32,
        min_width: u32,
        max_width: u32,
        min_height: u32,
        max_height: u32,
    }
}

pub const MODE_GET_CRTC: u64 = 0xA1;
define_ioctl_data! {
    struct drm_mode_crtc, DrmModeCrtc {
        set_connectors_ptr: u64,
        count_connectors: u32,
        crtc_id: u32,
        fb_id: u32,
        x: u32,
        y: u32,
        gamma_size: u32,
        mode_valid: u32,
        mode: drm_sys::drm_mode_modeinfo,
    }
}

pub const MODE_GET_ENCODER: u64 = 0xA6;
define_ioctl_data! {
    struct drm_mode_get_encoder, DrmModeGetEncoder {
        encoder_id: u32,
        encoder_type: u32,
        crtc_id: u32,
        possible_crtcs: u32,
        possible_clones: u32,
    }
}

pub const MODE_GET_CONNECTOR: u64 = 0xA7;
define_ioctl_data! {
    struct drm_mode_get_connector, DrmModeGetConnector {
        encoders_ptr: u64 [array<u32, count_encoders>],
        modes_ptr: u64 [array<drm_sys::drm_mode_modeinfo, count_modes>],
        props_ptr: u64 [array<u32, count_props>],
        prop_values_ptr: u64 [array<u64, count_props>],
        count_modes: u32,
        count_props: u32,
        count_encoders: u32,
        encoder_id: u32,
        connector_id: u32,
        connector_type: u32,
        connector_type_id: u32,
        connection: u32,
        mm_width: u32,
        mm_height: u32,
        subpixel: u32,
        pad: u32,
    }
}

pub const MODE_GET_FB: u64 = 0xAD;
define_ioctl_data! {
    struct drm_mode_fb_cmd, DrmModeFbCmd {
        fb_id: u32,
        width: u32,
        height: u32,
        pitch: u32,
        bpp: u32,
        depth: u32,
        handle: u32,
    }
}

pub const MODE_GET_PLANE_RES: u64 = 0xB5;
define_ioctl_data! {
    struct drm_mode_get_plane_res, DrmModeGetPlaneRes {
        plane_id_ptr: u64 [array<u32, count_planes>],
        count_planes: u32,
    }
}

pub const MODE_GET_PLANE: u64 = 0xB6;
define_ioctl_data! {
    struct drm_mode_get_plane, DrmModeGetPlane {
        plane_id: u32,
        crtc_id: u32,
        fb_id: u32,
        possible_crtcs: u32,
        gamma_size: u32,
        count_format_types: u32,
        format_type_ptr: u64 [array<u32, count_format_types>],
    }
}

pub const MODE_OBJ_GET_PROPERTIES: u64 = 0xB9;
define_ioctl_data! {
    struct drm_mode_obj_get_properties, DrmModeObjGetProperties {
        props_ptr: u64 [array<u32, count_props>],
        prop_values_ptr: u64 [array<u64, count_props>],
        count_props: u32,
        obj_id: u32,
        obj_type: u32,
    }
}

pub const MODE_GET_FB2: u64 = 0xCE;
define_ioctl_data! {
    struct drm_mode_fb_cmd2, DrmModeFbCmd2 {
        fb_id: u32,
        width: u32,
        height: u32,
        pixel_format: u32,
        flags: u32,
        handles: [u32; 4],
        pitches: [u32; 4],
        offsets: [u32; 4],
        modifier: [u64; 4],
    }
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
