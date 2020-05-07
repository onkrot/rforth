use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

#[derive(Clone)]
pub enum ForthExp {
    Number(i64),
    Op(ForthOp),
}

#[derive(Clone)]
pub enum ForthFunc {
    Native(fn(&mut ForthEnv) -> Result<(), ForthErr>),
    User(Vec<ForthExp>),
    Variable,
    ConstantDef(String),
}

pub enum ForthErr {
    Msg(String),
}

pub type ForthResult<T> = ::std::result::Result<T, ForthErr>;

#[derive(Clone)]
pub struct ForthEnv {
    pub words: HashMap<ForthOp, ForthFunc>,
    pub stack: Vec<ForthExp>,
    pub variables: HashMap<String, i64>,
    pub parser: ForthParser,
}

#[derive(Clone)]
pub struct ForthParser {
    pub tokens: Vec<String>,
    pub cur: usize,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ForthOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    DivMod,
    FMD,
    FMDM,
    Abs,
    Neg,
    Add1,
    Sub1,
    Add2,
    Sub2,
    Mul2,
    Div2,
    Dup,
    Drop,
    Over,
    Rot,
    Swap,
    Pick,
    Roll,
    Print,
    And,
    Or,
    Xor,
    Not,
    Lt,
    Gt,
    Eq,
    Le,
    Ge,
    Ne,
    Variable(String),
    Constant(String),
    GetVar,
    SetVar,
    UserWord(String),
}

impl fmt::Display for ForthExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            ForthExp::Op(s) => format!("{}", s),
            ForthExp::Number(n) => n.to_string(),
        };

        write!(f, "{}", str)
    }
}

impl fmt::Display for ForthOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl ForthEnv {
    pub fn pop_num(&mut self) -> Result<i64, ForthErr> {
        let exp = self
            .stack
            .pop()
            .ok_or(ForthErr::Msg("Empty stack".to_string()))?;
        match exp {
            ForthExp::Number(num) => Ok(num),
            _ => Err(ForthErr::Msg("expected a number".to_string())),
        }
    }
    pub fn push(&mut self, exp: ForthExp) {
        self.stack.push(exp);
    }
    pub fn get_op(&self, op: ForthOp) -> Result<&ForthFunc, ForthErr> {
        self.words
            .get(&op)
            .ok_or(ForthErr::Msg(format!("Not implemented {}", op)))
    }
}

impl ForthParser {
    pub fn next(&mut self) -> ForthResult<String> {
        if self.cur < self.tokens.len() {
            self.cur += 1;
            Ok(self.tokens[self.cur - 1].clone())
        } else {
            Err(ForthErr::Msg("No next token".to_string()))
        }
    }
    pub fn get_cur(&self) -> String {
        self.tokens[self.cur].clone()
    }
    pub fn get_var_name(&self) -> ForthResult<String> {
        if self.cur > 1 {
            Ok(self.tokens[self.cur - 2].clone())
        } else {
            Err(ForthErr::Msg("No var name".to_string()))
        }
    }
}
