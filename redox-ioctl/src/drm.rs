use alloc::vec::Vec;
use core::{
    cmp,
    ffi::{c_char, c_int, c_uint},
    iter, mem, slice,
};

pub use drm_sys::{
    __kernel_size_t, drm_get_cap, drm_mode_card_res, drm_mode_create_dumb, drm_mode_crtc,
    drm_mode_destroy_dumb, drm_mode_fb_cmd, drm_mode_fb_cmd2, drm_mode_get_connector,
    drm_mode_get_encoder, drm_mode_get_plane, drm_mode_get_plane_res, drm_mode_map_dumb,
    drm_mode_modeinfo, drm_mode_obj_get_properties, drm_set_client_cap, drm_version,
};

pub const VERSION: u64 = 0;
define_ioctl_data! {
    struct drm_version, DrmVersion {
        version_major: c_int,
        version_minor: c_int,
        version_patchlevel: c_int,
        name_len: __kernel_size_t,
        name: *mut c_char [array<c_char, name_len>],
        date_len: __kernel_size_t,
        date: *mut c_char [array<c_char, date_len>],
        desc_len: __kernel_size_t,
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
        set_connectors_ptr: u64 [array<u32, count_connectors>],
        count_connectors: u32,
        crtc_id: u32,
        fb_id: u32,
        x: u32,
        y: u32,
        gamma_size: u32,
        mode_valid: u32,
        mode: drm_mode_modeinfo,
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
        modes_ptr: u64 [array<drm_mode_modeinfo, count_modes>],
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
pub const MODE_ADD_FB: u64 = 0xAE;
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

pub const MODE_RM_FB: u64 = 0xAF;
define_ioctl_data! {
    struct standin_for_uint, StandinForUint {
        inner: c_uint,
    }
}

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct standin_for_uint {
    pub inner: c_uint,
}

pub const MODE_CREATE_DUMB: u64 = 0xB2;
define_ioctl_data! {
    struct drm_mode_create_dumb, DrmModeCreateDumb {
        height: u32,
        width: u32,
        bpp: u32,
        flags: u32,
        handle: u32,
        pitch: u32,
        size: u64,
    }
}

pub const MODE_MAP_DUMB: u64 = 0xB3;
define_ioctl_data! {
    struct drm_mode_map_dumb, DrmModeMapDumb {
        handle: u32,
        pad: u32,
        offset: u64,
    }
}

pub const MODE_DESTROY_DUMB: u64 = 0xB4;
define_ioctl_data! {
    struct drm_mode_destroy_dumb, DrmModeDestroyDumb {
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
