use std::any::Any;

/// A cacheable type must be mappable to an 'any', and provide its size
///
/// Note that Any requires 'static, so we require that here too
pub trait Cacheable: Any + Sync + Send + 'static {
    fn as_any(&self) -> &dyn Any;
    fn size(&self) -> usize;
}
