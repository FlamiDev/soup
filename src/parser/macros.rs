macro_rules! ws_sep {
    ($first:expr, $second:expr $(, $rest: expr)+ $(,)?) => {
        separated_pair($first, ws, ws_sep!($second  $(, $rest)*))
    };
    ($first:expr, $second:expr $(,)?) => {
        separated_pair($first, multispace1, $second)
    };
}
