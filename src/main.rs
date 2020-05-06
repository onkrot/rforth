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

fn eval(exp: ForthExp, env: &mut ForthEnv) -> ForthResult<()>  {
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
            }
        }
        ForthExp::Number(a) => env.push(ForthExp::Number(a)),
    }
    Ok(())
}

fn parse_eval(expr: &str, env: &mut ForthEnv) -> ForthResult<()>  {
    let (parsed_exp, new_words) = parse(&tokenize(expr))?;
    for new_word in new_words {
        env.words
            .insert(ForthOp::UserWord(new_word.0), ForthFunc::User(new_word.1));
    }
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
    let env = &mut default_env();
    loop {
        println!("rforth >");
        let expr = slurp_expr();
        match parse_eval(&expr, env) {
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
