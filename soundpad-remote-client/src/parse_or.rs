use std::str::FromStr;

pub trait ParseOr<T> {
    fn parse_or(self, default: T) -> T;
}
pub trait ParseOrDefault<T> {
    fn parse_or_default(self) -> T;
}

impl<T, S> ParseOrDefault<T> for S
where
    T: Default,
    S: ParseOr<T>,
{
    fn parse_or_default(self) -> T {
        self.parse_or(T::default())
    }
}

impl<T: FromStr> ParseOr<T> for &str {
    fn parse_or(self, default: T) -> T {
        self.parse().unwrap_or(default)
    }
}

impl<T, Y> ParseOr<T> for Option<Y>
where
    Y: ParseOr<T>,
{
    fn parse_or(self, default: T) -> T {
        match self {
            Some(s) => s.parse_or(default),
            None => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_num() {
        assert_eq!(12, "12".parse_or(0));
        assert_eq!(31, "NaN".parse_or(31))
    }

    #[test]
    fn parse_num_default() {
        assert_eq!(12, "12".parse_or_default());
        assert_eq!(0, "NaN".parse_or_default())
    }

    #[test]
    fn parse_option_some_num() {
        assert_eq!(31, Some("31").parse_or_default());
        assert_eq!(0, Some("ASDF").parse_or_default())
    }

    #[test]
    fn parse_option_none_num() {
        let option: Option<&str> = None;
        assert_eq!(0, option.parse_or_default())
    }
}
