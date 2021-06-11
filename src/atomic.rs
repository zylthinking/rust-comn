use crate::{cptr, mptr};
use std::intrinsics::transmute;
use std::sync::atomic::{
    AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize, Ordering,
};

pub trait AtomicN {
    fn atomic_load(&self, order: Ordering) -> Self;
    fn atomic_store(&mut self, val: Self, order: Ordering);
    fn atomic_swap(&mut self, val: Self, order: Ordering) -> Self;
    fn atomic_fetch_add(&mut self, val: Self, ord: Ordering) -> Self;
    fn atomic_fetch_sub(&mut self, val: Self, ord: Ordering) -> Self;
    fn atomic_fetch_and(&mut self, val: Self, ord: Ordering) -> Self;
    fn atomic_fetch_nand(&mut self, val: Self, ord: Ordering) -> Self;
    fn atomic_fetch_or(&mut self, val: Self, ord: Ordering) -> Self;
    fn atomic_fetch_xor(&mut self, val: Self, ord: Ordering) -> Self;
}

macro_rules! atomic_n_impl {
    ($primitive:ty, $Atomic:ty) => {
        impl AtomicN for $primitive {
            fn atomic_load(&self, order: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.load(order)
            }

            fn atomic_store(&mut self, val: Self, order: Ordering) {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.store(val, order)
            }

            fn atomic_swap(&mut self, val: Self, order: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.swap(val, order)
            }

            fn atomic_fetch_add(&mut self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_add(val, ord)
            }

            fn atomic_fetch_sub(&mut self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_sub(val, ord)
            }

            fn atomic_fetch_and(&mut self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_and(val, ord)
            }

            fn atomic_fetch_nand(&mut self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_nand(val, ord)
            }

            fn atomic_fetch_or(&mut self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_or(val, ord)
            }

            fn atomic_fetch_xor(&mut self, val: Self, ord: Ordering) -> Self {
                let adtomic_ptr: &$Atomic = unsafe { transmute(self) };
                adtomic_ptr.fetch_xor(val, ord)
            }
        }
    };
}

atomic_n_impl!(i8, AtomicI8);
atomic_n_impl!(u8, AtomicU8);
atomic_n_impl!(i16, AtomicI16);
atomic_n_impl!(u16, AtomicU16);
atomic_n_impl!(i32, AtomicI32);
atomic_n_impl!(u32, AtomicU32);
atomic_n_impl!(i64, AtomicI64);
atomic_n_impl!(u64, AtomicU64);
atomic_n_impl!(isize, AtomicIsize);
atomic_n_impl!(usize, AtomicUsize);

pub trait AtomicP<U>: Sized {
    fn atomic_load(&self, order: Ordering) -> Self;
    fn atomic_store(&mut self, val: Self, order: Ordering);
    fn atomic_swap(&mut self, val: Self, order: Ordering) -> Self;

    fn atomic_compare_exchange(
        &mut self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;

    fn atomic_compare_exchange_weak(
        &mut self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;
}

impl<U> AtomicP<U> for *const U {
    fn atomic_load(&self, order: Ordering) -> Self {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.load(order)
    }

    fn atomic_store(&mut self, val: Self, order: Ordering) {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.store(mptr!(val), order)
    }

    fn atomic_swap(&mut self, val: Self, order: Ordering) -> Self {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.swap(mptr!(val), order)
    }

    fn atomic_compare_exchange(
        &mut self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        match adtomic_ptr.compare_exchange(mptr!(current), mptr!(new), success, failure) {
            Ok(x) => Ok(cptr!(x)),
            Err(x) => Err(cptr!(x)),
        }
    }

    fn atomic_compare_exchange_weak(
        &mut self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        match adtomic_ptr.compare_exchange_weak(mptr!(current), mptr!(new), success, failure) {
            Ok(x) => Ok(cptr!(x)),
            Err(x) => Err(cptr!(x)),
        }
    }
}

impl<U> AtomicP<U> for *mut U {
    fn atomic_load(&self, order: Ordering) -> Self {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.load(order)
    }

    fn atomic_store(&mut self, val: Self, order: Ordering) {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.store(mptr!(val), order)
    }

    fn atomic_swap(&mut self, val: Self, order: Ordering) -> Self {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        adtomic_ptr.swap(mptr!(val), order)
    }

    fn atomic_compare_exchange(
        &mut self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        match adtomic_ptr.compare_exchange(mptr!(current), mptr!(new), success, failure) {
            Ok(x) => Ok(x),
            Err(x) => Err(x),
        }
    }

    fn atomic_compare_exchange_weak(
        &mut self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let adtomic_ptr: &AtomicPtr<U> = unsafe { transmute(self) };
        match adtomic_ptr.compare_exchange_weak(mptr!(current), mptr!(new), success, failure) {
            Ok(x) => Ok(x),
            Err(x) => Err(x),
        }
    }
}
