//! Common utilities.

#[macro_export]
/// A debug print that calls dbg_print! but only does so on the debug profile.
///
/// This means when compiling in debug mode, you get easy debug logging, but
/// this is completely absent from the release binary.
macro_rules! dbg_print {
    ($($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        {
            let _ = dbg!($($val),+);
        }
    };
}
