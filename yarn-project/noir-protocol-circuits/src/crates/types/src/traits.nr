// Preferred over Default trait to convey intent, as default doesn't necessarily mean empty.
trait Empty {
    fn empty() -> Self;
}

trait Hash {
    fn hash(self) -> Field;
}
