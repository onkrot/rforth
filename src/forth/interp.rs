use super::parser::ForthParser;
use super::types::*;
use std::cmp::{max, min};
use std::collections::HashMap;

const TRUE: i64 = -1;
const FALSE: i64 = 0;

macro_rules! n_ary_op {
    ($n: expr, $func: expr) => {
        ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
            let mut x: [i64; $n] = [0; $n];
            for i in 0..$n {
                x[i] = interp.pop_num()?;
            }
            interp.push(ForthExp::Number($func(x)));
            return Ok(());
        });
    };
}

macro_rules! restore_stack {
    ($a: expr, $b: expr, $interp: ident) => {
        $interp.push(ForthExp::Number($a));
        $interp.push(ForthExp::Number($b));
    };
    ($a: expr, $b: expr, $c:expr, $interp: ident) => {
        $interp.push(ForthExp::Number($a));
        $interp.push(ForthExp::Number($b));
        $interp.push(ForthExp::Number($c));
    };
}

macro_rules! checked_div {
    ($n: expr, $func: expr) => {
        ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
            let mut x: [i64; $n] = [0; $n];
            for i in 0..$n {
                x[i] = interp.pop_num()?;
            }
            if x[0] == 0 {
                for i in (0..$n).rev() {
                    interp.push(ForthExp::Number(x[i]));
                }
                return Err(ForthErr::Msg("Division by zero".to_string()));
            }
            interp.push(ForthExp::Number($func(x)));
            return Ok(());
        });
    };
}

#[derive(Clone)]
pub struct ForthInterp {
    pub words: HashMap<ForthOp, ForthFunc>,
    pub stack: Vec<ForthExp>,
    pub variables: HashMap<String, i64>,
    pub parser: ForthParser,
}

impl ForthInterp {
    pub fn new() -> ForthInterp {
        ForthInterp {
            words: HashMap::new(),
            stack: vec![],
            variables: HashMap::new(),
            parser: ForthParser::new(),
        }
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
    pub fn eval_op(&self, op: ForthOp) -> ForthResult<ForthFunc> {
        let func = match op {
            ForthOp::Add => n_ary_op!(2, |x: [i64; 2]| x[1].wrapping_add(x[0])),
            ForthOp::Sub => n_ary_op!(2, |x: [i64; 2]| x[1].wrapping_sub(x[0])),
            ForthOp::Mul => n_ary_op!(2, |x: [i64; 2]| x[1].wrapping_mul(x[0])),
            ForthOp::Div => checked_div!(2, |x: [i64; 2]| x[1] / x[0]),
            ForthOp::Mod => checked_div!(2, |x: [i64; 2]| x[1] % x[0]),
            ForthOp::DivMod => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                if b == 0 {
                    restore_stack!(a, b, interp);
                    return Err(ForthErr::Msg("Division by zero".to_string()));
                }
                interp.push(ForthExp::Number(a % b));
                interp.push(ForthExp::Number(a / b));
                return Ok(());
            }),
            ForthOp::FMD => checked_div!(3, |x: [i64; 3]| x[2].wrapping_mul(x[1]) / x[0]),
            ForthOp::FMDM => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let c = interp.pop_num()?;
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                if c == 0 {
                    restore_stack!(a, b, c, interp);
                    return Err(ForthErr::Msg("Division by zero".to_string()));
                }
                interp.push(ForthExp::Number((a.wrapping_mul(b)) % c));
                interp.push(ForthExp::Number((a.wrapping_mul(b)) / c));
                return Ok(());
            }),
            ForthOp::Abs => n_ary_op!(1, |x: [i64; 1]| x[0].abs()),
            ForthOp::Neg => n_ary_op!(1, |x: [i64; 1]| -x[0]),
            ForthOp::Add1 => n_ary_op!(1, |x: [i64; 1]| x[0].wrapping_add(1)),
            ForthOp::Sub1 => n_ary_op!(1, |x: [i64; 1]| x[0].wrapping_sub(1)),
            ForthOp::Add2 => n_ary_op!(1, |x: [i64; 1]| x[0].wrapping_add(2)),
            ForthOp::Sub2 => n_ary_op!(1, |x: [i64; 1]| x[0].wrapping_sub(2)),
            ForthOp::Mul2 => n_ary_op!(1, |x: [i64; 1]| x[0].wrapping_mul(2)),
            ForthOp::Div2 => n_ary_op!(1, |x: [i64; 1]| x[0] / 2),
            ForthOp::Dup => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(a));
                return Ok(());
            }),
            ForthOp::IfDup => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(a));
                if a != 0 {
                    interp.push(ForthExp::Number(a));
                }
                return Ok(());
            }),
            ForthOp::Drop => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                interp.pop_num()?;
                return Ok(());
            }),
            ForthOp::Over => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(b));
                interp.push(ForthExp::Number(a));
                return Ok(());
            }),
            ForthOp::Rot => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let c = interp.pop_num()?;
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(b));
                interp.push(ForthExp::Number(c));
                interp.push(ForthExp::Number(a));
                return Ok(());
            }),
            ForthOp::Swap => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(b));
                interp.push(ForthExp::Number(a));
                return Ok(());
            }),
            ForthOp::Dup2 => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(b));
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(b));
                return Ok(());
            }),
            ForthOp::Drop2 => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                interp.pop_num()?;
                interp.pop_num()?;
                return Ok(());
            }),
            ForthOp::Over2 => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let d = interp.pop_num()?;
                let c = interp.pop_num()?;
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(b));
                interp.push(ForthExp::Number(c));
                interp.push(ForthExp::Number(d));
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(b));
                return Ok(());
            }),
            ForthOp::Swap2 => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let d = interp.pop_num()?;
                let c = interp.pop_num()?;
                let b = interp.pop_num()?;
                let a = interp.pop_num()?;
                interp.push(ForthExp::Number(c));
                interp.push(ForthExp::Number(d));
                interp.push(ForthExp::Number(a));
                interp.push(ForthExp::Number(b));
                return Ok(());
            }),
            ForthOp::Pick => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let n = interp.pop_num()?;
                if n < interp.stack.len() as i64 {
                    let t: usize = interp.stack.len() - (n + 1) as usize;
                    interp.push(interp.stack[t].clone());
                } else {
                    interp.push(ForthExp::Number(n));
                    return Err(ForthErr::Msg("Not enough values".to_string()));
                }
                Ok(())
            }),
            ForthOp::Roll => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let n = interp.pop_num()?;
                if n < interp.stack.len() as i64 {
                    let t: usize = interp.stack.len() - (n + 1) as usize;
                    let val = interp.stack.remove(t);
                    interp.push(val);
                } else {
                    interp.push(ForthExp::Number(n));
                    return Err(ForthErr::Msg("Not enough values".to_string()));
                }
                Ok(())
            }),
            ForthOp::Print => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                let a = interp.pop_num()?;
                println!("{} ", a);
                return Ok(());
            }),
            ForthOp::Depth => ForthFunc::Native(|interp: &mut ForthInterp| -> ForthResult<()> {
                interp.push(ForthExp::Number(interp.stack.len() as i64));
                return Ok(());
            }),
            ForthOp::And => n_ary_op!(2, |x: [i64; 2]| x[0] & x[1]),
            ForthOp::Or => n_ary_op!(2, |x: [i64; 2]| x[0] | x[1]),
            ForthOp::Xor => n_ary_op!(2, |x: [i64; 2]| x[0] ^ x[1]),
            ForthOp::Not => n_ary_op!(1, |x: [i64; 1]| !x[0]),
            ForthOp::Invert => n_ary_op!(1, |x: [i64; 1]| if x[0] != FALSE { FALSE } else { TRUE }),
            ForthOp::Lt => n_ary_op!(2, |x: [i64; 2]| if x[1] < x[0] { TRUE } else { FALSE }),
            ForthOp::Gt => n_ary_op!(2, |x: [i64; 2]| if x[1] > x[0] { TRUE } else { FALSE }),
            ForthOp::Eq => n_ary_op!(2, |x: [i64; 2]| if x[1] == x[0] { TRUE } else { FALSE }),
            ForthOp::Le => n_ary_op!(2, |x: [i64; 2]| if x[1] <= x[0] { TRUE } else { FALSE }),
            ForthOp::Ge => n_ary_op!(2, |x: [i64; 2]| if x[1] >= x[0] { TRUE } else { FALSE }),
            ForthOp::Ne => n_ary_op!(2, |x: [i64; 2]| if x[1] != x[0] { TRUE } else { FALSE }),
            ForthOp::Lt0 => n_ary_op!(1, |x: [i64; 1]| if x[0] < 0 { TRUE } else { FALSE }),
            ForthOp::Eq0 => n_ary_op!(1, |x: [i64; 1]| if x[0] == 0 { TRUE } else { FALSE }),
            ForthOp::Gt0 => n_ary_op!(1, |x: [i64; 1]| if x[0] > 0 { TRUE } else { FALSE }),
            ForthOp::Ne0 => n_ary_op!(1, |x: [i64; 1]| if x[0] != 0 { TRUE } else { FALSE }),
            ForthOp::True => n_ary_op!(0, |_: [i64; 0]| TRUE),
            ForthOp::False => n_ary_op!(0, |_: [i64; 0]| FALSE),
            ForthOp::Lshift => n_ary_op!(2, |x: [i64; 2]| x[1] << x[0]),
            ForthOp::Rshift => n_ary_op!(2, |x: [i64; 2]| x[1] >> x[0]),
            ForthOp::Max => n_ary_op!(2, |x: [i64; 2]| max(x[0], x[1])),
            ForthOp::Min => n_ary_op!(2, |x: [i64; 2]| min(x[0], x[1])),
            ForthOp::Variable(_) => ForthFunc::Variable,
            ForthOp::GetVar(num) => self
                .words
                .get(&ForthOp::GetVar(num))
                .ok_or(ForthErr::Msg(format!("Not defined variable at {}", num)))?
                .clone(),
            ForthOp::SetVar(num) => self
                .words
                .get(&ForthOp::SetVar(num))
                .ok_or(ForthErr::Msg(format!("Not defined variable at {}", num)))?
                .clone(),
            ForthOp::UserWord(name) => {
                if self.variables.contains_key(&name) {
                    ForthFunc::Variable
                } else {
                    self.words
                        .get(&ForthOp::UserWord(name.clone()))
                        .ok_or(ForthErr::Msg(format!("Not implemented {}", name)))?
                        .clone()
                }
            }
            ForthOp::IfThenElse(num) => self
                .words
                .get(&ForthOp::IfThenElse(num))
                .ok_or(ForthErr::Msg(format!("No body for if at {}", num)))?
                .clone(),
            ForthOp::BeginUntil(num) => self
                .words
                .get(&ForthOp::BeginUntil(num))
                .ok_or(ForthErr::Msg(format!("No body for cycle at {}", num)))?
                .clone(),
            ForthOp::BeginWhile(num) => self
                .words
                .get(&ForthOp::BeginWhile(num))
                .ok_or(ForthErr::Msg(format!("No body for cycle at {}", num)))?
                .clone(),
        };

        Ok(func)
    }
    pub fn eval(&mut self, exp: ForthExp) -> ForthResult<()> {
        match exp {
            ForthExp::Op(op) => {
                let func = self.eval_op(op)?.clone();
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
                                ForthOp::UserWord(name.clone()),
                                ForthFunc::User(vec![ForthExp::Number(num)]),
                            );
                        }
                        Err(_) => return Err(ForthErr::Msg("No constant value".to_string())),
                    },
                    ForthFunc::IfThenElse((then, r#else)) => {
                        let cond = self.pop_num()?;
                        if cond != 0 {
                            for e in then {
                                self.eval(e)?;
                            }
                        } else if let Some(v) = r#else {
                            for e in v {
                                self.eval(e)?;
                            }
                        }
                    }
                    ForthFunc::GetVar(name) => {
                        let num = self
                            .variables
                            .get(&name)
                            .ok_or(ForthErr::Msg(format!("Not defined variable at {}", name)))?
                            .clone();
                        self.push(ForthExp::Number(num));
                    }
                    ForthFunc::SetVar(name) => {
                        let num = self.pop_num()?;
                        self.variables.insert(name, num);
                    }
                    ForthFunc::BeginUntil(v) => {
                        let mut a = 0;
                        while a == 0 {
                            for e in &v {
                                self.eval(e.clone())?;
                            }
                            a = self.pop_num()?;
                        }
                    }
                    ForthFunc::BeginWhile(head, body) => loop {
                        for e in &head {
                            self.eval(e.clone())?;
                        }
                        let a = self.pop_num()?;
                        if a == 0 {
                            break;
                        }
                        for e in &body {
                            self.eval(e.clone())?;
                        }
                    },
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
