//a Imports
use std::any::Any;

//a Cacheable
//tt Cacheable
/// Note that Any requires 'static, so we require that here too
pub trait Cacheable: Any + Sync + Send + 'static {
    fn as_any(&self) -> &dyn Any;
    fn size(&self) -> usize;
}
