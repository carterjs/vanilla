use std::borrow::Borrow;
use std::{fmt};

use std::fmt::Formatter;

use crate::standard::{self};
use crate::value::{Value, Object};

// TODO: maybe use a macro to define these?
pub const POP: u8 = 0;
pub const PUSH: u8 = 1;
pub const PUSH_LOCAL: u8 = 2;
pub const POP_LOCAL: u8 = 3;
pub const GET_LOCAL: u8 = 4;
pub const GET_UPVALUE: u8 = 5;
pub const GET_GLOBAL: u8 = 6;
pub const CALL: u8 = 7;
pub const MAKE_ARRAY: u8 = 8;
pub const MAKE_BLOCK: u8 = 9;
pub const INDEX: u8 = 10;
pub const MAKE_CLOSURE: u8 = 11;
pub const OR: u8 = 12;
pub const AND: u8 = 13;
pub const ADD: u8 = 14;
pub const CONCATENATE: u8 = 15;
pub const SUBTRACT: u8 = 16;
pub const MULTIPLY: u8 = 17;
pub const DIVIDE: u8 = 18;
pub const EQUAL: u8 = 19;
pub const NOT_EQUAL: u8 = 20;
pub const GREATER_THAN: u8 = 21;
pub const GREATER_THAN_EQUAL: u8 = 22;
pub const LESS_THAN: u8 = 23;
pub const LESS_THAN_EQUAL: u8 = 24;
pub const NEGATE: u8 = 25;
pub const NOT: u8 = 26;
pub const JUMP: u8 = 27;
pub const JUMP_IF_FALSE: u8 = 28;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn read_u16(&self, i: &mut usize) -> u16 {
        let v = u16::from_be_bytes(self.code[*i..*i+2].try_into().unwrap_or_default());
        *i += 2;
        v
    }

    pub fn read_i32(&self, i: usize) -> i32 {
        i32::from_le_bytes(self.code[i..i + 4].try_into().unwrap_or_default())
    }

    pub fn read_bool(&self, i: usize) -> bool {
        self.code[i] == 1
    }

    pub fn write(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn write_pair(&mut self, op: u8, index: u16) {
        self.code.push(op);
        self.code.extend(u16::to_be_bytes(index));
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let globals = standard::get_functions().iter().map(|f| f.get_name()).collect::<Vec<_>>();
        let mut i = 0;
        while i < self.code.len() {
            write!(f, "{:<5}| ", i)?;
            i += 1;
            match self.code[i - 1] {
                POP => {
                    writeln!(f, "POP")?;
                },
                PUSH => {
                    let value = self.constants[self.read_u16(&mut i) as usize].clone();
                    writeln!(f, "{:16}{:?}", "PUSH", value.clone())?;

                    // Print out the value if it's a function or closure
                    match value {
                        Value::Object(o) => {
                            match o.borrow() {
                                Object::Function(func) => {
                                    let n = if func.name.len() > 0 {
                                        format!("function \"{}\"", &func.name)
                                    } else {
                                        "anonymous function".to_string()
                                    };
                                    writeln!(f, "(entering {})", n)?;
                                    write!(f, "{:?}", func.chunk)?;
                                    writeln!(f, "(exiting {})", n)?;
                                },
                                crate::value::Object::Closure(c) => {
                                    let n = if c.function.name.len() > 0 {
                                        format!("{}", &c.function.name)
                                    } else {
                                        "anonymous closure".to_string()
                                    };
                                    writeln!(f, "(entering {})", n)?;
                                    write!(f, "{:?}", c.function.chunk)?;
                                    writeln!(f, "(exiting {})", n)?;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },
                PUSH_LOCAL =>  writeln!(f, "PUSH_LOCAL")?,
                POP_LOCAL => writeln!(f, "POP_LOCAL")?,
                GET_LOCAL => writeln!(f, "{:16}{}", "GET_LOCAL", self.read_u16(&mut i))?,
                GET_UPVALUE => writeln!(f, "{:16}{}", "GET_UPVALUE", self.read_u16(&mut i))?,
                GET_GLOBAL => writeln!(f, "{:16}{}", "GET_GLOBAL", globals[self.read_u16(&mut i) as usize])?,
                CALL => writeln!(f, "{:16}{}", "CALL", self.read_u16(&mut i))?,
                MAKE_ARRAY => writeln!(f, "{:16}{}", "MAKE_ARRAY", self.read_u16(&mut i))?,
                MAKE_BLOCK => writeln!(f, "{:16}{}", "MAKE_BLOCK", self.read_u16(&mut i))?,
                INDEX => writeln!(f, "INDEX")?,
                MAKE_CLOSURE => writeln!(f, "MAKE_CLOSURE")?,
                OR => writeln!(f, "OR")?,
                AND => writeln!(f, "AND")?,
                ADD => writeln!(f, "ADD")?,
                CONCATENATE => writeln!(f, "{:16}{}", "CONCATENATE", self.read_u16(&mut i))?,
                SUBTRACT => writeln!(f, "SUBTRACT")?,
                MULTIPLY => writeln!(f, "MULTIPLY")?,
                DIVIDE => writeln!(f, "DIVIDE")?,
                EQUAL => writeln!(f, "EQUAL")?,
                NOT_EQUAL => writeln!(f, "NOT_EQUAL")?,
                GREATER_THAN => writeln!(f, "GREATER_THAN")?,
                GREATER_THAN_EQUAL => writeln!(f, "GREATER_THAN_EQUAL")?,
                LESS_THAN => writeln!(f, "LESS_THAN")?,
                LESS_THAN_EQUAL => writeln!(f, "LESS_THAN_EQUAL")?,
                NEGATE => writeln!(f, "NEGATE")?,
                NOT => writeln!(f, "NOT")?,
                JUMP => writeln!(f, "{:16}{}", "JUMP", self.read_u16(&mut i))?,
                JUMP_IF_FALSE => writeln!(f, "{:16}{}", "JUMP_IF_FALSE", self.read_u16(&mut i))?,
                _ => writeln!(f, "UNKNOWN")?,
            }
        }

        Ok(())
    }
}