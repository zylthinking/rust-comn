use std::intrinsics::transmute;
use std::sync::atomic::{
    AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicPtr, AtomicU16, AtomicU32, AtomicU64,
    AtomicUsize, Ordering,
};

use crate::{cptr, mptr};

pub trait Atomic {
    fn load(&self, order: Ordering) -> Self;
    fn store(&self, val: Self, order: Ordering);
    fn swap(&self, val: Self, order: Ordering) -> Self;
    fn fetch_add(&self, val: Self, ord: Ordering) -> Self;
    fn fetch_sub(&self, val: Self, ord: Ordering) -> Self;
    fn fetch_and(&self, val: Self, ord: Ordering) -> Self;
    fn fetch_nand(&self, val: Self, ord: Ordering) -> Self;
    fn fetch_or(&self, val: Self, ord: Ordering) -> Self;
    fn fetch_xor(&self, val: Self, ord: Ordering) -> Self;
}

#[macro_export]
macro_rules! atomic_impl {
    ($primitive:ty, $Atomic:ty) => {
        impl Atomic for $primitive {
            fn load(&self, order: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.load(order)
            }

            fn store(&self, val: Self, order: Ordering) {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.store(val, order)
            }

            fn swap(&self, val: Self, order: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.swap(val, order)
            }

            fn fetch_add(&self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_add(val, ord)
            }

            fn fetch_sub(&self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_sub(val, ord)
            }

            fn fetch_and(&self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_and(val, ord)
            }

            fn fetch_nand(&self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_nand(val, ord)
            }

            fn fetch_or(&self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_or(val, ord)
            }

            fn fetch_xor(&self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_xor(val, ord)
            }
        }
    };
}

atomic_impl!(i16, AtomicI16);
atomic_impl!(u16, AtomicU16);
atomic_impl!(i32, AtomicI32);
atomic_impl!(u32, AtomicU32);
atomic_impl!(i64, AtomicI64);
atomic_impl!(u64, AtomicU64);
atomic_impl!(isize, AtomicIsize);
atomic_impl!(usize, AtomicUsize);

pub trait AtomicPointer<T>: Sized {
    fn load(&self, order: Ordering) -> Self;
    fn store(&self, val: Self, order: Ordering);
    fn swap(&self, val: Self, order: Ordering) -> Self;

    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;

    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;
}

impl<U> AtomicPointer<U> for *const U {
    fn load(&self, order: Ordering) -> Self {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.load(order)
    }

    fn store(&self, val: Self, order: Ordering) {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.store(mptr!(val), order)
    }

    fn swap(&self, val: Self, order: Ordering) -> Self {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.swap(mptr!(val), order)
    }

    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        unsafe {
            transmute(adtomic_ptr.compare_exchange(mptr!(current), mptr!(new), success, failure))
        }
    }

    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        unsafe {
            transmute(adtomic_ptr.compare_exchange(mptr!(current), mptr!(new), success, failure))
        }
    }
}
