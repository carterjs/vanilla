use std::{borrow::Borrow, io::Write as _};

use crate::{vm::{VM, self}, callable::Callable, types::Type, value::{Value, Object}};

use super::BuiltIn;

pub struct Write;
impl Callable for Write {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error> {
        let value = vm.pop()?;
        let path = vm.pop()?;

        match (path.borrow(), value.borrow()) {
            (Value::Object(Object::String(path)), Value::Object(Object::String(value))) => {
                // Open file at path and write value to it
                let mut file = std::fs::OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .create(true)
                    .open(path)
                    .expect("Something went wrong opening the file");
                
                file.write_all(value.as_bytes()).expect("Something went wrong writing to the file");
                file.flush().expect("Something went wrong flushing the file");
            },
            _ => {
                return Err(vm::Error::RuntimeError(format!("Expected a string and a string, got {:?} and {:?}", path, value)));
            }
        }

        Ok(())
    }
}
impl BuiltIn for Write {
    fn get_name(&self) -> &str {
        "write"
    }
    fn get_type(&self) -> Type {
        Type::Function(vec![Type::String, Type::Any], Box::from(Type::Nil))
    }
}