use std::{fmt, rc::Rc};

use crate::{compiler::Function, vm::Closure, standard::BuiltIn};

pub enum Object {
    String(String),
    Array(Vec<Value>),
    Block(Vec<Value>),
    Function(Rc<Function>),
    Closure(Rc<Closure>),
    BuiltIn(Rc<&'static dyn BuiltIn>),
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "String({})", s),
            Self::Array(l) => write!(f, "Array({:?})", l),
            Self::Block(o) => write!(f, "Block({:?})", o),
            Self::Function(func) => write!(f, "Function({})", func.name),
            Self::Closure(c) => write!(f, "Closure({})", c.function.name),
            Self::BuiltIn(_b) => write!(f, "BuiltIn"),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Array(l) => {
                for value in l.iter() {
                    write!(f, "{}", value)?;
                }
                Ok(())
            },
            Self::Block(_o) => write!(f, "(block)"),
            Self::Function(_f) => write!(f, "(function)"),
            Self::Closure(_c) => write!(f, "(closure)"),
            Self::BuiltIn(_b) => write!(f, "(built in)"),
        }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Self::String(s) => Self::String(s.clone()),
            Self::Array(l) => Self::Array(l.clone()),
            Self::Block(o) => Self::Block(o.clone()),
            Self::Function(f) => Self::Function(f.clone()),
            Self::Closure(c) => Self::Closure(c.clone()),
            Self::BuiltIn(b) => Self::BuiltIn(b.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i32),
    Boolean(bool),
    Object(Object),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Object(h) => write!(f, "{}", h),
        }
    }
}