use super::Representations;

pub trait Payload: Sized + 'static {
    fn write(&self, out: &mut Representations);
    fn read(source: &Representations) -> Option<Self>;

    fn contains(source: &Representations) -> bool {
        Self::read(source).is_some()
    }
}
