use crate::types::*;
use std::num::ParseIntError;

#[derive(Eq, PartialEq)]
enum ParserState{
    Normal,
    WordName,
    WordBody,
}

pub fn tokenize(expr: &str) -> Vec<&str> {
    expr.split_whitespace().collect()
}

pub fn parse(tokens: &[&str]) -> ForthResult<(Vec<ForthExp>, Vec<(String, Vec<ForthExp>)>)> {
    let mut res: Vec<ForthExp> = vec![];
    let mut new_word: Vec<ForthExp> = vec![];
    let mut name = "";
    let mut new_words: Vec<(String, Vec<ForthExp>)> = vec![];
    let mut state = ParserState::Normal;
    for token in tokens {
        match state {
            ParserState::Normal => {
                match *token {
                    ":" => {
                        if state == ParserState::WordBody {
                            return Err(ForthErr::Msg("Unexpected :".to_string()));
                        }
                        state = ParserState::WordName;
                    },
                    ";" => {
                        if state == ParserState::Normal {
                            return Err(ForthErr::Msg("Unexpected ;".to_string()));
                        }
                        state = ParserState::Normal;
                        new_words.push((name.to_string(), new_word));
                        new_word = vec![];
                    }
                    t => res.push(parse_atom(t)?)
                }
            }
            ParserState::WordName => name = token,
            ParserState::WordBody => new_word.push(parse_atom(token)?),
        }
    }
    Ok((res, new_words))
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
                word => ForthOp::UserWord(word.to_string()),
            };
            ForthExp::Op(op)
        }
    };
    Ok(res)
}
