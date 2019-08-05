/// This trait guarantees an object that implements it can return it's
/// own size.
pub trait SizedObject {
    fn size(&self) -> u64;
}
