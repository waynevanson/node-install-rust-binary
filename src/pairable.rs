#[derive(Debug, PartialEq, PartialOrd)]
pub struct Pair<A, B>(A, B);

impl<A, B> Pair<A, B> {
    pub fn map_first<F, C>(self, f: F) -> Pair<C, B>
    where
        F: FnOnce(A) -> C,
    {
        Pair(f(self.0), self.1)
    }

    pub fn fmap_second<F, C>(self, f: F) -> Pair<A, C>
    where
        F: FnOnce(B) -> C,
    {
        Pair(self.0, f(self.1))
    }
}

impl<A, B> From<Pair<A, B>> for (A, B) {
    fn from(pair: Pair<A, B>) -> Self {
        (pair.0, pair.1)
    }
}

impl<A, B> From<(A, B)> for Pair<A, B> {
    fn from(tuple: (A, B)) -> Self {
        Self(tuple.0, tuple.1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_convert_from_tuple_to_pair() {
        let first = "first";
        let second = "second";
        let result = Pair::from((first, second));
        let expected = Pair(first, second);
        assert_eq!(result, expected);
    }

    #[test]
    fn should_convert_from_pair_to_tuple() {
        let first = "first";
        let second = "second";
        let result: (_, _) = Pair(first, second).into();
        let expected = (first, second);
        assert_eq!(result, expected);
    }

    #[test]
    fn should_map_the_first_argument_only() {
        let first = 4;
        let second = 9;
        let result = Pair(first, second).map_first(|number| number * number);
        let expected = Pair(16, second);
        assert_eq!(result, expected);
    }

    #[test]
    fn should_map_the_second_argument_only() {
        let first = 4;
        let second = 9;
        let result = Pair(first, second).fmap_second(|number| number * number);
        let expected = Pair(first, 81);
        assert_eq!(result, expected);
    }
}
