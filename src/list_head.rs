#![allow(unused_macros)]
#![allow(dead_code)]

pub struct ListHead {
    pub next: *mut ListHead,
    pub prev: *mut ListHead,
}

#[macro_export]
macro_rules! InitListHead {
    ($head:ident) => {
        let mut $head = $crate::ListHead::new();
        $head.init_list_head();
    };
}

#[macro_export]
macro_rules! list_entry {
    ($ptr:expr, $type:ty, $member:ident) => {
        container_of!($ptr, $type, $member)
    };
}

impl ListHead {
    #[inline]
    pub fn new() -> ListHead {
        ListHead {
            next: 0 as _,
            prev: 0 as _,
        }
    }

    #[inline]
    pub fn init_list_head(&mut self) {
        self.next = self;
        self.prev = self;
    }

    #[inline]
    unsafe fn __list_add(new: *mut ListHead, prev: *mut ListHead, next: *mut ListHead) {
        (*next).prev = new;
        (*new).next = next;
        (*new).prev = prev;
        (*prev).next = new;
    }

    #[inline]
    unsafe fn __list_del(prev: *mut ListHead, next: *mut ListHead) {
        (*next).prev = prev;
        (*prev).next = next;
    }

    #[inline]
    pub unsafe fn list_add(&mut self, new: &mut ListHead) {
        ListHead::__list_add(new, self, (*self).next);
    }

    #[inline]
    pub unsafe fn list_add_tail(&mut self, new: &mut ListHead) {
        ListHead::__list_add(new, (*self).prev, self);
    }

    #[inline]
    pub unsafe fn list_del(&mut self) {
        ListHead::__list_del((*self).prev, (*self).next);
    }

    #[inline]
    pub unsafe fn list_del_init(&mut self) {
        self.list_del();
        self.init_list_head();
    }

    #[inline]
    pub fn list_empty(&self) -> bool {
        self.next == self.prev
    }

    #[inline]
    pub fn list_is_singular(&self) -> bool {
        return !self.list_empty() && (self.next == self.prev);
    }

    #[inline]
    pub unsafe fn list_join(&mut self, other: &mut ListHead) {
        let me = &mut *self.prev;
        (*(me.next)).prev = other.prev;
        (*(other.prev)).next = me.next;
        me.next = other;
        other.prev = me;
    }

    #[inline]
    pub unsafe fn list_split(&mut self, other: &mut ListHead) {
        let entry = &mut *other.prev;
        (*(self.prev)).next = other;
        other.prev = self.prev;
        self.prev = entry;
        entry.next = self;
    }
}
