#[macro_export]
macro_rules! c_str {
    ( $s:expr ) => {{
        const IT: &'static [$crate::misc::__::u8] = $crate::misc::__::core::concat!($s, "\0").as_bytes();
        #[allow(deprecated)]
        {
            use $crate::misc::__::*;
            let _: NoInnerNullBytesUntil<{ $s.len() }> = NoInnerNullBytesUntil::<{ c_strlen(IT) }>;
            unsafe { core::mem::transmute::<&'static [u8], &'static std::ffi::CStr>(IT) }
        }
    }};
}

#[doc(hidden)]
#[deprecated(note = "Not part of the public API")]
pub mod __ {
    pub use ::core;
    pub use ::std;

    pub use u8;

    pub const fn c_strlen(bytes: &'static [u8]) -> usize {
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\0' {
                return i;
            }
            i += 1;
        }
        i + 1
    }

    pub struct NoInnerNullBytesUntil<const IDX: usize>;
}
