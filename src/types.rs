#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Array(Box<Type>),
    Block(Vec<(String, Type)>),
    Any,
    Unknown,
    Function(Vec<Type>, Box<Type>),
    Nil,
}

impl Type {
    pub fn satisfies(&self, constraint: Type) -> bool {
        if constraint == Type::Any {
            return true;
        }

        match (self, constraint) {
            (Type::Number, Type::Number) => true,
            (Type::String, Type::String) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::Array(t1), Type::Array(t2)) => t1.satisfies(Type::Nil) || t1.satisfies(*t2),
            (Type::Block(t1), Type::Block(t2)) => {
                // Make sure t1 has at least all the keys in t2
                for (k2, v2) in t2.iter() {
                    let mut has = false;
                    for (k1, v1) in t1.iter() {
                        if k1 == k2 {
                            has = v1.satisfies(v2.clone());
                        }
                    }

                    if !has {
                        return false;
                    }
                }

                true
            },
            (Type::Any, _) => true,
            (Type::Function(t1, t2), Type::Function(t3, t4)) => t1.iter().zip(t3.iter()).all(|(a, b)| a.satisfies(b.clone())) && t2.satisfies(*t4),
            (Type::Nil, Type::Nil) => true,
            _ => false,
        }
    }
}