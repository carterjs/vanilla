use std::{iter::Peekable, rc::Rc};

use crate::{op::{self, Chunk}, scanner::Scanner, token::{TokenValue, Token}, types::Type, callable::Callable, standard, value::{Value, Object}};

#[derive(Debug)]
pub enum ErrorValue {
    UnexpectedEOF,
    InvalidTypeAnnotation(String),
    BranchTypeMismatch(Type, Type),
    ArgumentTypeMismatch(Type, Type),
    ListItemTypeMismatch(Type, Type),
    TypeMismatch(Type, Type),
    UnexpectedToken(Token),
    RecursiveCall(String),
    InvalidGetTarget(Type),
    InvalidGetIdentifier(String)
}
#[derive(Debug)]
pub struct Error {
    pub value: ErrorValue,
    pub line: usize,
}

impl Error {
    pub fn new(value: ErrorValue, line: usize) -> Self {
        Self {
            value,
            line
        }
    }
}

pub fn compile<'source>(source: String) -> Result<Function, Error> {
    // Construct the compiler
    let mut compiler = Compiler::new(source);

    compiler.compile()
}

#[derive(Debug)]
pub struct Local {
    name: String,
    index: usize,
    depth: usize,
    type_: Type,
}


impl Clone for Local {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            index: self.index,
            depth: self.depth,
            type_: self.type_.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Upvalue {
    pub index: usize,
    pub is_local: bool
}

#[derive(Debug)]
struct Global {
    name: String,
    type_: Type,
}

impl Clone for Global {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            type_: self.type_.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub chunk: Chunk,
    pub locals: Vec<Local>,
    pub upvalues: Vec<Upvalue>,
    pub depth: usize,
}

impl Function {
    fn new(name: String, chunk: Chunk) -> Self {
        Self {
            name,
            chunk,
            locals: Vec::new(),
            upvalues: Vec::new(),
            depth: 0,
        }
    }

    fn add_local(&mut self, name: String, type_: Type) -> usize {
        let index = self.locals.len();

        self.locals.push(Local {
            name,
            index,
            depth: self.depth,
            type_,
        });

        index
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        for (i, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i;
            }
        }

        self.upvalues.push(Upvalue {
            index,
            is_local,
        });

        self.upvalues.len() - 1
    }

    fn begin_scope(&mut self) {
        self.depth += 1;
    }

    fn end_scope(&mut self) {
        self.depth -= 1;
        
        // Pop all locals with a greater depth
        self.locals.retain(|local| {
            if local.depth > self.depth {
                self.chunk.write(op::POP_LOCAL);
                false
            } else {
                true
            }
        });
    }

    fn resolve(&self, name: &str) -> Option<&Local> {
        for (_i, local) in self.locals.iter().enumerate() {
            if local.name == name {
                return Some(local);
            }
        }
        return None;
    }
}

impl Callable for Function {
    fn call(&self, vm: &mut crate::vm::VM) -> Result<(), crate::vm::Error> {
        vm.run(self, Vec::new())
    }
}

struct Compiler {
    tokens: Peekable<Scanner>,
    functions: Vec<Function>,
    globals: Vec<Global>,
    last_type: Type,
    unresolved_types: Vec<String>,
    line: usize
}

impl Compiler {
    fn new(source: String) -> Compiler {
        Compiler {
            tokens: Scanner::new(source).peekable(),
            functions: Vec::new(),
            globals: standard::get_functions().iter().map(|f| Global {
                name: f.get_name().to_string(),
                type_: f.get_type(),
            }).collect(),
            last_type: Type::Nil,
            unresolved_types: Vec::new(),
            line: 1
        }
    }

    fn get_function(&mut self) -> &mut Function {
        self.functions.last_mut().unwrap()
    }

    fn ignore_whitespace(&mut self) {
        while self.take(TokenValue::Newline).is_some() { }
    }

    fn peek(&mut self) -> Option<&Token> {
       self.tokens.peek()
    }

    fn next(&mut self) -> Option<Token> {
        match self.tokens.next() {
            Some(token) => {
                self.line = token.line;
                Some(token)
            },
            None => None
        }
    }

    fn has(&mut self, value: TokenValue) -> bool {
        self.peek().map(|token| token.value == value).unwrap_or(false)
    }

    fn take(&mut self, value: TokenValue) -> Option<Token> {
        if self.has(value) {
            self.next()
        } else {
            None
        }
    }

    fn take_any(&mut self, values: Vec<TokenValue>) -> Option<Token> {
        for value in values {
            if let Some(token) = self.take(value) {
                return Some(token);
            }
        }

        None
    }

    fn error(&mut self, value: ErrorValue) -> Error {
        Error::new(value, self.line)
    }

    fn assert_type(&mut self, expected: Type) -> Result<(), Error> {
        if self.last_type == Type::Unknown {
            let n = self.unresolved_types.pop().unwrap();

            for local in self.get_function().locals.iter_mut().rev() {
                if local.name == n {
                    local.type_ = expected.clone();
                }
            }
        } else if !self.last_type.satisfies(expected.clone()) {
            return Err(self.error(ErrorValue::TypeMismatch(self.last_type.clone(), expected)));
        }

        self.last_type = expected;

        Ok(())
    }

    fn assert_type_with_error(&mut self, expected: Type, error: Error) -> Result<(), Error> {
        if let Ok(()) = self.assert_type(expected.clone()) {
            Ok(())
        } else {
            Err(error)
        }
    }
    
    fn compile(&mut self) -> Result<Function, Error> {
        self.functions.push(Function::new(String::from(""), Chunk::new()));

        while self.peek().is_some() {
            self.ignore_whitespace();
            self.expression(false)?;
            self.ignore_whitespace();
        }

        Ok(self.functions.pop().unwrap())
    }

    fn expression(&mut self, keep: bool) -> Result<(), Error> {
        self.or()?;

        // Automatically call functions
        match &self.last_type {
            Type::Function(_params, _return_type) => {
                self.execute_call()?;
            },
            _ => {}
        }

        if !keep && self.last_type != Type::Nil {
            self.get_function().chunk.write(op::POP);
        } 

        Ok(())
    }

    fn or(&mut self) -> Result<(), Error> {
        self.and()?;
        while self.take(TokenValue::Or).is_some() {
            self.assert_type(Type::Boolean)?;
            self.and()?;
            self.assert_type(Type::Boolean)?;
            self.get_function().chunk.write(op::OR)
        }
        Ok(())
    }

    fn and(&mut self) -> Result<(), Error> {
        self.equality()?;
        while self.take(TokenValue::And).is_some() {
            self.assert_type(Type::Boolean)?;
            self.equality()?;
            self.assert_type(Type::Boolean)?;
            self.get_function().chunk.write(op::AND);
        }
        Ok(())
    }

    fn equality(&mut self) -> Result<(), Error> {
        self.comparison()?;
        while let Some(t) = self.take_any(vec![TokenValue::BangEqual, TokenValue::EqualEqual]) {
            self.comparison()?;
            self.last_type = Type::Boolean;
            match t.value {
                TokenValue::BangEqual => self.get_function().chunk.write(op::NOT_EQUAL),
                TokenValue::EqualEqual => self.get_function().chunk.write(op::EQUAL),
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn comparison(&mut self) -> Result<(), Error> {
        self.addition()?;
        while let Some(t) = self.take_any(vec![TokenValue::GreaterThan, TokenValue::GreaterThanEqual, TokenValue::LessThan, TokenValue::LessThanEqual]) {
            self.assert_type(Type::Number)?;
            self.addition()?;
            self.assert_type(Type::Number)?;

            self.last_type = Type::Boolean;
            match t.value {
                TokenValue::GreaterThan => self.get_function().chunk.write(op::GREATER_THAN),
                TokenValue::GreaterThanEqual => self.get_function().chunk.write(op::GREATER_THAN_EQUAL),
                TokenValue::LessThan => self.get_function().chunk.write(op::LESS_THAN),
                TokenValue::LessThanEqual => self.get_function().chunk.write(op::LESS_THAN_EQUAL),
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn addition(&mut self) -> Result<(), Error> {
        self.multiplication()?;
        while let Some(t) = self.take_any(vec![TokenValue::Plus, TokenValue::Minus]) {
            self.assert_type(Type::Number)?;
            self.multiplication()?;
            self.assert_type(Type::Number)?;

            match t.value {
                TokenValue::Plus => self.get_function().chunk.write(op::ADD),
                TokenValue::Minus => self.get_function().chunk.write(op::SUBTRACT),
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn multiplication(&mut self) -> Result<(), Error> {
        self.unary()?;
        while let Some(t) = self.take_any(vec![TokenValue::Star, TokenValue::Slash]) {
            self.assert_type(Type::Number)?;
            self.unary()?;
            self.assert_type(Type::Number)?;
            match t.value {
                TokenValue::Star => self.get_function().chunk.write(op::MULTIPLY),
                TokenValue::Slash => self.get_function().chunk.write(op::DIVIDE),
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn unary(&mut self) -> Result<(), Error> {
        if let Some(t) = self.take_any(vec![TokenValue::Bang, TokenValue::Minus]) {
            self.get()?;
            match t.value {
                TokenValue::Bang => {
                    self.assert_type(Type::Boolean)?;
                    self.get_function().chunk.write(op::NOT)
                },
                TokenValue::Minus => {
                    self.assert_type(Type::Number)?;
                    self.get_function().chunk.write(op::NEGATE)
                },
                _ => unreachable!(),
            }
        } else {
            self.get()?;
        }

        Ok(())
    }

    fn get(&mut self) -> Result<(), Error> {
        self.primary()?;

        // TODO: find a cleaner way to do this
        'outer: while self.take(TokenValue::Dot).is_some() {
            let obj_type = self.last_type.clone();

            match obj_type {
                Type::Array(list_type) => {
                    // Write constant index
                    self.primary()?;
                    self.get_function().chunk.write(op::INDEX);

                    self.last_type = *list_type;
                },
                Type::Block(block) => {
                    // Get the unchecked identifier
                    if let Some(t) = self.next() {
                        if let TokenValue::Identifier(name) = t.value {
                            // Check if it's a local
                            for (i, (n, t)) in block.iter().enumerate() {
                                if n.eq(&name) {
                                    // Add numeric constant
                                    let i = self.get_function().chunk.add_constant(Value::Number(i as i32));
                                    self.get_function().chunk.write_pair(op::PUSH, i);
                                    self.get_function().chunk.write(op::INDEX);

                                    self.last_type = t.clone();

                                    // Execute call for functions
                                    match t {
                                        Type::Function(_, _) => {
                                            self.execute_call()?;
                                        },
                                        _ => {}
                                    }

                                    continue 'outer;
                                }
                            }

                            return Err(self.error(ErrorValue::InvalidGetIdentifier(name)));
                        } else {
                            return Err(self.error(ErrorValue::UnexpectedToken(t)));
                        }
                    } else {
                        return Err(self.error(ErrorValue::UnexpectedEOF));
                    }
                },
                t => {
                    if t == Type::Unknown {
                        continue;
                    }
                    return Err(self.error(ErrorValue::InvalidGetTarget(t)));
                }
            }
        }

        Ok(())
    }

    fn primary(&mut self) -> Result<(), Error> {
        if let Some(t) = self.next() {
            return match t.value {
                TokenValue::LeftParen => self.group(),
                TokenValue::LeftBracket => self.list(),
                TokenValue::LeftBrace => self.block(),
                TokenValue::String(s) => {
                    let constant = self.get_function().chunk.add_constant(Value::Object(Object::String(s)));
                    self.last_type = Type::String;
                    Ok(self.get_function().chunk.write_pair(op::PUSH, constant))
                },
                TokenValue::Number(n) => {
                    let constant = self.get_function().chunk.add_constant(Value::Number(n));
                    self.last_type = Type::Number;
                    Ok(self.get_function().chunk.write_pair(op::PUSH, constant))
                },
                TokenValue::Boolean(b) => {
                    let constant = self.get_function().chunk.add_constant(Value::Boolean(b));
                    self.last_type = Type::Boolean;
                    Ok(self.get_function().chunk.write_pair(op::PUSH, constant))
                },
                TokenValue::Identifier(s) => self.call(s),
                TokenValue::BackSlash => self.function(String::new()),
                TokenValue::If => {
                    self.if_(None)
                },
                TokenValue::For => {
                    self.for_()
                },
                _ => {
                    Err(self.error(ErrorValue::UnexpectedToken(t)))
                }
            };
        }

        return Err(self.error(ErrorValue::UnexpectedEOF));
    }

    fn if_(&mut self, type_: Option<Type>) -> Result<(), Error> {   
        let mut type_ = type_;

        // Already consumed the if
        self.expression(true)?;

        // Emit initial jump
        let then_jump = self.get_function().chunk.code.len();
        self.get_function().chunk.write_pair(op::JUMP_IF_FALSE, 0);

        // Then block
        self.expression(true)?;

        if let Some(t) = &type_ {
            // Assert type
            let error = self.error(ErrorValue::BranchTypeMismatch(t.clone(), self.last_type.clone()));
            self.assert_type_with_error(t.clone(), error)?;
        } else {
            // Set type
            type_ = Some(self.last_type.clone());
        }

        // Emit else jump
        let else_jump = self.get_function().chunk.code.len();
        self.get_function().chunk.write_pair(op::JUMP, 0);

        // Patch the initial jump
        let index_bytes = u16::to_be_bytes((self.get_function().chunk.code.len() - then_jump) as u16 - 1);
        self.get_function().chunk.code[then_jump + 1] = index_bytes[0];
        self.get_function().chunk.code[then_jump + 2] = index_bytes[1];

        // Handle optional else
        if self.take(TokenValue::Else).is_some() {
            self.expression(true)?;
            
            if let Some(t) = type_.clone() {
                // Assert type
                let error = self.error(ErrorValue::BranchTypeMismatch(t.clone(), self.last_type.clone()));
                self.assert_type_with_error(t.clone(), error)?;
            }
        }

        // Patch the else jump
        let index_bytes = u16::to_be_bytes((self.get_function().chunk.code.len() - else_jump) as u16 - 1);
        self.get_function().chunk.code[else_jump + 1] = index_bytes[0];
        self.get_function().chunk.code[else_jump + 2] = index_bytes[1];

        Ok(())
    }

    fn for_(&mut self) -> Result<(), Error> {
        // TODO: Compile body as a function and call it with a "Loop" opcode
        todo!("implement for")
    }

    fn resolve_upvalue(&mut self, name: &str) -> bool {
        // Resolve, flagging upvalues top to bottom
        for i in (0..(self.functions.len() - 1)).rev() {
            if let Some(local) = &self.functions[i].resolve(name).cloned() {
                let is_local = i >= self.functions.len() - 2;
                let upvalue_index = self.get_function().add_upvalue(local.index, is_local);
                self.last_type = local.type_.clone();
                self.get_function().chunk.write_pair(op::GET_UPVALUE, upvalue_index as u16);

                // Now we need to need to add that upvalue to all the intermediate ones
                let mut is_local = true;
                let mut index = local.index;
                for j in (i + 1)..self.functions.len() - 1 {
                    index = self.functions[j].add_upvalue(index, is_local);
                    is_local = false;
                }

                return true
            }
        }

        false
    }

    fn resolve_local(&mut self, name: &str) -> bool {
        if let Some(local) = self.get_function().resolve(name).cloned() {
            self.last_type = local.type_.clone();

            self.unresolved_types.push(name.to_string());
            self.get_function().chunk.write_pair(op::GET_LOCAL, local.index as u16);
            return true;
        }

        false
    }

    fn resolve_global(&mut self, name: &str) -> bool {
        let globals = self.globals.clone();
        for (i, g) in globals.iter().clone().enumerate() {
            if g.name == name {
                self.get_function().chunk.write_pair(op::GET_GLOBAL, i as u16);
                self.last_type = g.type_.clone();
                return true;
            }
        }

       false
    }

    fn call(&mut self, name: String) -> Result<(), Error> {
        if self.resolve_local(&name) || self.resolve_upvalue(&name) || self.resolve_global(&name){
            self.execute_call()
        } else if self.get_function().name == name {
            Err(self.error(ErrorValue::RecursiveCall(name)))
        } else {
            self.assignment(name)
        }
    }

    fn execute_call(&mut self) -> Result<(), Error> {
        match self.last_type.clone() {
            Type::Function(params, return_type) => {
                if self.has(TokenValue::Newline) | self.has(TokenValue::RightParen) | self.peek().is_none() {
                    // Not a call, just passing a function around
                    return Ok(())
                }

                for t in params.iter() {
                    self.expression(true)?;
                    if !self.last_type.satisfies(t.clone()) {
                        return Err(self.error(ErrorValue::ArgumentTypeMismatch(t.clone(), self.last_type.clone())));
                    }
                }

                self.get_function().chunk.write_pair(op::CALL, params.len() as u16);

                self.last_type = *return_type;

                Ok(())
            },
            t => {
                self.last_type = t;
                Ok(())
            },
        }
    }

    fn type_(&mut self) -> Result<Type, Error> {
        self.ignore_whitespace();
        if let Some(t) = self.next() {
            return match t.value {
                TokenValue::Identifier(s) => match s.as_str() {
                    "string" => Ok(Type::String),
                    "number" => Ok(Type::Number),
                    "boolean" => Ok(Type::Boolean),
                    "nil" => Ok(Type::Nil),
                    "any" => Ok(Type::Any),
                    _ => Err(self.error(ErrorValue::InvalidTypeAnnotation(s))),
                },
                TokenValue::BackSlash => {
                    // Function
                    self.ignore_whitespace();
                    
                    // Collect param types
                    let mut params = Vec::new();
                    while !self.has(TokenValue::Equals) {
                        params.push(self.type_()?);
                    }

                    if !self.take(TokenValue::Equals).is_some() {
                        return Err(self.error(ErrorValue::InvalidTypeAnnotation("Function".to_string())));
                    }

                    // Collect return type
                    let return_type = self.type_()?;

                    Ok(Type::Function(params, Box::new(return_type)))
                },
                TokenValue::LeftParen => {
                    // Just a singular grouping
                    self.ignore_whitespace();
                    let t = self.type_()?;
                    self.ignore_whitespace();
                    if !self.take(TokenValue::RightParen).is_some() {
                        return Err(self.error(ErrorValue::InvalidTypeAnnotation("Group".to_string())));
                    }

                    Ok(t)
                },
                TokenValue::LeftBrace => {
                    // Get identifier
                    let mut members = Vec::new();
                    while !self.has(TokenValue::RightBrace) {
                        self.ignore_whitespace();
                        if let Some(t) = self.next() {
                            if let TokenValue::Identifier(name) = t.value {
                                self.ignore_whitespace();
    
                                // Get params
                                let mut params = Vec::new();
                                while !self.has(TokenValue::Equals) {
                                    params.push(self.type_()?);
                                }
    
                                if self.take(TokenValue::Equals).is_none() {
                                    return Err(self.error(ErrorValue::InvalidTypeAnnotation("Block".to_string())));
                                }
    
                                // Get return type
                                let return_type = self.type_()?;
    
                                if params.len() > 0 {
                                    // Function type
                                    members.push((name, Type::Function(params, Box::new(return_type))));
                                } else {
                                    // Constant
                                    members.push((name, return_type));
                                }
                            } else {
                                return Err(self.error(ErrorValue::InvalidTypeAnnotation("Block".to_string())));
                            }
                        }

                        self.ignore_whitespace();
                    }
        

                    if self.take(TokenValue::RightBrace).is_none() {
                        return Err(self.error(ErrorValue::InvalidTypeAnnotation("Block".to_string())));
                    }


                    Ok(Type::Block(members))
                },
                TokenValue::LeftBracket => {
                    self.ignore_whitespace();
                    let t = self.type_()?;
                    self.ignore_whitespace();
                    if !self.take(TokenValue::RightBracket).is_some() {
                        return Err(self.error(ErrorValue::InvalidTypeAnnotation("Array".to_string())));
                    }

                    Ok(Type::Array(Box::new(t)))
                },
                _ => Err(self.error(ErrorValue::UnexpectedToken(t))),
            }
        }

        Err(self.error(ErrorValue::UnexpectedEOF))
    }

    fn function(&mut self, name: String) -> Result<(), Error> {
        let mut params = Vec::new();
        let mut valid = false;

        while let Some(t) = self.next() {
            match t.value {
                TokenValue::Equals => {
                    valid = true;
                    break;
                },
                TokenValue::Identifier(s) => {
                    if self.take(TokenValue::Colon).is_some() {
                        params.push((s.clone(), self.type_()?));
                    } else {
                        params.push((s.clone(), Type::Unknown));
                    }
                },
                _ => {
                    self.next();
                    break;
                },
            }
        }

        if !valid {
            return Err(self.error(ErrorValue::UnexpectedEOF));
        }

        // Begin a new function
        self.functions.push(Function::new(name.clone(), Chunk::new()));

        self.get_function().begin_scope();

        // Add params to locals
        for (name, type_) in params.iter().rev() {
            // TODO: read the param types
            self.get_function().add_local(name.clone(), type_.clone());
            self.get_function().chunk.write(op::PUSH_LOCAL);
        }

        // Compile the body
        self.expression(true)?;
        let return_type = self.last_type.clone();

        // Get local types of each param
        let mut param_types = Vec::new();
        for (name, _) in params.iter() {
            if let Some(local) = self.get_function().resolve(name).cloned() {
                if local.type_ == Type::Unknown {
                    param_types.push(Type::Any);
                } else {
                    param_types.push(local.type_.clone());
                }
            }
        }

        self.get_function().end_scope();

        let func = self.functions.pop().unwrap();
        let func = Rc::new(func);

        // Add function as constant
        let constant = self.get_function().chunk.add_constant(Value::Object(Object::Function(func.clone())));
        self.get_function().chunk.write_pair(op::PUSH, constant);

        // Handle closures when necessary
        if func.upvalues.len() > 0 {
            // Make the closure
            self.get_function().chunk.write(op::MAKE_CLOSURE);
        }

        // Set return type
        let t = Type::Function(param_types, Box::new(return_type));
        self.last_type = t;

        Ok(())
    }

    fn assignment(&mut self, name: String) -> Result<(), Error> {
        // Determine if it's a function or constant assignment
        if self.take(TokenValue::Equals).is_some() {
            // Compile the body
            self.get_function().begin_scope();
            self.expression(true)?;
            self.get_function().end_scope();

            if self.last_type == Type::Nil {
                // Add an empty string
                let constant = self.get_function().chunk.add_constant(Value::Object(Object::String(String::new())));
                self.get_function().chunk.write_pair(op::PUSH, constant);
            }

            self.get_function().chunk.write(op::PUSH_LOCAL);

            // Add local to closure
            let t = self.last_type.clone();
            self.get_function().add_local(name, t);

            // Emit nil return value from assignment
            self.last_type = Type::Nil;

            Ok(())
        } else {
            // Compile function body
            self.function(name.clone())?;
            self.get_function().chunk.write(op::PUSH_LOCAL);
            let t = self.last_type.clone();
            self.get_function().add_local(name.clone(), t);

            // Emit assignment return value of nil
            self.last_type = Type::Nil;

            Ok(())
        }
    }

    fn group(&mut self) -> Result<(), Error> {   
        self.get_function().begin_scope();

        let mut n = 0;

        self.ignore_whitespace();

        // Loop until right paren
        while !self.has(TokenValue::RightParen) {
            self.ignore_whitespace();
            self.expression(true)?;
            self.ignore_whitespace();

            if self.last_type != Type::Nil {
                n += 1;
            }
        }

        // Make sure it was properly closed
        if self.take(TokenValue::RightParen).is_none() {
            return Err(self.error(ErrorValue::UnexpectedEOF));
        }

        if n == 0 {
            // Since nil values don't actually exist...
            let empty_string = self.get_function().chunk.add_constant(Value::Object(Object::String(String::new())));
            self.get_function().chunk.write_pair(op::PUSH, empty_string);
            self.last_type = Type::Nil;
        } else if n > 1 {
            self.get_function().chunk.write_pair(op::CONCATENATE, n as u16);
            self.last_type = Type::String;
        }

        self.get_function().end_scope();

        Ok(())
    }

    fn list(&mut self) -> Result<(), Error> {
        let mut count = 0;
        let mut item_type = Type::Nil;

        self.get_function().begin_scope();

        self.ignore_whitespace();

        while !self.has(TokenValue::RightBracket) {
            self.ignore_whitespace();
            self.expression(true)?;
            self.ignore_whitespace();

            if self.last_type == Type::Nil {
                continue;     
            }

            count += 1;

            if item_type == Type::Nil {
                item_type = self.last_type.clone();
            } else {
                let t = self.last_type.clone();
                let error = self.error(ErrorValue::ListItemTypeMismatch(item_type.clone(), t));
                self.assert_type_with_error(item_type.clone(), error)?;
            }
        }
        
        if self.take(TokenValue::RightBracket).is_none() {
            return Err(self.error(ErrorValue::UnexpectedEOF));
        }

        self.get_function().chunk.write_pair(op::MAKE_ARRAY, count);

        self.last_type = Type::Array(Box::from(item_type));

        self.get_function().end_scope();

        Ok(())
    }

    fn block(&mut self) -> Result<(), Error> {
        self.get_function().begin_scope();

        self.ignore_whitespace();

        while !self.has(TokenValue::RightBrace) {
            self.ignore_whitespace();
            self.expression(false)?;
            self.ignore_whitespace();
        }

        if self.take(TokenValue::RightBrace).is_none() {
            return Err(self.error(ErrorValue::UnexpectedEOF));
        }

        // Push all locals onto the stack
        let mut types: Vec<(String, Type)> = Vec::new();
        let mut n = 0;
        for local in self.get_function().locals.clone() {
            if local.depth == self.get_function().depth {
                n += 1;
                types.push((local.name, local.type_.clone()));
                self.get_function().chunk.write_pair(op::GET_LOCAL, local.index as u16);
            }
        }

        // Make the block
        self.get_function().chunk.write_pair(op::MAKE_BLOCK, n);


        self.last_type = Type::Block(types);

        self.get_function().end_scope();

        Ok(())
    }

}
