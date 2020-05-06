use crate::types::*;
use std::collections::HashMap;

macro_rules! restore_stack {
    ($a: expr, $b: expr, $env: ident) => {
        $env.push(ForthExp::Number($a));
        $env.push(ForthExp::Number($b));
    };
    ($a: expr, $b: expr, $c:expr, $env: ident) => {
        $env.push(ForthExp::Number($a));
        $env.push(ForthExp::Number($b));
        $env.push(ForthExp::Number($c));
    };
}

macro_rules! generic_op {
    ($map: ident, $op: expr, $func: expr) => {
        $map.insert($op, ForthFunc::Native($func));
    };
}

macro_rules! n_ary_op {
    ($map: ident, $op: expr, $n: expr, $func: expr) => {
        generic_op!($map, $op, |env: &mut ForthEnv| -> ForthResult<()> {
            let mut x: [i64; $n] = [0; $n];
            for i in 0..$n {
                x[i] = env.pop_num()?;
            }
            env.push(ForthExp::Number($func(x)));
            return Ok(());
        });
    };
}

macro_rules! checked_div {
    ($map: ident, $op: expr, $n: expr, $func: expr) => {
        generic_op!($map, $op, |env: &mut ForthEnv| -> ForthResult<()> {
            let mut x: [i64; $n] = [0; $n];
            for i in 0..$n {
                x[i] = env.pop_num()?;
            }
            if x[0] == 0 {
                for i in (0..$n).rev() {
                    env.push(ForthExp::Number(x[i]));
                }
                return Err(ForthErr::Msg("Division by zero".to_string()));
            }
            env.push(ForthExp::Number($func(x)));
            return Ok(());
        });
    };
}

pub fn default_env() -> ForthEnv {
    let mut words: HashMap<ForthOp, ForthFunc> = HashMap::new();

    n_ary_op!(words, ForthOp::Add, 2, |x: [i64; 2]| x[1].wrapping_add(x[0]));
    n_ary_op!(words, ForthOp::Sub, 2, |x: [i64; 2]| x[1].wrapping_sub(x[0]));
    n_ary_op!(words, ForthOp::Mul, 2, |x: [i64; 2]| x[1].wrapping_mul(x[0]));
    checked_div!(words, ForthOp::Div, 2, |x: [i64; 2]| x[1] / x[0]);
    checked_div!(words, ForthOp::Mod, 2, |x: [i64; 2]| x[1] % x[0]);
    generic_op!(
        words,
        ForthOp::DivMod,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let b = env.pop_num()?;
            let a = env.pop_num()?;
            if b == 0 {
                restore_stack!(a, b, env);
                return Err(ForthErr::Msg("Division by zero".to_string()));
            }
            env.push(ForthExp::Number(a % b));
            env.push(ForthExp::Number(a / b));
            return Ok(());
        }
    );
    checked_div!(words, ForthOp::FMD, 3, |x: [i64; 3]| x[2].wrapping_mul(x[1])
        / x[0]);
    generic_op!(
        words,
        ForthOp::FMDM,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let c = env.pop_num()?;
            let b = env.pop_num()?;
            let a = env.pop_num()?;
            if c == 0 {
                restore_stack!(a, b, c, env);
                return Err(ForthErr::Msg("Division by zero".to_string()));
            }
            env.push(ForthExp::Number((a.wrapping_mul(b)) % c));
            env.push(ForthExp::Number((a.wrapping_mul(b)) / c));
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
    n_ary_op!(words, ForthOp::And, 2, |x: [i64; 2]| if x[0] != 0 && x[1] != 0 { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Or, 2, |x: [i64; 2]| if x[0] != 0 || x[1] != 0 { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Xor, 2, |x: [i64; 2]| if (x[0] != 0) != (x[1] != 0) { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Not, 1, |x: [i64; 1]| if x[0] != 0 { 0 } else { 1 });
    n_ary_op!(words, ForthOp::Lt, 2, |x: [i64; 2]| if x[1] < x[0] { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Eq, 2, |x: [i64; 2]| if x[1] == x[0] { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Gt, 2, |x: [i64; 2]| if x[1] > x[0] { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Le, 2, |x: [i64; 2]| if x[1] <= x[0] { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Ge, 2, |x: [i64; 2]| if x[1] >= x[0] { 1 } else { 0 });
    n_ary_op!(words, ForthOp::Ne, 2, |x: [i64; 2]| if x[1] != x[0] { 1 } else { 0 });
    generic_op!(
        words,
        ForthOp::Dup,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let a = env.pop_num()?;
            env.push(ForthExp::Number(a));
            env.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Drop,
        |env: &mut ForthEnv| -> ForthResult<()> {
            env.pop_num()?;
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Print,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let a = env.pop_num()?;
            println!("{} ", a);
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Over,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let b = env.pop_num()?;
            let a = env.pop_num()?;
            env.push(ForthExp::Number(a));
            env.push(ForthExp::Number(b));
            env.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Rot,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let c = env.pop_num()?;
            let b = env.pop_num()?;
            let a = env.pop_num()?;
            env.push(ForthExp::Number(b));
            env.push(ForthExp::Number(c));
            env.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Swap,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let b = env.pop_num()?;
            let a = env.pop_num()?;
            env.push(ForthExp::Number(b));
            env.push(ForthExp::Number(a));
            return Ok(());
        }
    );

    generic_op!(
        words,
        ForthOp::Pick,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let n = env.pop_num()?;
            if n < env.stack.len() as i64 {
                let t: usize = env.stack.len() - (n + 1) as usize;
                env.push(env.stack[t].clone());
            } else {
                env.push(ForthExp::Number(n));
                return Err(ForthErr::Msg("Not enough values".to_string()));
            }
            Ok(())
        }
    );

    generic_op!(
        words,
        ForthOp::Roll,
        |env: &mut ForthEnv| -> ForthResult<()> {
            let n = env.pop_num()?;
            if n < env.stack.len() as i64 {
                let t: usize = env.stack.len() - (n + 1) as usize;
                let val = env.stack.remove(t);
                env.push(val);
            } else {
                env.push(ForthExp::Number(n));
                return Err(ForthErr::Msg("Not enough values".to_string()));
            }
            Ok(())
        }
    );

    ForthEnv {
        words: words,
        stack: vec![],
    }
}
