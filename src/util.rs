#[macro_export]
/// A debug print that calls dbg_print! but only does so on the debug profile
macro_rules! dbg_print {
    ($($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        {
            #[allow(unused_variables)]
            dbg!($($val),+);
        }
    };
}
