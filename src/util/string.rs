use std::ops::Fn;

pub struct StringUtils {}

impl StringUtils {
    pub fn maybe_pluralize((singular, plural) : (&str, &str), qty: usize) -> String {
        format!("{}", if qty == 1 { singular } else { plural })
    }

    pub fn maybe_pluralize_count(count: usize, (singular, plural) : (&str, &str)) -> String {
        format!("{} {}", count, Self::maybe_pluralize((singular, plural), count))
    }
}
