#[macro_export]
macro_rules! nil {
    () => {
        0 as *mut _
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
            let offset = offset_of!($type, $member);
            &*((addr - offset) as *const $type)
        }
    }};
}
