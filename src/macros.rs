use std::{fmt::Display, mem::ManuallyDrop, mem::MaybeUninit};

#[macro_export]
macro_rules! suffx {
    ($($token:tt)*) => {
        $crate::_suffix!({$($token)*})
    };
}

#[macro_export]
macro_rules! nil {
    () => {
        0 as *mut _
    };
}

#[macro_export]
macro_rules! cptr {
    ($ptr:expr) => {
        ($ptr) as *const _
    };
}

#[macro_export]
macro_rules! mptr {
    ($ptr:expr) => {
        cptr!($ptr) as *mut _
    };
}

#[macro_export]
macro_rules! offset_of {
    ($type:ty, $member:ident) => {
        std::ptr::addr_of!((*(0 as *const $type)).$member) as *const _ as usize;
    };
}

#[macro_export]
macro_rules! container_of {
    ($ptr:expr, $type:ty, $member:ident) => {{
        unsafe {
            let addr = $ptr as *const _ as usize;
            let offset = $crate::offset_of!($type, $member);
            &*((addr - offset) as *const $type)
        }
    }};
}

#[derive(Debug)]
pub struct CallPos {
    pub line: u32,
    pub file: &'static str,
}

impl Display for CallPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.file, self.line)
    }
}

#[macro_export]
macro_rules! callpos {
    () => {{
        static POS: CallPos = CallPos {
            line: line!(),
            file: file!(),
        };
        &POS
    }};
}
