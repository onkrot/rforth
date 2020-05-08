use super::builtins::add_builtins;
use super::parser::ForthParser;
use super::types::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ForthInterp {
    pub words: HashMap<ForthOp, ForthFunc>,
    pub stack: Vec<ForthExp>,
    pub variables: HashMap<String, i64>,
    pub parser: ForthParser,
}

impl ForthInterp {
    pub fn new() -> ForthInterp {
        let mut interp = ForthInterp {
            words: HashMap::new(),
            stack: vec![],
            variables: HashMap::new(),
            parser: ForthParser::new(),
        };
        add_builtins(&mut interp);
        interp
    }
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
        if let ForthOp::UserWord(var) = op {
            return if self.words.contains_key(&ForthOp::Constant(var.clone())) {
                Ok(self.words.get(&ForthOp::Constant(var)).unwrap())
            } else if self.words.contains_key(&ForthOp::Variable(var.clone())) {
                Ok(self.words.get(&ForthOp::Variable(var)).unwrap())
            } else {
                self.words
                    .get(&ForthOp::UserWord(var.clone()))
                    .ok_or(ForthErr::Msg(format!("Not implemented {}", var)))
            };
        }

        self.words
            .get(&op)
            .ok_or(ForthErr::Msg(format!("Not implemented {}", op)))
    }
    pub fn eval(&mut self, exp: ForthExp) -> ForthResult<()> {
        match exp {
            ForthExp::Op(op) => {
                let func = self.get_op(op)?.clone();
                match func {
                    ForthFunc::Native(f) => f(self)?,
                    ForthFunc::User(v) => {
                        for e in v {
                            self.eval(e)?;
                        }
                    }
                    ForthFunc::Variable => {}
                    ForthFunc::ConstantDef(name) => match self.pop_num() {
                        Ok(num) => {
                            self.words.insert(
                                ForthOp::Constant(name.clone()),
                                ForthFunc::User(vec![ForthExp::Number(num)]),
                            );
                        }
                        Err(_) => return Err(ForthErr::Msg("No constant value".to_string())),
                    },
                    ForthFunc::IfThenElse((Then, Else)) => {
                        let cond = self.pop_num()?;
                        if cond != 0 {
                            for e in Then {
                                self.eval(e)?;
                            }
                        } else if let Some(v) = Else {
                            for e in v {
                                self.eval(e)?;
                            }
                        }
                    }
                }
            }
            ForthExp::Number(a) => self.push(ForthExp::Number(a)),
        }
        Ok(())
    }
    pub fn eval_str(&mut self, expr: &str) -> ForthResult<()> {
        let res = self.parser.parse_str(expr)?;
        self.words.extend(res.new_words);
        self.variables.extend(res.variables);
        for st in res.program {
            self.eval(st)?;
        }

        Ok(())
    }
}
