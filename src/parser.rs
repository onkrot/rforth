use crate::types::*;
use std::collections::HashMap;
use std::num::ParseIntError;

#[derive(Eq, PartialEq)]
enum ParserState {
    Normal,
    WordName,
    WordBody,
}

pub fn tokenize(expr: &str) -> Vec<&str> {
    expr.split_whitespace().collect()
}

pub fn parse(tokens: &[&str], env: &mut ForthEnv) -> ForthResult<Vec<ForthExp>> {
    for token in tokens {
        env.parser.tokens.push(token.to_string())
    }
    let mut res: Vec<ForthExp> = vec![];
    let mut new_word: Vec<ForthExp> = vec![];
    let mut name = "".to_string();
    let mut new_words: HashMap<ForthOp, ForthFunc> = HashMap::new();
    let mut state = ParserState::Normal;
    let mut variables: HashMap<String, i64> = HashMap::new();
    while let Ok(token) = env.parser.next() {
        match state {
            ParserState::Normal => match token.as_str() {
                ":" => {
                    if state == ParserState::WordBody {
                        return Err(ForthErr::Msg("Unexpected :".to_string()));
                    }
                    state = ParserState::WordName;
                }
                ";" => {
                    return Err(ForthErr::Msg("Unexpected ;".to_string()));
                }
                "variable" => {
                    let var = env.parser.next()?;
                    variables.insert(var.clone(), 0);
                    new_words.insert(ForthOp::Variable(var.clone()), ForthFunc::Variable);
                }
                "constant" => {
                    let var = env.parser.get_cur();
                    env.parser.next()?;
                    new_words.insert(
                        ForthOp::Constant(var.clone()),
                        ForthFunc::ConstantDef(var.clone()),
                    );
                    res.push(ForthExp::Op(ForthOp::Constant(var.clone())));
                    variables.insert(var, 0);
                }
                t => {
                    let atom = parse_atom(t)?;
                    if let ForthExp::Op(ForthOp::UserWord(var)) = atom {
                        if env.words.contains_key(&ForthOp::Constant(var.clone())) {
                            res.push(ForthExp::Op(ForthOp::Constant(var.clone())));
                        } else if env.words.contains_key(&ForthOp::Variable(var.clone())) {
                            res.push(ForthExp::Op(ForthOp::Variable(var.clone())));
                        } else {
                            res.push(ForthExp::Op(ForthOp::UserWord(var)))
                        };
                    } else {
                        res.push(atom)
                    }
                }
            },
            ParserState::WordName => {
                name = token;
                state = ParserState::WordBody
            }
            ParserState::WordBody => {
                if token.to_ascii_lowercase().as_str() == "variable" {
                    let var = env.parser.next()?;
                    variables.insert(var.clone(), 0);
                    new_words.insert(ForthOp::Variable(var.clone()), ForthFunc::Variable);
                } else if token == ";" {
                    state = ParserState::Normal;
                    new_words.insert(
                        ForthOp::UserWord(name.to_string()),
                        ForthFunc::User(new_word),
                    );
                    new_word = vec![];
                } else {
                    new_word.push(parse_atom(token.as_str())?)
                }
            }
        }
    }
    for new_word in new_words {
        env.words.insert(new_word.0, new_word.1);
    }
    env.variables.extend(variables);
    Ok(res)
}

fn parse_atom(token: &str) -> ForthResult<ForthExp> {
    let potential_int: Result<i64, ParseIntError> = token.parse();
    let res = match potential_int {
        Ok(v) => ForthExp::Number(v),
        Err(_) => {
            let op = match token.to_ascii_lowercase().as_str() {
                "+" => ForthOp::Add,
                "-" => ForthOp::Sub,
                "*" => ForthOp::Mul,
                "/" => ForthOp::Div,
                "1+" => ForthOp::Add1,
                "1-" => ForthOp::Sub1,
                "2+" => ForthOp::Add2,
                "2-" => ForthOp::Sub2,
                "2*" => ForthOp::Mul2,
                "2/" => ForthOp::Div2,
                "mod" => ForthOp::Mod,
                "/mod" => ForthOp::DivMod,
                "*/" => ForthOp::FMD,
                "*/mod" => ForthOp::FMDM,
                "abs" => ForthOp::Abs,
                "negate" => ForthOp::Neg,
                "dup" => ForthOp::Dup,
                "drop" => ForthOp::Drop,
                "over" => ForthOp::Over,
                "rot" => ForthOp::Rot,
                "swap" => ForthOp::Swap,
                "pick" => ForthOp::Pick,
                "roll" => ForthOp::Roll,
                "." => ForthOp::Print,
                "and" => ForthOp::And,
                "or" => ForthOp::Or,
                "xor" => ForthOp::Xor,
                "not" => ForthOp::Not,
                "<" => ForthOp::Lt,
                "=" => ForthOp::Eq,
                ">" => ForthOp::Gt,
                "<=" => ForthOp::Le,
                ">=" => ForthOp::Ge,
                "<>" => ForthOp::Ne,
                "@" => ForthOp::GetVar,
                "!" => ForthOp::SetVar,
                word => ForthOp::UserWord(word.to_string()),
            };
            ForthExp::Op(op)
        }
    };
    Ok(res)
}
