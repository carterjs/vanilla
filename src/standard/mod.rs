use crate::{callable::Callable, types::Type};

pub mod print;
pub mod write;
pub mod arrays;

pub trait BuiltIn: Callable {
    fn get_name(&self) -> &str;
    fn get_type(&self) -> Type;
}

pub fn get_functions() -> &'static [&'static dyn BuiltIn] {
    &[
        &print::Print,
        &print::Println,
        &write::Write,
        &arrays::Map,
        &arrays::Loop,
        &arrays::Length
    ]
}