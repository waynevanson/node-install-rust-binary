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
