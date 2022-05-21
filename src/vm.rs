use std::{rc::Rc, fmt, borrow::Borrow};

use crate::{op::{self}, value::{Value, Object}, compiler::{self, compile, Function}, standard::{self, BuiltIn}, callable::Callable};

#[derive(Debug)]
pub enum Error {
    ParseError(compiler::Error),
    FrameStackUnderflow,
    ValueStackUnderflow,
    ValueStackOverflow,
    InvalidStackIndex(usize),
    IndexOutOfBounds(i32, usize),
    RuntimeError(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "{:?}", e),
            Error::FrameStackUnderflow => write!(f, "Frame stack underflow"),
            Error::ValueStackUnderflow => write!(f, "Value stack underflow"),
            Error::ValueStackOverflow => write!(f, "Value stack overflow"),
            Error::InvalidStackIndex(i) => write!(f, "Invalid stack index {}", i),
            Error::IndexOutOfBounds(i, s) => write!(f, "Index {} out of bounds for array of length {}", i, s),
            Error::RuntimeError(s) => write!(f, "Runtime error: {}", s)
        }
    }
}

pub struct Closure {
    pub(crate) function: Rc<Function>,
    pub upvalues: Vec<Value>
}

impl Callable for Closure {
    fn call(&self, vm: &mut VM) -> Result<(), self::Error> {
        vm.run(&self.function, self.upvalues.clone())
    }
}

pub struct VM {
    stack: Vec<Value>,
    locals: Vec<Value>,
    globals: Vec<Rc<&'static dyn BuiltIn>>,
}

impl VM {
    pub fn new() -> Self {  
        let globals: Vec<Rc<&'static dyn BuiltIn>> = standard::get_functions().iter().map(|f| Rc::new(f.clone())).collect();         

        Self {
            stack: Vec::new(),
            locals: Vec::new(),
            globals,
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<(), Error> {
        let function = match compile(source) {
            Ok(c) => c,
            Err(e) => return Err(Error::ParseError(e))
        };

        println!("Binary representation: ");
        for b in &function.chunk.code {
            println!("{:#04X} ", b);
        }
        println!("");

        println!("Disassembled Bytecode:");
        println!("{:?}", function.chunk);

        self.run(&function, Vec::new())?;

        Ok(())
    }

    pub(crate) fn push(&mut self, value: Value) -> Result<(), Error> {
        Ok(self.stack.push(value))
    }

    pub(crate) fn pop(&mut self) -> Result<Value, Error> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err(Error::ValueStackUnderflow)
        }
    }

    // TODO: try to create a macro for binary operations
    // TODO: try to create macros for incrementing the ip too
    pub fn run(&mut self, function: &Function, upvalues: Vec<Value>) -> Result<(), Error> {  

        let base = self.locals.len();

        let mut ip = 0;
        while ip < function.chunk.code.len() {
            ip += 1;
            match function.chunk.code[ip - 1] {
                op::POP => {
                    self.pop()?;
                },
                op::PUSH => {
                    let i = function.chunk.read_u16(&mut ip);

                    // Take a constant from the chunk's constant pool and push it onto the stack
                    let constant = function.chunk.constants[i as usize].clone();
                    self.push(constant)?;
                },
                op::GET_LOCAL => {
                    let i = function.chunk.read_u16(&mut ip);
                    let v = self.locals[base + i as usize].clone();

                    self.push(v.clone())?;
                },
                op::GET_UPVALUE => {
                    let i = function.chunk.read_u16(&mut ip);
                    let v = upvalues[i as usize].clone();
                    self.push(v)?;
                },
                op::PUSH_LOCAL => {
                    let v = self.pop()?;
                    self.locals.push(v);
                },
                op::POP_LOCAL => {
                    self.locals.pop();
                },
                op::GET_GLOBAL => {
                    let i = function.chunk.read_u16(&mut ip);
                    let built_in = &self.globals[i as usize];
                    let v = Value::Object(Object::BuiltIn(built_in.clone()));

                    self.push(v)?;
                },
                op::CALL => {
                    let arity = function.chunk.read_u16(&mut ip);

                    let func = self.stack.remove(self.stack.len() - 1 - arity as usize);
                    match func.borrow() {
                        Value::Object(h) => {
                            match h.borrow() {
                               Object::BuiltIn(b) => {
                                    b.call(self)?;
                               },
                               Object::Closure(c) => {
                                    c.call(self)?;
                               },
                               Object::Function(f) => {
                                    f.call(self)?;
                               },
                               t => {
                                    return Err(Error::RuntimeError(format!("Cannot call {:?}", t)));
                               }
                            }
                        },
                        _ => return Err(Error::RuntimeError("Call on non-callable value".to_string()))
                    }
                },
                op::MAKE_ARRAY => {
                    let n = function.chunk.read_u16(&mut ip);

                    let values: Vec<Value> = self.stack.drain(self.stack.len() - n as usize..).collect();
                    self.push(Value::Object(Object::Array(values)))?;
                },
                op::MAKE_BLOCK => {
                    let n = function.chunk.read_u16(&mut ip);

                    let values: Vec<Value> = self.stack.drain(self.stack.len() - n as usize..).collect();
                    self.push(Value::Object(Object::Block(values)))?;
                },
                // TODO: add a variant of this that doesn't need the stack
                op::INDEX => {
                    let i = self.pop()?;
                    let target = self.pop()?;

                    match (target.borrow(), i.borrow()) {
                        (Value::Object(h), Value::Number(n)) => match h.borrow() {
                            Object::Array(l) => {
                                if *n < 0 || *n as usize >= l.len() {
                                    return Err(Error::IndexOutOfBounds(*n, l.len()));
                                }

                                let v = l[*n as usize].clone();
                                self.push(v)?;
                            },
                            Object::Block(o) => {
                                let v = o[*n as usize].clone();
                                self.push(v)?;
                            },
                            _ => {
                                return Err(Error::RuntimeError(format!("Cannot index a non-indexable value: {:?}", target)))
                            }
                        },
                        _ => {
                            return Err(Error::RuntimeError(format!("Cannot index a non-indexable primitive: {:?}", target)))
                        }
                    };
                },
                op::MAKE_CLOSURE => {
                    let func = self.pop()?;

                    match func.borrow() {
                        Value::Object(h) => match h.borrow(){
                            Object::Function(f) => {
                                let mut closure = Closure {
                                    function: f.clone(),
                                    upvalues: Vec::new()
                                };
    
                                // Resolve the upvalues
                                for upvalue in &f.upvalues {
                                    if upvalue.is_local {
                                        closure.upvalues.push(self.locals[base + upvalue.index as usize].clone());
                                    } else {
                                        closure.upvalues.push(upvalues[upvalue.index as usize].clone());
                                    }
                                }
    
                                self.push(Value::Object(Object::Closure(Rc::new(closure))))?;
                            },
                            _ => {
                                return Err(Error::RuntimeError(format!("Cannot make closure from non-function: {:?}", func)))
                            }
                        },
                        t => {
                            return Err(Error::RuntimeError(format!("Cannot make closure from non-function: {:?}", t)))
                        }
                    };
                },
                op::OR => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Value::Boolean(b1), Value::Boolean(b2)) => self.push(Value::Boolean(b1 || b2))?,
                        _ => return Err(Error::RuntimeError("Cannot OR non-boolean values".to_string()))
                    }
                },
                op::AND => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Value::Boolean(b1), Value::Boolean(b2)) => self.push(Value::Boolean(b1 && b2))?,
                        _ => return Err(Error::RuntimeError("Cannot AND non-boolean values".to_string()))
                    }
                },
                op::ADD => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a + b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't add {:?} and {:?}", a, b)))
                    };
                },
                op::CONCATENATE => {
                    let n = function.chunk.read_u16(&mut ip);

                    let s: String = self.stack.drain(self.stack.len() - n as usize..).map(|v| v.to_string()).collect();
                    self.push(Value::Object(Object::String(s)))?;
                },
                op::SUBTRACT => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a - b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't subtract {:?} and {:?}", a, b)))
                    };
                },
                op::MULTIPLY => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Number(a * b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't multiply {:?} and {:?}", a, b)))
                    };
                },
                op::DIVIDE => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => {
                            if *b == 0 {
                                return Err(Error::RuntimeError(format!("Can't divide {} by 0", a)))
                            }
                            self.push(Value::Number(a / b))?
                        },
                        _ => return Err(Error::RuntimeError(format!("Can't divide {:?} and {:?}", a, b)))
                    };
                },
                op::EQUAL => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a == b))?,
                        (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a == b))?,
                        (Value::Object(_h1), Value::Object(_h2)) => {
                            todo!("comparing non-primitives");
                        },
                        _ => self.push(Value::Boolean(false))?
                    };
                },
                op::NOT_EQUAL => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a != b))?,
                        (Value::Boolean(a), Value::Boolean(b)) => self.push(Value::Boolean(a != b))?,
                        (Value::Object(_h1), Value::Object(_h2)) => {
                            todo!("comparing non-primitives");
                        },
                        _ => self.push(Value::Boolean(true))?
                    };
                },
                op::GREATER_THAN => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a > b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't compare {:?} and {:?}", a, b)))
                    };
                },
                op::GREATER_THAN_EQUAL => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a >= b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't compare {:?} and {:?}", a, b)))
                    };
                },
                op::LESS_THAN => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a < b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't compare {:?} and {:?}", a, b)))
                    };
                },
                op::LESS_THAN_EQUAL => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a.borrow(), b.borrow()) {
                        (Value::Number(a), Value::Number(b)) => self.push(Value::Boolean(a <= b))?,
                        _ => return Err(Error::RuntimeError(format!("Can't compare {:?} and {:?}", a, b)))
                    };
                },
                op::NEGATE => {
                    let a = self.pop()?;
                    match a.borrow() {
                        Value::Number(a) => self.push(Value::Number(-a))?,
                        _ => return Err(Error::RuntimeError(format!("Can't negate {:?}", a)))
                    };
                },
                op::NOT => {
                    let a = self.pop()?;
                    match a.borrow() {
                        Value::Boolean(a) => self.push(Value::Boolean(!a))?,
                        _ => return Err(Error::RuntimeError(format!("Can't negate {:?}", a)))
                    };
                },
                op::JUMP => {
                    let offset = function.chunk.read_u16(&mut ip) - 2;
                    ip += offset as usize;
                },
                op::JUMP_IF_FALSE => {
                    let offset = function.chunk.read_u16(&mut ip) - 2;
                    let condition = self.pop()?;
                    match condition.borrow() {
                        Value::Boolean(false) => {
                            ip += offset as usize;
                        },
                        _ => ()
                    }
                },
                o => {
                    return Err(Error::RuntimeError(format!("Unknown opcode: {}", o)))
                }
            };

        }

        Ok(())

    }
}