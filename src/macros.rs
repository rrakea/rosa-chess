// "Inspired" from the crate debug_print
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
        log::info!($($arg)*)
    };
}

// My own version of panic :)
// Does not get removed in release
#[macro_export]
macro_rules! scream{
    ($($arg:tt)*) => {
        {
        ::log::error!($($arg)*);
        panic!($($arg)*)
        }
    };
}
