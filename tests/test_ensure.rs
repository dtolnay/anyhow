#![allow(
    clippy::diverging_sub_expression,
    clippy::if_same_then_else,
    clippy::ifs_same_cond,
    clippy::let_and_return,
    clippy::let_underscore_drop,
    clippy::match_bool,
    clippy::never_loop,
    clippy::redundant_closure_call,
    clippy::redundant_pattern_matching,
    clippy::unit_arg,
    clippy::while_immutable_condition
)]

use anyhow::{anyhow, ensure, Chain, Error, Result};
use std::fmt::Debug;
use std::iter;
use std::marker::PhantomData;
use std::ops::Add;
use std::ptr;

struct S;

impl<T> Add<T> for S {
    type Output = bool;
    fn add(self, rhs: T) -> Self::Output {
        let _ = rhs;
        false
    }
}

trait Trait: Sized {
    fn t(self, i: i32) -> i32 {
        i
    }
}

impl<T> Trait for T {}

#[track_caller]
fn assert_err<T: Debug>(result: impl FnOnce() -> Result<T>, expected: &'static str) {
    let actual = result().unwrap_err().to_string();

    let mut accepted_alternatives = expected.split('\n');
    let expected = accepted_alternatives.next_back().unwrap();
    if accepted_alternatives.any(|alternative| actual == alternative) {
        return;
    }

    assert_eq!(actual, expected);
}

#[test]
fn test_recursion() {
    // Must not blow the default #[recursion_limit], which is 128.
    #[rustfmt::skip]
    let test = || Ok(ensure!(
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false |
        false | false | false | false | false | false | false | false | false
    ));

    test().unwrap_err();
}

#[test]
fn test_low_precedence_control_flow() {
    #[allow(unreachable_code)]
    let test = || {
        let val = loop {
            // Break has lower precedence than the comparison operators so the
            // expression here is `S + (break (1 == 1))`. It would be bad if the
            // ensure macro partitioned this input into `(S + break 1) == (1)`
            // because that means a different thing than what was written.
            ensure!(S + break 1 == 1);
        };
        Ok(val)
    };

    assert!(test().unwrap());
}

#[test]
fn test_low_precedence_binary_operator() {
    // Must not partition as `false == (true && false)`.
    let test = || Ok(ensure!(false == true && false));
    assert_err(test, "Condition failed: `false == true && false`");

    // But outside the root level, it is fine.
    let test = || Ok(ensure!(while false == true && false {} < ()));
    assert_err(
        test,
        "Condition failed: `while false == true && false { } < ()` (() vs ())",
    );
}

#[test]
fn test_closure() {
    // Must not partition as `(S + move) || (1 == 1)` by treating move as an
    // identifier, nor as `(S + move || 1) == (1)` by misinterpreting the
    // closure precedence.
    let test = || Ok(ensure!(S + move || 1 == 1));
    assert_err(test, "Condition failed: `S + (move || 1 == 1)`");

    let test = || Ok(ensure!(S + || 1 == 1));
    assert_err(test, "Condition failed: `S + (|| 1 == 1)`");

    // Must not partition as `S + ((move | ()) | 1) == 1` by treating those
    // pipes as bitwise-or.
    let test = || Ok(ensure!(S + move |()| 1 == 1));
    assert_err(test, "Condition failed: `S + (move |()| 1 == 1)`");

    let test = || Ok(ensure!(S + |()| 1 == 1));
    assert_err(test, "Condition failed: `S + (|()| 1 == 1)`");
}

#[test]
fn test_unary() {
    let mut x = &1;
    let test = || Ok(ensure!(*x == 2));
    assert_err(test, "Condition failed: `*x == 2` (1 vs 2)");

    let test = || Ok(ensure!(!x == 1));
    assert_err(test, "Condition failed: `!x == 1` (-2 vs 1)");

    let test = || Ok(ensure!(-x == 1));
    assert_err(test, "Condition failed: `-x == 1` (-1 vs 1)");

    let test = || Ok(ensure!(&x == &&2));
    assert_err(test, "Condition failed: `&x == &&2` (1 vs 2)");

    let test = || Ok(ensure!(&mut x == *&&mut &2));
    assert_err(test, "Condition failed: `&mut x == *&&mut &2` (1 vs 2)");
}

#[test]
fn test_if() {
    #[rustfmt::skip]
    let test = || Ok(ensure!(if false {}.t(1) == 2));
    assert_err(test, "Condition failed: `if false { }.t(1) == 2` (1 vs 2)");

    #[rustfmt::skip]
    let test = || Ok(ensure!(if false {} else {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if false { } else { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if false {} else if false {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if false { } else if false { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if let 1 = 2 {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if let 1 = 2 { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if let 1 | 2 = 2 {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if let 1 | 2 = 2 { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(if let | 1 | 2 = 2 {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `if let 1 | 2 = 2 { }.t(1) == 2` (1 vs 2)",
    );
}

#[test]
fn test_loop() {
    #[rustfmt::skip]
    let test = || Ok(ensure!(1 + loop { break 1 } == 1));
    assert_err(
        test,
        // 1.54 puts a double space after loop
        "Condition failed: `1 + loop  { break 1  } == 1` (2 vs 1)\n\
         Condition failed: `1 + loop { break 1  } == 1` (2 vs 1)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(1 + 'a: loop { break 'a 1 } == 1));
    assert_err(
        test,
        // 1.54 puts a double space after loop
        "Condition failed: `1 + 'a: loop  { break 'a 1  } == 1` (2 vs 1)\n\
         Condition failed: `1 + 'a: loop { break 'a 1  } == 1` (2 vs 1)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(while false {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `while false { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(while let None = Some(1) {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `while let None = Some(1) { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(for _x in iter::once(0) {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `for _x in iter::once(0) { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(for | _x in iter::once(0) {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `for _x in iter::once(0) { }.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(for true | false in iter::empty() {}.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `for true | false in iter::empty() { }.t(1) == 2` (1 vs 2)",
    );
}

#[test]
fn test_match() {
    #[rustfmt::skip]
    let test = || Ok(ensure!(match 1 == 1 { true => 1, false => 0 } == 2));
    assert_err(
        test,
        "Condition failed: `match 1 == 1 { true => 1, false => 0, } == 2` (1 vs 2)",
    );
}

#[test]
fn test_atom() {
    let test = || Ok(ensure!([false, false].len() > 3));
    assert_err(
        test,
        "Condition failed: `[false, false].len() > 3` (2 vs 3)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!({ let x = 1; x } >= 3));
    assert_err(test, "Condition failed: `{ let x = 1; x } >= 3` (1 vs 3)");

    let test = || Ok(ensure!(S + async { 1 } == true));
    assert_err(
        test,
        "Condition failed: `S + async  { 1 } == true` (false vs true)",
    );

    let test = || Ok(ensure!(S + async move { 1 } == true));
    assert_err(
        test,
        "Condition failed: `S + async move  { 1 } == true` (false vs true)",
    );

    let x = &1;
    let test = || Ok(ensure!(S + unsafe { ptr::read(x) } == true));
    assert_err(
        test,
        "Condition failed: `S + unsafe { ptr::read(x) } == true` (false vs true)",
    );
}

#[test]
fn test_path() {
    let test = || Ok(ensure!(crate::S.t(1) == 2));
    assert_err(test, "Condition failed: `crate::S.t(1) == 2` (1 vs 2)");

    let test = || Ok(ensure!(::anyhow::Error::root_cause.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `::anyhow::Error::root_cause.t(1) == 2` (1 vs 2)",
    );

    let test = || Ok(ensure!(Error::msg::<&str>.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `Error::msg::<&str>.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(Error::msg::<&str,>.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `Error::msg::<&str>.t(1) == 2` (1 vs 2)",
    );

    let test = || Ok(ensure!(Error::msg::<<str as ToOwned>::Owned>.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `Error::msg::<<str as ToOwned>::Owned>.t(1) == 2` (1 vs 2)",
    );

    let test = || Ok(ensure!(Chain::<'static>::new.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `Chain::<'static>::new.t(1) == 2` (1 vs 2)",
    );

    #[rustfmt::skip]
    let test = || Ok(ensure!(Chain::<'static,>::new.t(1) == 2));
    assert_err(
        test,
        "Condition failed: `Chain::<'static>::new.t(1) == 2` (1 vs 2)",
    );
}

#[test]
fn test_macro() {
    let test = || Ok(ensure!(anyhow!("...").to_string().len() <= 1));
    assert_err(
        test,
        "Condition failed: `anyhow!(\"...\").to_string().len() <= 1` (3 vs 1)",
    );

    let test = || Ok(ensure!(vec![1].len() < 1));
    assert_err(test, "Condition failed: `vec![1].len() < 1` (1 vs 1)");

    let test = || Ok(ensure!(stringify! {} != ""));
    assert_err(
        test,
        "Condition failed: `stringify! { } != \"\"` (\"\" vs \"\")",
    );
}

#[test]
fn test_trailer() {
    let test = || Ok(ensure!((|| 1)() == 2));
    assert_err(test, "Condition failed: `(|| 1)() == 2` (1 vs 2)");

    let test = || Ok(ensure!(b"hmm"[1] == b'c'));
    assert_err(test, "Condition failed: `b\"hmm\"[1] == b'c'` (109 vs 99)");

    let test = || Ok(ensure!(PhantomData::<u8> {} != PhantomData));
    assert_err(
        test,
        "Condition failed: `PhantomData::<u8>{} != PhantomData` (PhantomData vs PhantomData)",
    );

    let result = Ok::<_, Error>(1);
    let test = || Ok(ensure!(result? == 2));
    assert_err(test, "Condition failed: `result? == 2` (1 vs 2)");

    let test = || Ok(ensure!((2, 3).1 == 2));
    assert_err(test, "Condition failed: `(2, 3).1 == 2` (3 vs 2)");

    #[rustfmt::skip]
    let test = || Ok(ensure!((2, (3, 4)). 1.1 == 2));
    assert_err(test, "Condition failed: `(2, (3, 4)).1.1 == 2` (4 vs 2)");

    let err = anyhow!("");
    let test = || Ok(ensure!(err.is::<&str>() == false));
    assert_err(
        test,
        "Condition failed: `err.is::<&str>() == false` (true vs false)",
    );

    let test = || Ok(ensure!(err.is::<<str as ToOwned>::Owned>() == true));
    assert_err(
        test,
        "Condition failed: `err.is::<<str as ToOwned>::Owned>() == true` (false vs true)",
    );
}

#[test]
fn test_whitespace() {
    #[derive(Debug)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    let point = Point { x: 0, y: 0 };
    let test = || Ok(ensure!("" == format!("{:#?}", point)));
    assert_err(
        test,
        "Condition failed: `\"\" == format!(\"{:#?}\", point)`",
    );
}

#[test]
fn test_too_long() {
    let test = || Ok(ensure!("" == "x".repeat(10)));
    assert_err(
        test,
        "Condition failed: `\"\" == \"x\".repeat(10)` (\"\" vs \"xxxxxxxxxx\")",
    );

    let test = || Ok(ensure!("" == "x".repeat(80)));
    assert_err(test, "Condition failed: `\"\" == \"x\".repeat(80)`");
}
