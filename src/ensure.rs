use crate::{anyhow, Error};
use core::fmt::Debug;

#[doc(hidden)]
pub trait BothDebug {
    fn __dispatch_ensure(self, msg: &'static str) -> Error;
}

impl<A, B> BothDebug for (A, B)
where
    A: Debug,
    B: Debug,
{
    fn __dispatch_ensure(self, msg: &'static str) -> Error {
        anyhow!("{} ({:?} vs {:?})", msg, self.0, self.1)
    }
}

#[doc(hidden)]
pub trait NotBothDebug {
    fn __dispatch_ensure(self, msg: &'static str) -> Error;
}

impl<A, B> NotBothDebug for &(A, B) {
    fn __dispatch_ensure(self, msg: &'static str) -> Error {
        Error::msg(msg)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __parse_ensure {
    (atom () $bail:tt $fuel:tt {($($rhs:tt)+) ($($lhs:tt)+) $op:tt} $(,)?) => {
        $crate::__fancy_ensure!($($lhs)+, $op, $($rhs)+)
    };

    // low precedence control flow constructs

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt return $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt break $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt continue $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt yield $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt move $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    // unary operators

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} * $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* *) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ! $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* !) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} - $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* -) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} let $(|)? $($pat:pat)|+ = $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* let $($pat)|+ =) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $life:lifetime : $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $life :) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} &mut $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* &mut) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} & $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* &) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} &&mut $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* &&mut) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} && $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* &&) $($parse)*} $($rest)*)
    };

    // control flow constructs

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} if $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* if) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} match $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* match) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} while $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* while) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} for $(|)? $($pat:pat)|+ in $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* for $($pat)|+ in) $($parse)*} $($rest)*)
    };

    (atom (cond $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(cond $stack $bail ($($fuel)*) {($($buf)* {$($block)*}) $($parse)*} $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} else if $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* else if) $($parse)*} $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} else {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* else {$($block)*}) $($parse)*} $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) $parse:tt $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) $parse $($rest)*)
    };

    // atomic expressions

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($($paren:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* ($($paren)*)) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} [$($array:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* [$($array)*]) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* {$($block)*}) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} loop {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* loop {$($block)*}) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} async {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* async {$($block)*}) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} async move {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* async move {$($block)*}) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} unsafe {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* unsafe {$($block)*}) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $lit) $($parse)*} $($rest)*)
    };

    // path expressions

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} :: $($rest:tt)*) => {
        $crate::__parse_ensure!(path $stack $bail ($($fuel)*) {($($buf)* ::) $($parse)*} $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(component $stack $bail ($($fuel)*) {($($buf)* $ident) $($parse)*} $($rest)*)
    };

    (path $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(component $stack $bail ($($fuel)*) {($($buf)* $ident) $($parse)*} $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} :: < $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (component $stack) $bail ($($fuel)*) {($($buf)* :: <) $($parse)*} $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} :: << $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (component $stack) $bail ($($fuel)*) {($($buf)* :: <) $($parse)*} < $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} :: $($rest:tt)*) => {
        $crate::__parse_ensure!(path $stack $bail ($($fuel)*) {($($buf)* ::) $($parse)*} $($rest)*)
    };

    // macro invocations

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ! ($($mac:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* ! ($($mac)*)) $($parse)*} $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ! [$($mac:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* ! [$($mac)*]) $($parse)*} $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ! {$($mac:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* ! {$($mac)*}) $($parse)*} $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) $parse:tt $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) $parse $($rest)*)
    };

    // trailer expressions

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($($call:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* ($($call)*)) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} [$($index:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* [$($index)*]) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} {$($init:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* {$($init)*}) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ? $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* ?) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} . $ident:ident :: < $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* . $ident :: <) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} . $ident:ident :: << $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* . $ident :: <) $($parse)*} < $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} . $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* . $ident) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} . $lit:tt $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* . $lit) $($parse)*} $($rest)*)
    };

    // angle bracketed generic arguments

    (generic ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} > $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* >) $($parse)*} $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(arglist $stack $bail ($($fuel)*) {($($buf)* $lit) $($parse)*} $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(arglist $stack $bail ($($fuel)*) {($($buf)* {$($block)*}) $($parse)*} $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $life:lifetime $($rest:tt)*) => {
        $crate::__parse_ensure!(arglist $stack $bail ($($fuel)*) {($($buf)* $life) $($parse)*} $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $ty:ty , $($rest:tt)*) => {
        $crate::__parse_ensure!(generic $stack $bail ($($fuel)*) {($($buf)* $ty ,) $($parse)*} $($rest)*)
    };

    (generic ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $ty:ty > $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $ty >) $($parse)*} $($rest)*)
    };

    (arglist $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} , $($rest:tt)*) => {
        $crate::__parse_ensure!(generic $stack $bail ($($fuel)*) {($($buf)* ,) $($parse)*} $($rest)*)
    };

    (arglist ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} > $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* >) $($parse)*} $($rest)*)
    };

    // high precedence binary operators

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} + $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* +) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} - $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* -) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} * $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* *) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} / $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* /) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} % $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* %) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ^ $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* ^) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} & $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* &) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} | $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* |) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} << $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* <<) $($parse)*} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} >> $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* >>) $($parse)*} $($rest)*)
    };

    // comparison binary operators

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} == $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) ==} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} == $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* ==) $($parse)*} $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} <= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) <=} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} <= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* <=) $($parse)*} $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} < $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) <} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} < $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* <) $($parse)*} $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} != $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) !=} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} != $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* !=) $($parse)*} $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} >= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) >=} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} >= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* >=) $($parse)*} $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} > $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) >} $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} > $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* >) $($parse)*} $($rest)*)
    };

    // low precedence binary operators

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} && $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* &&) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} || $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* ||) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} = $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* =) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} += $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* +=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} -= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* -=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} *= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* *=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} /= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* /=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} %= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* %=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ^= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* ^=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} &= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* &=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} |= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* |=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} <<= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* <<=) $($parse)*} $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} >>= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* >>=) $($parse)*} $($rest)*)
    };

    // unrecognized expression

    ($state:tt $stack:tt ($($bail:tt)*) $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fancy_ensure {
    ($lhs:expr, $op:tt, $rhs:expr) => {
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                if !(lhs $op rhs) {
                    #[allow(unused_imports)]
                    use $crate::private::{BothDebug, NotBothDebug};
                    return Err((lhs, rhs).__dispatch_ensure(concat!("Condition failed: `", stringify!($lhs), " ", stringify!($op), " ", stringify!($rhs), "`")));
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __fallback_ensure {
    ($cond:expr $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::Error::msg(
                $crate::private::concat!("Condition failed: `", $crate::private::stringify!($cond), "`")
            ));
        }
    };
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::anyhow!($msg));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::anyhow!($err));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return $crate::private::Err($crate::anyhow!($fmt, $($arg)*));
        }
    };
}
