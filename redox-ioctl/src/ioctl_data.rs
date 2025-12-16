use alloc::vec::Vec;

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

        impl $crate::ioctl_data::IoctlData for $ioctl_ty {
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
