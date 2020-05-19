use super::interp::ForthInterp;
use std::fmt;
use std::fmt::Debug;

#[derive(Clone)]
pub enum ForthExp {
    Number(i64),
    Op(ForthOp),
}

#[derive(Clone)]
pub enum ForthFunc {
    Native(fn(&mut ForthInterp) -> Result<(), ForthErr>),
    User(Vec<ForthExp>),
    Variable,
    ConstantDef(String),
    GetVar(String),
    SetVar(String),
    IfThenElse((Vec<ForthExp>, Option<Vec<ForthExp>>)),
    BeginUntil(Vec<ForthExp>),
    BeginWhile(Vec<ForthExp>, Vec<ForthExp>),
}

pub enum ForthErr {
    Msg(String),
}

pub type ForthResult<T> = ::std::result::Result<T, ForthErr>;

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
    Lt0,
    Eq0,
    Gt0,
    Dup2,
    Swap2,
    Drop2,
    Over2,
    Variable(String),
    GetVar(usize),
    SetVar(usize),
    UserWord(String),
    IfThenElse(usize),
    BeginUntil(usize),
    BeginWhile(usize),
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

impl fmt::Display for ForthErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ForthErr::Msg(str) => write!(f, "{}", str),
        }
    }
}
