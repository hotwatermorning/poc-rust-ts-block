#[allow(unused_imports)]
extern crate ts_macro_impl;
#[doc(hidden)]
pub use ts_macro_impl::*;

#[macro_export]
macro_rules! ts_block {
  // inline closure
  ({$($rest:tt)*}) => {{
      #[allow(unused)]
      #[derive($crate::__ts_block_internal_closure)]
      enum TsClosureInput {
          Input = (stringify!($($rest)*), 0).1
      }
      __ts_block_closure_impl!()
  }};
}
