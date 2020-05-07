use super::types::*;
use super::ForthInterp;
use std::collections::HashMap;

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

macro_rules! generic_op {
    ($map: ident, $op: expr, $func: expr) => {
        $map.insert($op, ForthFunc::Native($func));
    };
}

macro_rules! n_ary_op {
    ($map: ident, $op: expr, $n: expr, $func: expr) => {
        generic_op!($map, $op, |interp: &mut ForthInterp| -> ForthResult<()> {
            let mut x: [i64; $n] = [0; $n];
            for i in 0..$n {
                x[i] = interp.pop_num()?;
            }
            interp.push(ForthExp::Number($func(x)));
            return Ok(());
        });
    };
}

macro_rules! checked_div {
    ($map: ident, $op: expr, $n: expr, $func: expr) => {
        generic_op!($map, $op, |interp: &mut ForthInterp| -> ForthResult<()> {
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

pub fn add_builtins(interp: &mut ForthInterp) {
    let mut words: HashMap<ForthOp, ForthFunc> = HashMap::new();

    n_ary_op!(words, ForthOp::Add, 2, |x: [i64; 2]| x[1]
        .wrapping_add(x[0]));
    n_ary_op!(words, ForthOp::Sub, 2, |x: [i64; 2]| x[1]
        .wrapping_sub(x[0]));
    n_ary_op!(words, ForthOp::Mul, 2, |x: [i64; 2]| x[1]
        .wrapping_mul(x[0]));
    checked_div!(words, ForthOp::Div, 2, |x: [i64; 2]| x[1] / x[0]);
    checked_div!(words, ForthOp::Mod, 2, |x: [i64; 2]| x[1] % x[0]);
    generic_op!(
        words,
        ForthOp::DivMod,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let b = interp.pop_num()?;
            let a = interp.pop_num()?;
            if b == 0 {
                restore_stack!(a, b, interp);
                return Err(ForthErr::Msg("Division by zero".to_string()));
            }
            interp.push(ForthExp::Number(a % b));
            interp.push(ForthExp::Number(a / b));
            return Ok(());
        }
    );
    checked_div!(words, ForthOp::FMD, 3, |x: [i64; 3]| x[2]
        .wrapping_mul(x[1])
        / x[0]);
    generic_op!(
        words,
        ForthOp::FMDM,
        |interp: &mut ForthInterp| -> ForthResult<()> {
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
        }
    );
    n_ary_op!(words, ForthOp::Abs, 1, |x: [i64; 1]| x[0].abs());
    n_ary_op!(words, ForthOp::Neg, 1, |x: [i64; 1]| -x[0]);
    n_ary_op!(words, ForthOp::Add1, 1, |x: [i64; 1]| x[0].wrapping_add(1));
    n_ary_op!(words, ForthOp::Sub1, 1, |x: [i64; 1]| x[0].wrapping_sub(1));
    n_ary_op!(words, ForthOp::Add2, 1, |x: [i64; 1]| x[0].wrapping_add(2));
    n_ary_op!(words, ForthOp::Sub2, 1, |x: [i64; 1]| x[0].wrapping_sub(2));
    n_ary_op!(words, ForthOp::Mul2, 1, |x: [i64; 1]| x[0].wrapping_mul(2));
    n_ary_op!(words, ForthOp::Div2, 1, |x: [i64; 1]| x[0] / 2);
    n_ary_op!(
        words,
        ForthOp::And,
        2,
        |x: [i64; 2]| if x[0] != 0 && x[1] != 0 { 1 } else { 0 }
    );
    n_ary_op!(
        words,
        ForthOp::Or,
        2,
        |x: [i64; 2]| if x[0] != 0 || x[1] != 0 { 1 } else { 0 }
    );
    n_ary_op!(
        words,
        ForthOp::Xor,
        2,
        |x: [i64; 2]| if (x[0] != 0) != (x[1] != 0) { 1 } else { 0 }
    );
    n_ary_op!(words, ForthOp::Not, 1, |x: [i64; 1]| if x[0] != 0 {
        0
    } else {
        1
    });
    n_ary_op!(words, ForthOp::Lt, 2, |x: [i64; 2]| if x[1] < x[0] {
        1
    } else {
        0
    });
    n_ary_op!(words, ForthOp::Eq, 2, |x: [i64; 2]| if x[1] == x[0] {
        1
    } else {
        0
    });
    n_ary_op!(words, ForthOp::Gt, 2, |x: [i64; 2]| if x[1] > x[0] {
        1
    } else {
        0
    });
    n_ary_op!(words, ForthOp::Le, 2, |x: [i64; 2]| if x[1] <= x[0] {
        1
    } else {
        0
    });
    n_ary_op!(words, ForthOp::Ge, 2, |x: [i64; 2]| if x[1] >= x[0] {
        1
    } else {
        0
    });
    n_ary_op!(words, ForthOp::Ne, 2, |x: [i64; 2]| if x[1] != x[0] {
        1
    } else {
        0
    });
    generic_op!(
        words,
        ForthOp::GetVar,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let name = interp.parser.get_var_name()?;
            let a = interp
                .variables
                .get(&name)
                .ok_or(ForthErr::Msg(format!("Undefined variable")))?
                .clone();
            interp.push(ForthExp::Number(a));
            return Ok(());
        }
    );
    generic_op!(
        words,
        ForthOp::SetVar,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let name = interp.parser.get_var_name()?;
            if interp.words.contains_key(&ForthOp::Constant(name.clone())) {
                return Err(ForthErr::Msg("Cannot reset constant".to_string()));
            } else if interp.words.contains_key(&ForthOp::Variable(name.clone())) {
                let a = interp.pop_num()?;
                interp.variables.insert(name, a);
                return Ok(());
            } else {
                return Err(ForthErr::Msg("Undefined variable".to_string()));
            };
        }
    );
    generic_op!(
        words,
        ForthOp::Dup,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let a = interp.pop_num()?;
            interp.push(ForthExp::Number(a));
            interp.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Drop,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            interp.pop_num()?;
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Print,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let a = interp.pop_num()?;
            println!("{} ", a);
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Over,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let b = interp.pop_num()?;
            let a = interp.pop_num()?;
            interp.push(ForthExp::Number(a));
            interp.push(ForthExp::Number(b));
            interp.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Rot,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let c = interp.pop_num()?;
            let b = interp.pop_num()?;
            let a = interp.pop_num()?;
            interp.push(ForthExp::Number(b));
            interp.push(ForthExp::Number(c));
            interp.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Swap,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let b = interp.pop_num()?;
            let a = interp.pop_num()?;
            interp.push(ForthExp::Number(b));
            interp.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Pick,
        |interp: &mut ForthInterp| -> ForthResult<()> {
            let n = interp.pop_num()?;
            if n < interp.stack.len() as i64 {
                let t: usize = interp.stack.len() - (n + 1) as usize;
                interp.push(interp.stack[t].clone());
            } else {
                interp.push(ForthExp::Number(n));
                return Err(ForthErr::Msg("Not enough values".to_string()));
            }
            Ok(())
        }
    );

    generic_op!(
        words,
        ForthOp::Roll,
        |interp: &mut ForthInterp| -> ForthResult<()> {
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
        }
    );
    interp.words.extend(words);
}
