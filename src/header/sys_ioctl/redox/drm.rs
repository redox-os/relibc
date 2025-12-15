use alloc::vec::Vec;
use core::{mem, slice};
use drm_sys::{
    drm_get_cap, drm_mode_card_res, drm_mode_crtc, drm_mode_fb_cmd, drm_mode_fb_cmd2,
    drm_mode_get_connector, drm_mode_get_encoder, drm_mode_get_plane, drm_mode_get_plane_res,
    drm_mode_obj_get_properties, drm_set_client_cap, drm_version,
};

use crate::{
    error::{Errno, Result},
    header::errno::EINVAL,
    platform::types::*,
};

use super::{IoctlBuffer, graphics_ipc as ipc, graphics_ipc::IoctlData};

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

    unsafe fn call<T>(&self, payload: &mut T, func: u64) -> syscall::Result<usize> {
        let bytes = slice::from_raw_parts_mut(payload as *mut T as *mut u8, mem::size_of::<T>());
        redox_rt::sys::sys_call(
            self.fd as usize,
            bytes,
            syscall::CallFlags::empty(),
            &[func],
        )
    }

    unsafe fn read_write_ioctl<T: ipc::IoctlData>(
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

    unsafe fn write_ioctl<T: ipc::IoctlData>(
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
        Ok(res as c_int)
    }

    fn display_count(&self) -> Result<usize> {
        let mut cmd = ipc::DisplayCount { count: 0 };
        unsafe {
            self.call(&mut cmd, ipc::DISPLAY_COUNT)?;
        }
        Ok(cmd.count)
    }

    fn display_size(&self, display_id: usize) -> Result<(u32, u32)> {
        let mut cmd = ipc::DisplaySize {
            display_id,
            width: 0,
            height: 0,
        };
        unsafe {
            self.call(&mut cmd, ipc::DISPLAY_SIZE)?;
        }
        Ok((cmd.width, cmd.height))
    }
}

pub(super) unsafe fn ioctl(fd: c_int, func: u8, buf: IoctlBuffer) -> Result<c_int> {
    let dev = Dev::new(fd)?;
    match func {
        0x00 => dev.read_write_ioctl::<drm_version>(buf, ipc::VERSION),
        0x0C => dev.read_write_ioctl::<drm_get_cap>(buf, ipc::GET_CAP),
        0x0D => dev.write_ioctl::<drm_set_client_cap>(buf, ipc::SET_CLIENT_CAP),
        0xA0 => dev.read_write_ioctl::<drm_mode_card_res>(buf, ipc::MODE_CARD_RES),
        0xA1 => dev.read_write_ioctl::<drm_mode_crtc>(buf, ipc::MODE_GET_CRTC),
        0xA6 => dev.read_write_ioctl::<drm_mode_get_encoder>(buf, ipc::MODE_GET_ENCODER),
        0xA7 => dev.read_write_ioctl::<drm_mode_get_connector>(buf, ipc::MODE_GET_CONNECTOR),
        0xAD => dev.read_write_ioctl::<drm_mode_fb_cmd>(buf, ipc::MODE_GET_FB),
        0xB5 => dev.read_write_ioctl::<drm_mode_get_plane_res>(buf, ipc::MODE_GET_PLANE_RES),
        0xB6 => dev.read_write_ioctl::<drm_mode_get_plane>(buf, ipc::MODE_GET_PLANE),
        0xB9 => {
            dev.read_write_ioctl::<drm_mode_obj_get_properties>(buf, ipc::MODE_OBJ_GET_PROPERTIES)
        }
        0xCE => dev.read_write_ioctl::<drm_mode_fb_cmd2>(buf, ipc::MODE_GET_FB2),
        _ => {
            eprintln!("unimplemented DRM ioctl({}, 0x{:02x}, {:?})", fd, func, buf);
            Err(Errno(EINVAL))
        }
    }
}
