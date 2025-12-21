use core::slice;

use redox_ioctl::{IoctlData, drm::*};

use crate::{
    error::{Errno, Result},
    header::errno::EINVAL,
    platform::types::*,
};

use super::IoctlBuffer;

const DRM_FORMAT_ARGB8888: u32 = 0x34325241; // 'AR24' fourcc code, for ARGB8888

fn id_index(id: u32) -> u32 {
    id & 0xFF
}

fn conn_id(i: u32) -> u32 {
    id_index(i) | (1 << 8)
}

fn crtc_id(i: u32) -> u32 {
    id_index(i) | (1 << 9)
}

fn enc_id(i: u32) -> u32 {
    id_index(i) | (1 << 10)
}

fn fb_id(i: u32) -> u32 {
    id_index(i) | (1 << 11)
}

fn fb_handle_id(i: u32) -> u32 {
    id_index(i) | (1 << 12)
}

fn plane_id(i: u32) -> u32 {
    id_index(i) | (1 << 13)
}

unsafe fn copy_array<T: Copy>(src: &[T], dst_ptr: *mut T, dst_len: usize) -> usize {
    let dst = slice::from_raw_parts_mut(dst_ptr, dst_len);
    dst.copy_from_slice(&src[..src.len().min(dst_len)]);
    src.len()
}

struct Dev {
    fd: c_int,
}

impl Dev {
    fn new(fd: c_int) -> Result<Self> {
        //TODO: check display scheme using fpath?
        Ok(Self { fd })
    }

    unsafe fn read_write_ioctl<T: IoctlData>(
        &self,
        mut buf: IoctlBuffer,
        func: u64,
    ) -> Result<c_int> {
        let mut data = buf.read::<T>()?;
        let mut wire = data.write();
        let res = redox_rt::sys::sys_call(
            self.fd as usize,
            &mut wire,
            syscall::CallFlags::empty(),
            &[func],
        )?;
        data.read_from(&wire);
        buf.write(data)?;
        Ok(res as c_int)
    }

    unsafe fn write_ioctl<T: IoctlData>(&self, mut buf: IoctlBuffer, func: u64) -> Result<c_int> {
        let mut data = buf.read::<T>()?;
        let mut wire = data.write();
        let res = redox_rt::sys::sys_call(
            self.fd as usize,
            &mut wire,
            syscall::CallFlags::empty(),
            &[func],
        )?;
        Ok(res as c_int)
    }
}

pub(super) unsafe fn ioctl(fd: c_int, func: u8, buf: IoctlBuffer) -> Result<c_int> {
    let dev = Dev::new(fd)?;
    match func {
        0x00 => dev.read_write_ioctl::<drm_version>(buf, VERSION),
        0x0C => dev.read_write_ioctl::<drm_get_cap>(buf, GET_CAP),
        0x0D => dev.write_ioctl::<drm_set_client_cap>(buf, SET_CLIENT_CAP),
        0xA0 => dev.read_write_ioctl::<drm_mode_card_res>(buf, MODE_CARD_RES),
        0xA1 => dev.read_write_ioctl::<drm_mode_crtc>(buf, MODE_GET_CRTC),
        0xA6 => dev.read_write_ioctl::<drm_mode_get_encoder>(buf, MODE_GET_ENCODER),
        0xA7 => dev.read_write_ioctl::<drm_mode_get_connector>(buf, MODE_GET_CONNECTOR),
        0xAA => dev.read_write_ioctl::<drm_mode_get_property>(buf, MODE_GET_PROPERTY),
        0xAB => dev.read_write_ioctl::<drm_mode_connector_set_property>(buf, MODE_SET_PROPERTY),
        0xAC => dev.read_write_ioctl::<drm_mode_get_blob>(buf, MODE_GET_PROP_BLOB),
        0xAD => dev.read_write_ioctl::<drm_mode_fb_cmd>(buf, MODE_GET_FB),
        0xAE => dev.read_write_ioctl::<drm_mode_fb_cmd>(buf, MODE_ADD_FB),
        0xAF => dev.read_write_ioctl::<standin_for_uint>(buf, MODE_RM_FB),
        0xB2 => dev.read_write_ioctl::<drm_mode_create_dumb>(buf, MODE_CREATE_DUMB),
        0xB3 => dev.read_write_ioctl::<drm_mode_map_dumb>(buf, MODE_MAP_DUMB),
        0xB4 => dev.read_write_ioctl::<drm_mode_destroy_dumb>(buf, MODE_DESTROY_DUMB),
        0xB5 => dev.read_write_ioctl::<drm_mode_get_plane_res>(buf, MODE_GET_PLANE_RES),
        0xB6 => dev.read_write_ioctl::<drm_mode_get_plane>(buf, MODE_GET_PLANE),
        0xB9 => dev.read_write_ioctl::<drm_mode_obj_get_properties>(buf, MODE_OBJ_GET_PROPERTIES),
        0xCE => dev.read_write_ioctl::<drm_mode_fb_cmd2>(buf, MODE_GET_FB2),
        _ => {
            eprintln!("unimplemented DRM ioctl({}, 0x{:02x}, {:?})", fd, func, buf);
            Err(Errno(EINVAL))
        }
    }
}
