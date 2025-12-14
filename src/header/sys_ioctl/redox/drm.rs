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

use super::{IoctlBuffer, graphics_ipc as ipc};

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

    fn version(&self) -> Result<ipc::Version> {
        let mut cmd = ipc::Version {
            version_major: 0,
            version_minor: 0,
            version_patchlevel: 0,
            name_len: 0,
            name: [0; 16],
            desc_len: 0,
            desc: [0; 16],
        };
        unsafe {
            self.call(&mut cmd, ipc::VERSION)?;
        }
        Ok(cmd)
    }

    fn get_cap(&self, capability: u64) -> Result<u64> {
        let mut cmd = drm_get_cap {
            capability,
            value: 0,
        };
        unsafe {
            self.call(&mut cmd, ipc::GET_CAP)?;
        }
        Ok(cmd.value)
    }

    fn set_client_cap(&self, capability: u64, value: u64) -> Result<()> {
        let mut cmd = drm_set_client_cap { capability, value };
        unsafe {
            self.call(&mut cmd, ipc::SET_CLIENT_CAP)?;
        }
        Ok(())
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

unsafe fn version(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut vers = buf.read::<drm_version>()?;
    let version = dev.version()?;
    vers.version_major = version.version_major as i32;
    vers.version_minor = version.version_minor as i32;
    vers.version_patchlevel = version.version_patchlevel as i32;
    vers.name_len = copy_array(
        &version.name[..version.name_len],
        vers.name as *mut u8,
        vers.name_len,
    );
    vers.date_len = copy_array("0".as_bytes(), vers.date as *mut u8, vers.date_len);
    vers.desc_len = copy_array(
        &version.desc[..version.desc_len],
        vers.desc as *mut u8,
        vers.desc_len,
    );
    buf.write(vers)?;
    Ok(0)
}

unsafe fn get_cap(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut cap = buf.read::<drm_get_cap>()?;
    cap.value = dev.get_cap(cap.capability)?;
    buf.write(cap)?;
    Ok(0)
}

unsafe fn set_client_cap(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut cap = buf.read::<drm_set_client_cap>()?;
    dev.set_client_cap(cap.capability, cap.value)?;
    Ok(0)
}

unsafe fn mode_card_res(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut res = buf.read::<drm_mode_card_res>()?;
    let count = dev.display_count()?;
    let mut conn_ids = Vec::with_capacity(count);
    let mut crtc_ids = Vec::with_capacity(count);
    let mut enc_ids = Vec::with_capacity(count);
    let mut fb_ids = Vec::with_capacity(count);
    for i in 0..(count as u32) {
        conn_ids.push(conn_id(i));
        crtc_ids.push(crtc_id(i));
        enc_ids.push(enc_id(i));
        fb_ids.push(fb_id(i));
    }
    res.count_fbs = copy_array(&fb_ids, res.fb_id_ptr as *mut u32, res.count_fbs as usize) as u32;
    res.count_crtcs = copy_array(
        &crtc_ids,
        res.crtc_id_ptr as *mut u32,
        res.count_crtcs as usize,
    ) as u32;
    res.count_connectors = copy_array(
        &conn_ids,
        res.connector_id_ptr as *mut u32,
        res.count_connectors as usize,
    ) as u32;
    res.count_encoders = copy_array(
        &enc_ids,
        res.encoder_id_ptr as *mut u32,
        res.count_encoders as usize,
    ) as u32;
    res.min_width = 0;
    res.max_width = 16384;
    res.min_height = 0;
    res.max_height = 16384;
    buf.write(res)?;
    Ok(0)
}

unsafe fn mode_get_crtc(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut crtc = buf.read::<drm_mode_crtc>()?;
    let i = id_index(crtc.crtc_id);
    let (width, height) = dev.display_size(i as usize)?;
    //TOOD: connectors
    crtc.fb_id = fb_id(i);
    crtc.x = 0;
    crtc.y = 0;
    crtc.gamma_size = 0;
    crtc.mode_valid = 0;
    //TODO: mode
    crtc.mode = Default::default();
    buf.write(crtc)?;
    Ok(0)
}

unsafe fn mode_get_encoder(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut enc = buf.read::<drm_mode_get_encoder>()?;
    let i = id_index(enc.encoder_id);
    let (width, height) = dev.display_size(i as usize)?;
    enc.crtc_id = crtc_id(i);
    enc.possible_crtcs = (1 << i);
    enc.possible_clones = (1 << i);
    buf.write(enc)?;
    Ok(0)
}

unsafe fn mode_get_connector(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut conn = buf.read::<drm_mode_get_connector>()?;
    let i = id_index(conn.connector_id);
    let (width, height) = dev.display_size(i as usize)?;
    conn.count_modes = 0;
    conn.count_props = 0;
    conn.count_encoders = copy_array(
        &[enc_id(i)],
        conn.encoders_ptr as *mut u32,
        conn.count_encoders as usize,
    ) as u32;
    buf.write(conn)?;
    Ok(0)
}

unsafe fn mode_get_fb(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut fb = buf.read::<drm_mode_fb_cmd>()?;
    let i = id_index(fb.fb_id);
    let (width, height) = dev.display_size(i as usize)?;
    fb.width = width;
    fb.height = height;
    fb.pitch = width * 4; //TODO: stride
    fb.bpp = 32;
    fb.depth = 24;
    fb.handle = fb_handle_id(i);
    buf.write(fb)?;
    Ok(0)
}

unsafe fn mode_get_plane_res(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut res = buf.read::<drm_mode_get_plane_res>()?;
    let count = dev.display_count()?;
    let mut ids = Vec::with_capacity(count);
    for i in 0..(count as u32) {
        ids.push(plane_id(i));
    }
    res.count_planes = copy_array(
        &ids,
        res.plane_id_ptr as *mut u32,
        res.count_planes as usize,
    ) as u32;
    buf.write(res)?;
    Ok(0)
}

unsafe fn mode_get_plane(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut plane = buf.read::<drm_mode_get_plane>()?;
    let i = id_index(plane.plane_id);
    let (width, height) = dev.display_size(i as usize)?;
    plane.crtc_id = crtc_id(i);
    plane.fb_id = fb_id(i);
    plane.possible_crtcs = (1 << i);
    plane.count_format_types = copy_array(
        &[DRM_FORMAT_ARGB8888],
        plane.format_type_ptr as *mut u32,
        plane.count_format_types as usize,
    ) as u32;
    buf.write(plane)?;
    Ok(0)
}

unsafe fn mode_obj_get_properties(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut props = buf.read::<drm_mode_obj_get_properties>()?;
    //TODO
    props.count_props = 0;
    buf.write(props)?;
    Ok(0)
}

unsafe fn mode_get_fb2(dev: Dev, mut buf: IoctlBuffer) -> Result<c_int> {
    let mut fb = buf.read::<drm_mode_fb_cmd2>()?;
    let i = id_index(fb.fb_id);
    let (width, height) = dev.display_size(i as usize)?;
    fb.width = width;
    fb.height = height;
    fb.pixel_format = DRM_FORMAT_ARGB8888;
    fb.handles[0] = fb_handle_id(i);
    fb.pitches[0] = width * 4;
    fb.offsets[0] = 0;
    fb.modifier[0] = 0;
    buf.write(fb)?;
    Ok(0)
}

pub(super) unsafe fn ioctl(fd: c_int, func: u8, buf: IoctlBuffer) -> Result<c_int> {
    let dev = Dev::new(fd)?;
    match func {
        0x00 => version(dev, buf),
        0x0C => get_cap(dev, buf),
        0x0D => set_client_cap(dev, buf),
        0xA0 => mode_card_res(dev, buf),
        0xA1 => mode_get_crtc(dev, buf),
        0xA6 => mode_get_encoder(dev, buf),
        0xA7 => mode_get_connector(dev, buf),
        0xAD => mode_get_fb(dev, buf),
        0xB5 => mode_get_plane_res(dev, buf),
        0xB6 => mode_get_plane(dev, buf),
        0xB9 => mode_obj_get_properties(dev, buf),
        0xCE => mode_get_fb2(dev, buf),
        _ => {
            eprintln!("unimplemented DRM ioctl({}, 0x{:02x}, {:?})", fd, func, buf);
            Err(Errno(EINVAL))
        }
    }
}
