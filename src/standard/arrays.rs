use std::borrow::Borrow;

use crate::{callable::Callable, vm::{self, VM}, value::{Value, Object}, types::Type};

use super::BuiltIn;

pub struct Map;

// TODO: figure out how to make this more efficient with functions and closures
impl Callable for Map {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error> {
        let function = vm.pop()?;
        let list = vm.pop()?;

        match (list.borrow(), function.borrow()) {
            (Value::Object(Object::Array(list)), Value::Object(Object::Function(function))) => {
                let mut new_list = Vec::new();
                for (i, v) in list.iter().enumerate() {
                    vm.push(v.clone())?;
                    vm.push(Value::Number(i as i32))?;

                    vm.run(function, vec![])?;

                    let mapped_value = vm.pop()?;
                    new_list.push(mapped_value);
                }

                vm.push(Value::Object(Object::Array(new_list)))?;
            },
            (Value::Object(Object::Array(list)), Value::Object(Object::Closure(closure))) => {
                let mut new_list = Vec::new();
                for (i, v) in list.iter().enumerate() {
                    vm.push(v.clone())?;
                    vm.push(Value::Number(i as i32))?;

                    vm.run(closure.function.borrow(), closure.upvalues.clone())?;

                    let mapped_value = vm.pop()?;
                    new_list.push(mapped_value);
                }

                vm.push(Value::Object(Object::Array(new_list)))?;
            },
            _ => return Err(vm::Error::RuntimeError("map requires a list and a function".to_string())),
        }
        
        Ok(())
    }
}

impl BuiltIn for Map {
    fn get_name(&self) -> &str {
        "map"
    }
    fn get_type(&self) -> crate::types::Type {
        Type::Function(vec![Type::Array(Box::from(Type::Any)), Type::Function(vec![Type::Any, Type::Number], Box::from(Type::Any))], Box::from(Type::Array(Box::from(Type::Any))))
    }
}

pub struct Loop;
impl Callable for Loop {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error> {
        let function = vm.pop()?;
        let list = vm.pop()?;

        match (list.borrow(), function.borrow()) {
            (Value::Object(Object::Array(list)), Value::Object(Object::Function(function))) => {
                for (i, v) in list.iter().enumerate() {
                    vm.push(v.clone())?;
                    vm.push(Value::Number(i as i32))?;

                    vm.run(function, vec![])?;
                }
            },
            (Value::Object(Object::Array(list)), Value::Object(Object::Closure(closure))) => {
                for (i, v) in list.iter().enumerate() {
                    vm.push(v.clone())?;
                    vm.push(Value::Number(i as i32))?;

                    vm.run(closure.function.borrow(), closure.upvalues.clone())?;
                }
            },
            _ => return Err(vm::Error::RuntimeError("map requires a list and a function".to_string())),
        }
        
        Ok(())
    }
}

impl BuiltIn for Loop {
    fn get_name(&self) -> &str {
        "loop"
    }
    fn get_type(&self) -> crate::types::Type {
        Type::Function(vec![Type::Array(Box::from(Type::Any)), Type::Function(vec![Type::Any, Type::Number], Box::from(Type::Any))], Box::from(Type::Nil))
    }
}

pub struct Length;

impl Callable for Length {
    fn call(&self, vm: &mut VM) -> Result<(), vm::Error> {
        let list = vm.pop()?;

        match list.borrow() {
            Value::Object(Object::Array(list)) => {
                vm.push(Value::Number(list.len() as i32))?;
            },
            Value::Object(Object::String(s)) => {
                vm.push(Value::Number(s.len() as i32))?;
            },
            _ => return Err(vm::Error::RuntimeError("length requires a list".to_string())),
        }

        Ok(())
    } 
}

impl BuiltIn for Length {
    fn get_name(&self) -> &str {
        "length"
    }
    fn get_type(&self) -> crate::types::Type {
        Type::Function(vec![Type::Any], Box::from(Type::Number))
    }
}