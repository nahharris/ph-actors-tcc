#[macro_export]
macro_rules! arc_slice {
    [$($x:expr),*] => {
        ph::utils::ArcSlice::from([$($x.into()),*])
    };
}

#[macro_export]
macro_rules! arc_str {
    ($x:expr) => {
        ph::utils::ArcStr::from($x)
    };
}