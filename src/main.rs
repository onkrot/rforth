use crate::builtins::*;
use crate::parser::*;
use crate::types::*;
use std::io;

mod builtins;
mod parser;
mod types;

fn eval_str(str: Vec<ForthExp>, env: &mut ForthEnv) -> Result<(), ForthErr> {
    for st in str {
        eval(st, env)?;
    }

    Ok(())
}

fn eval(exp: ForthExp, env: &mut ForthEnv) -> ForthResult<()> {
    match exp {
        ForthExp::Op(op) => {
            let func = env.get_op(op)?.clone();
            match func {
                ForthFunc::Native(f) => f(env)?,
                ForthFunc::User(v) => {
                    for e in v {
                        eval(e, env)?;
                    }
                }
                ForthFunc::Variable => {}
                ForthFunc::ConstantDef(name) => match env.pop_num() {
                    Ok(num) => {
                        env.words.insert(
                            ForthOp::Constant(name.clone()),
                            ForthFunc::User(vec![ForthExp::Number(num)]),
                        );
                    }
                    Err(_) => return Err(ForthErr::Msg("No constant value".to_string())),
                },
            }
        }
        ForthExp::Number(a) => env.push(ForthExp::Number(a)),
    }
    Ok(())
}

fn parse_eval(expr: &str, env: &mut ForthEnv) -> ForthResult<()> {
    let parsed_exp = parse(&tokenize(expr), env)?;
    eval_str(parsed_exp, env)?;

    Ok(())
}

fn slurp_expr() -> String {
    let mut expr = String::new();

    io::stdin()
        .read_line(&mut expr)
        .expect("Failed to read line");

    expr
}

fn main() {
    let mut env = default_env();
    loop {
        println!("rforth >");
        let expr = slurp_expr();
        match parse_eval(&expr, &mut env) {
            Ok(_) => {
                print!("// stack => ");
                for exp in &env.stack {
                    print!("{} ", exp)
                }
                println!();
            }

            Err(e) => match e {
                ForthErr::Msg(msg) => println!("// err => {}", msg),
            },
        }
    }
}
