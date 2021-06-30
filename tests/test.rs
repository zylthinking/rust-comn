#![allow(unused_imports)]

#[macro_use]
extern crate comn;
use comn::*;
use std::{self, panic};

#[test]
fn test_proc_macro() {
    suffx!(
        let n!(r#pub) = 1;
        let n!(y) = 2;
    );

    suffx!(
        let n!(x) = 3;
        let n!(y) = 4;
    );

    assert!(pub_0 == 1);
    assert!(y_0 == 2);
    assert!(x_1 == 3);
    assert!(y_1 == 4);
}

#[test]
fn test_head() {
    struct Xx {
        _i: i32,
        head: ListHead,
    }

    let mut x0 = Xx {
        _i: 9,
        head: ListHead::new(),
    };
    x0.head.init_list_head();

    let z = list_entry!(&x0.head, Xx, head);
    assert_eq!(z as *const _, &x0 as *const _);

    InitListHead!(head);
    assert_eq!(&head as *const ListHead, head.next);
    assert_eq!(head.next, head.prev);

    InitListHead!(head2);
    unsafe {
        head.list_add(&mut head2);
    }

    let mut head = ListHead::new();
    head.init_list_head();
    println!("{:p}, {:p}, {:p}", &head, head.next, head.prev);
}

#[test]
fn test_my_handle() {
    struct X {
        _n: i32,
    }

    impl Drop for X {
        fn drop(&mut self) {
            println!("X freed");
        }
    }

    let h = MyHandle::attach(Box::new(X { _n: 98 }));
    let h2 = h.clone();
    let _ = h.get();
    h.dettach();
    println!("X should not freed");
    h.put();
    println!("X should freed");
    drop(h);
    println!("nothing happend");

    drop(h2);
    println!("handle done");
    // if let &Some(ref x) = z {
    //     println!("fuck {}", x.n)
    // }
}

#[test]
fn lkf_test() {
    let mut q = Lkf::new();
    let mut x = LkfNode::new();
    assert!(lkf_next_unsafe!(&mut x) == nil!());

    let _ = lkf_put_unsafe!(&mut q, &mut x).unwrap();
    let result = lkf_put_unsafe!(&mut q, &mut x);
    assert!(matches!(result, Err(_)));

    let node = lkf_get!(&mut q);
    assert!(lkf_next_unsafe!(node) == &mut x);
    assert!(lkf_next_unsafe!(node) == nil!());

    let _ = lkf_put_unsafe!(q, &mut x).unwrap();
    let result = panic::catch_unwind(|| drop(x));
    assert!(result.is_err());
}

#[test]
fn a() {
    fn t<'a>(_f: fn(&'a String) -> &'a String) {
        let _s = "1".to_owned();
        println!("accept _f");
        //_f(&s);
    }

    fn x1<'a>(_s: &'a String) -> &'static String {
        let b = Box::new(String::from("111"));
        unsafe { &*Box::into_raw(b) }
    }
    t(x1);
}
