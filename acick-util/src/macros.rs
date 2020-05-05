/// Create and cache Regex object
#[macro_export]
macro_rules! regex {
    ($expr:expr) => {{
        static REGEX: ::once_cell::sync::Lazy<::regex::Regex> =
            ::once_cell::sync::Lazy::new(|| ::regex::Regex::new($expr).unwrap());
        &REGEX
    }};
    ($expr:expr,) => {
        regex!($expr)
    };
}

#[macro_export]
macro_rules! select {
    ($selectors:literal) => {{
        static SELECTOR: ::once_cell::sync::Lazy<::scraper::selector::Selector> =
            ::once_cell::sync::Lazy::new(|| {
                ::scraper::selector::Selector::parse($selectors).unwrap()
            });
        &SELECTOR
    }};
    ($selectors:literal,) => {
        selector!($selectors)
    };
}

/// Check if an expression matches a refutable pattern.
///
/// Syntax: `matches!(` *expression* `,` *pattern* `)`
///
/// Return a boolean, true if the expression matches the pattern, false otherwise.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate acick_util;
///
/// pub enum Foo<T> {
///     A,
///     B(T),
/// }
///
/// impl<T> Foo<T> {
///     pub fn is_a(&self) -> bool {
///         matches!(*self, Foo::A)
///     }
///
///     pub fn is_b(&self) -> bool {
///         matches!(*self, Foo::B(_))
///     }
/// }
///
/// # fn main() { }
/// ```
#[macro_export]
macro_rules! matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => true,
            _ => false
        }
    }
}

/// Assert that an expression matches a refutable pattern.
///
/// Syntax: `assert_matches!(` *expression* `,` *pattern* `)`
///
/// Panic with a message that shows the expression if it does not match the
/// pattern.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate acick_util;
///
/// fn main() {
///     let data = [1, 2, 3];
///     assert_matches!(data.get(1), Some(_));
/// }
/// ```
#[macro_export]
macro_rules! assert_matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => (),
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($pattern)+)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_regex() {
        // check multiple regex! calls creates only one instance and caches it
        let regs: Vec<_> = (0..2).map(|_| regex!(r"\A(hello)+\z")).collect();
        assert_eq!(regs[0] as *const _, regs[1] as *const _);
    }

    #[test]
    fn test_select() {
        // check multiple select! calls creates only one instance and caches it
        let selects: Vec<_> = (0..2).map(|_| select!("div a")).collect();
        assert_eq!(selects[0] as *const _, selects[1] as *const _);
    }
}
