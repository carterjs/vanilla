use crate::{vm::{VM, self}, callable::Callable, types::Type};

use super::BuiltIn;

pub struct Print;
impl Callable for Print {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error> {
        let value = vm.pop()?;

        print!("{}", value);

        Ok(())
    }
}
impl BuiltIn for Print {
    fn get_name(&self) -> &str {
        "print"
    }
    fn get_type(&self) -> Type {
        Type::Function(vec![Type::Any], Box::from(Type::Nil))
    }
}

pub struct Println;
impl Callable for Println {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error> {
        let value = vm.pop()?;

        println!("{}", value);

        Ok(())
    }
}
impl BuiltIn for Println {
    fn get_name(&self) -> &str {
        "println"
    }
    fn get_type(&self) -> crate::types::Type {
        Type::Function(vec![Type::Any], Box::from(Type::Nil))
    }
}