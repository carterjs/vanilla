use crate::{vm::{VM, self}};

pub trait Callable {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error>;
}