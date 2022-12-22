pub trait RelativeFile {
    fn resolve(self, cwd: &str) -> Self;
}
