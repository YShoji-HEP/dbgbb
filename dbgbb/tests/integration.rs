use array_object::adaptor::VecShape;
use dbgbb::*;
use num_complex::Complex64;

macro_rules! post_read {
    ($x:expr,$title:literal,$ty:ty) => {
        dbgbb!($x);
        let a: $ty = dbgbb_read!($title);
        assert_eq!($x, a);
    };
}

#[test]
fn scalar() {
    post_read!(1usize, "1usize", usize);
    post_read!(1u64, "1u64", u64);
    post_read!(-1i64, "-1i64", i64);
    post_read!(1f64, "1f64", f64);
    post_read!(Complex64::new(1., 2.), "Complex64::new(1., 2.)", Complex64);
    post_read!("text".to_string(), "\"text\".to_string()", String);
}

#[test]
fn array() {
    post_read!(vec![1u32, 2u32, 3u32], "vec![1u32, 2u32, 3u32]", Vec<u32>);
}

#[test]
fn accumulate() {
    let mut c = vec![];
    for i in 0..12 {
        dbgbb_acc!(label=>"test", i);
        c.push(i);
    }
    dbgbb_acc!("test" => post);
    let res: Vec<i32> = dbgbb_read!("i");
    assert_eq!(res, c)
}

#[test]
fn flatten() {
    let vv = vec![vec![1, 2], vec![3, 4]];
    dbgbb_flatten!(vv, depth=>1);
    let flat1: Vec<i32> = dbgbb_read!("vv");
    assert_eq!(flat1, vec![3, 4]);
    dbgbb_flatten!(vv.rename("vv2"), depth=>2);
    let flat2: i32 = dbgbb_read!("vv2");
    assert_eq!(flat2, 4);
}

#[test]
fn concat() {
    let vv3 = vec![vec![1, 2], vec![3, 4]];
    dbgbb_concat!(vv3, depth=>1);
    let VecShape::<i32>(v, shape) = dbgbb_read!("vv3");
    assert_eq!(v, vec![1, 2, 3, 4]);
    assert_eq!(shape, vec![2, 2]);
}

#[test]
fn buffer() {
    let _buf = dbgbb::Buffer::on();
    for i in 0..100 {
        dbgbb!(vec![1f64; 100].rename(&i.to_string()));
    }
}
