use crate::Error;
use alloc::string::String;
use core::fmt::{self, Debug, Write};
use core::mem::MaybeUninit;
use core::ptr;
use core::slice;
use core::str;

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
        render(msg, &self.0, &self.1)
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

struct Buf {
    bytes: [MaybeUninit<u8>; 40],
    written: usize,
}

impl Buf {
    fn new() -> Self {
        Buf {
            bytes: [MaybeUninit::uninit(); 40],
            written: 0,
        }
    }

    fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(
                self.bytes.as_ptr().cast::<u8>(),
                self.written,
            ))
        }
    }
}

impl Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.bytes().any(|b| b == b' ' || b == b'\n') {
            return Err(fmt::Error);
        }

        let remaining = self.bytes.len() - self.written;
        if s.len() > remaining {
            return Err(fmt::Error);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                s.as_ptr(),
                self.bytes.as_mut_ptr().add(self.written).cast::<u8>(),
                s.len(),
            );
        }
        self.written += s.len();
        Ok(())
    }
}

fn render(msg: &'static str, lhs: &dyn Debug, rhs: &dyn Debug) -> Error {
    let mut lhs_buf = Buf::new();
    if fmt::write(&mut lhs_buf, format_args!("{:?}", lhs)).is_ok() {
        let mut rhs_buf = Buf::new();
        if fmt::write(&mut rhs_buf, format_args!("{:?}", rhs)).is_ok() {
            let lhs_str = lhs_buf.as_str();
            let rhs_str = rhs_buf.as_str();
            // "{msg} ({lhs} vs {rhs})"
            let len = msg.len() + 2 + lhs_str.len() + 4 + rhs_str.len() + 1;
            let mut string = String::with_capacity(len);
            string.push_str(msg);
            string.push_str(" (");
            string.push_str(lhs_str);
            string.push_str(" vs ");
            string.push_str(rhs_str);
            string.push(')');
            return Error::msg(string);
        }
    }
    Error::msg(msg)
}

#[doc(hidden)]
#[macro_export]
macro_rules! __parse_ensure {
    (atom () $bail:tt $fuel:tt {($($rhs:tt)+) ($($lhs:tt)+) $op:tt} $dup:tt $(,)?) => {
        $crate::__fancy_ensure!($($lhs)+, $op, $($rhs)+)
    };

    // low precedence control flow constructs

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt return $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt break $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt continue $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt yield $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    (0 $stack:tt ($($bail:tt)*) $fuel:tt $parse:tt $dup:tt move $($rest:tt)*) => {
        $crate::__fallback_ensure!($($bail)*)
    };

    // unary operators

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($deref:tt $($dup:tt)*) * $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $deref) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($not:tt $($dup:tt)*) ! $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $not) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($neg:tt $($dup:tt)*) - $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $neg) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($let:tt $($dup:tt)*) let $(|)? $($pat:pat)|+ = $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $let $($pat)|+ =) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($life:tt $colon:tt $($dup:tt)*) $label:lifetime : $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $life $colon) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($and:tt $mut:tt $($dup:tt)*) &mut $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $and $mut) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($and:tt $($dup:tt)*) & $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $and) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($andand:tt $mut:tt $($dup:tt)*) &&mut $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $andand $mut) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($andand:tt $($dup:tt)*) && $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $andand) $($parse)*} ($($rest)*) $($rest)*)
    };

    // control flow constructs

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($if:tt $($dup:tt)*) if $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $if) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($match:tt $($dup:tt)*) match $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $match) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($while:tt $($dup:tt)*) while $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $while) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($for:tt $($dup:tt)*) for $(|)? $($pat:pat)|+ in $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $for $($pat)|+ in) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom (cond $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(cond $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($else:tt $if:tt $($dup:tt)*) else if $($rest:tt)*) => {
        $crate::__parse_ensure!(0 (cond $stack) $bail ($($fuel)*) {($($buf)* $else $if) $($parse)*} ($($rest)*) $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($else:tt $brace:tt $($dup:tt)*) else {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $else $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (cond $stack:tt $bail:tt (~$($fuel:tt)*) $parse:tt $dup:tt $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) $parse $dup $($rest)*)
    };

    // atomic expressions

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($paren:tt $($dup:tt)*) ($($content:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $paren) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bracket:tt $($dup:tt)*) [$($array:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bracket) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($loop:tt $block:tt $($dup:tt)*) loop {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $loop $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($async:tt $block:tt $($dup:tt)*) async {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $async $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($async:tt $move:tt $block:tt $($dup:tt)*) async move {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $async $move $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($unsafe:tt $block:tt $($dup:tt)*) unsafe {$($body:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $unsafe $block) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $lit) $($parse)*} ($($rest)*) $($rest)*)
    };

    // path expressions

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: $($rest:tt)*) => {
        $crate::__parse_ensure!(path $stack $bail ($($fuel)*) {($($buf)* $colons) $($parse)*} ($($rest)*) $($rest)*)
    };

    (0 $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(component $stack $bail ($($fuel)*) {($($buf)* $ident) $($parse)*} ($($rest)*) $($rest)*)
    };

    (path $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $ident:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(component $stack $bail ($($fuel)*) {($($buf)* $ident) $($parse)*} ($($rest)*) $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $langle:tt $($dup:tt)*) :: < $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (component $stack) $bail ($($fuel)*) {($($buf)* $colons $langle) $($parse)*} ($($rest)*) $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: << $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (component $stack) $bail ($($fuel)*) {($($buf)* $colons <) $($parse)*} (< $($rest)*) < $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($colons:tt $($dup:tt)*) :: $($rest:tt)*) => {
        $crate::__parse_ensure!(path $stack $bail ($($fuel)*) {($($buf)* $colons) $($parse)*} ($($rest)*) $($rest)*)
    };

    // macro invocations

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bang:tt $args:tt $($dup:tt)*) ! ($($mac:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bang $args) $($parse)*} ($($rest)*) $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bang:tt $args:tt $($dup:tt)*) ! [$($mac:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bang $args) $($parse)*} ($($rest)*) $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bang:tt $args:tt $($dup:tt)*) ! {$($mac:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bang $args) $($parse)*} ($($rest)*) $($rest)*)
    };

    (component $stack:tt $bail:tt (~$($fuel:tt)*) $parse:tt $dup:tt $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) $parse $dup $($rest)*)
    };

    // trailer expressions

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($paren:tt $($dup:tt)*) ($($call:tt)*) $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $paren) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bracket:tt $($dup:tt)*) [$($index:tt)*] $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $bracket) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($init:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($question:tt $($dup:tt)*) ? $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $question) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $ident:tt $colons:tt $langle:tt $($dup:tt)*) . $i:ident :: < $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* $dot $ident $colons $langle) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $ident:tt $colons:tt $($dup:tt)*) . $i:ident :: << $($rest:tt)*) => {
        $crate::__parse_ensure!(generic (atom $stack) $bail ($($fuel)*) {($($buf)* $dot $ident $colons <) $($parse)*} (< $($rest)*) < $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $($dup:tt)*) . $field:ident $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $dot $field) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($dot:tt $index:tt $($dup:tt)*) . $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(atom $stack $bail ($($fuel)*) {($($buf)* $dot $index) $($parse)*} ($($rest)*) $($rest)*)
    };

    // angle bracketed generic arguments

    (generic ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($rangle:tt $($dup:tt)*) > $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $rangle) $($parse)*} ($($rest)*) $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $lit:literal $($rest:tt)*) => {
        $crate::__parse_ensure!(arglist $stack $bail ($($fuel)*) {($($buf)* $lit) $($parse)*} ($($rest)*) $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($brace:tt $($dup:tt)*) {$($block:tt)*} $($rest:tt)*) => {
        $crate::__parse_ensure!(arglist $stack $bail ($($fuel)*) {($($buf)* $brace) $($parse)*} ($($rest)*) $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $life:lifetime $($rest:tt)*) => {
        $crate::__parse_ensure!(arglist $stack $bail ($($fuel)*) {($($buf)* $life) $($parse)*} ($($rest)*) $($rest)*)
    };

    (generic $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $ty:ty , $($rest:tt)*) => {
        $crate::__parse_ensure!(generic $stack $bail ($($fuel)*) {($($buf)* $ty ,) $($parse)*} ($($rest)*) $($rest)*)
    };

    (generic ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} $dup:tt $ty:ty > $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $ty >) $($parse)*} ($($rest)*) $($rest)*)
    };

    (arglist $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($comma:tt $($dup:tt)*) , $($rest:tt)*) => {
        $crate::__parse_ensure!(generic $stack $bail ($($fuel)*) {($($buf)* $comma) $($parse)*} ($($rest)*) $($rest)*)
    };

    (arglist ($pop:ident $stack:tt) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($rangle:tt $($dup:tt)*) > $($rest:tt)*) => {
        $crate::__parse_ensure!($pop $stack $bail ($($fuel)*) {($($buf)* $rangle) $($parse)*} ($($rest)*) $($rest)*)
    };

    // high precedence binary operators

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($add:tt $($dup:tt)*) + $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $add) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($sub:tt $($dup:tt)*) - $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $sub) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($mul:tt $($dup:tt)*) * $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $mul) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($div:tt $($dup:tt)*) / $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $div) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($rem:tt $($dup:tt)*) % $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $rem) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bitxor:tt $($dup:tt)*) ^ $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $bitxor) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bitand:tt $($dup:tt)*) & $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $bitand) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bitor:tt $($dup:tt)*) | $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $bitor) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($shl:tt $($dup:tt)*) << $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $shl) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($shr:tt $($dup:tt)*) >> $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $shr) $($parse)*} ($($rest)*) $($rest)*)
    };

    // comparison binary operators

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($eq:tt $($dup:tt)*) == $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) $eq} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} ($eq:tt $($dup:tt)*) == $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $eq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($le:tt $($dup:tt)*) <= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) $le} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} ($le:tt $($dup:tt)*) <= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $le) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($lt:tt $($dup:tt)*) < $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) $lt} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} ($lt:tt $($dup:tt)*) < $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $lt) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($ne:tt $($dup:tt)*) != $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) $ne} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} ($ne:tt $($dup:tt)*) != $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $ne) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($ge:tt $($dup:tt)*) >= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) $ge} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} ($ge:tt $($dup:tt)*) >= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $ge) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom () $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($gt:tt $($dup:tt)*) > $($rest:tt)*) => {
        $crate::__parse_ensure!(0 () $bail ($($fuel)*) {() $($parse)* ($($buf)*) $gt} ($($rest)*) $($rest)*)
    };

    (atom $stack:tt $bail:tt (~$($fuel:tt)*) {($($buf:tt)+) $($parse:tt)*} ($gt:tt $($dup:tt)*) > $($rest:tt)*) => {
        $crate::__parse_ensure!(0 $stack $bail ($($fuel)*) {($($buf)* $gt) $($parse)*} ($($rest)*) $($rest)*)
    };

    // low precedence binary operators

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($and:tt $($dup:tt)*) && $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $and) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($or:tt $($dup:tt)*) || $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $or) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($assign:tt $($dup:tt)*) = $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $assign) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($addeq:tt $($dup:tt)*) += $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $addeq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($subeq:tt $($dup:tt)*) -= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $subeq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($muleq:tt $($dup:tt)*) *= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $muleq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($diveq:tt $($dup:tt)*) /= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $diveq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($remeq:tt $($dup:tt)*) %= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $remeq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bitxoreq:tt $($dup:tt)*) ^= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $bitxoreq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bitandeq:tt $($dup:tt)*) &= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $bitandeq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($bitoreq:tt $($dup:tt)*) |= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $bitoreq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($shleq:tt $($dup:tt)*) <<= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $shleq) $($parse)*} ($($rest)*) $($rest)*)
    };

    (atom ($($stack:tt)+) $bail:tt (~$($fuel:tt)*) {($($buf:tt)*) $($parse:tt)*} ($shreq:tt $($dup:tt)*) >>= $($rest:tt)*) => {
        $crate::__parse_ensure!(0 ($($stack)*) $bail ($($fuel)*) {($($buf)* $shreq) $($parse)*} ($($rest)*) $($rest)*)
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
