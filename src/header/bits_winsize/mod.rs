use crate::platform::types::c_ushort;

#[repr(C)]
#[derive(Default)]
pub struct winsize {
    ws_row: c_ushort,
    ws_col: c_ushort,
    ws_xpixel: c_ushort,
    ws_ypixel: c_ushort,
}

impl winsize {
    pub fn get_row_col(&self) -> (c_ushort, c_ushort) {
        (self.ws_row, self.ws_col)
    }
}
