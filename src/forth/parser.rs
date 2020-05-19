use super::types::*;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::slice::Iter;

#[derive(Eq, PartialEq, Clone)]
enum ParserState {
    Normal,
    WordName,
    WordBody,
}

#[derive(Clone)]
pub struct ForthParser {
    tokens: Vec<String>,
    cur: usize,
    state: ParserState,
    word_name: String,
    new_word: Vec<ForthExp>,
}

pub struct ParserResult {
    pub program: Vec<ForthExp>,
    pub new_words: HashMap<ForthOp, ForthFunc>,
    pub variables: HashMap<String, i64>,
}

impl ForthParser {
    pub fn new() -> ForthParser {
        ForthParser {
            tokens: vec![],
            cur: 0,
            state: ParserState::Normal,
            word_name: String::new(),
            new_word: vec![],
        }
    }
    pub fn parse_str(&mut self, expr: &str) -> ForthResult<ParserResult> {
        let lower_expr = expr.to_ascii_lowercase();
        let parsed_exp = self.parse(&tokenize(&lower_expr))?;

        Ok(parsed_exp)
    }
    fn parse(&mut self, tokens: &[&str]) -> ForthResult<ParserResult> {
        let mut res = ParserResult {
            program: vec![],
            new_words: HashMap::new(),
            variables: HashMap::new(),
        };
        let mut normal_tokens = vec![];
        let mut word_tokens = vec![];
        let mut iter = tokens.iter();
        while let Some(token) = iter.next() {
            match self.state {
                ParserState::Normal => match *token {
                    ":" => {
                        let mut expr = self.parse_simple(normal_tokens, &mut res)?;
                        normal_tokens = vec![];
                        res.program.append(expr.as_mut());
                        self.state = ParserState::WordName;
                    }
                    ";" => {
                        return Err(ForthErr::Msg("Unexpected ;".to_string()));
                    }
                    t => {
                        normal_tokens.push(t.to_string());
                    }
                },
                ParserState::WordName => {
                    self.word_name = token.to_string();
                    self.state = ParserState::WordBody
                }
                ParserState::WordBody => {
                    if *token == ";" {
                        self.state = ParserState::Normal;
                        let expr = self.parse_simple(word_tokens, &mut res)?;
                        word_tokens = vec![];
                        res.new_words.insert(
                            ForthOp::UserWord(self.word_name.clone()),
                            ForthFunc::User(expr),
                        );
                        self.new_word.clear();
                    } else if *token == ":" {
                        return Err(ForthErr::Msg("Unexpected :".to_string()));
                    } else {
                        word_tokens.push(token.to_string());
                    }
                }
            }
        }
        let mut expr = self.parse_simple(normal_tokens, &mut res)?;
        res.program.append(expr.as_mut());
        Ok(res)
    }

    fn parse_simple(
        &mut self,
        tokens: Vec<String>,
        res: &mut ParserResult,
    ) -> ForthResult<Vec<ForthExp>> {
        let mut parsed_tokens = vec![];
        let mut prev_token = "";
        let mut iter = tokens.iter();
        while let Some(token) = iter.next() {
            let parsed_token = match token.as_str() {
                "variable" => {
                    let var = iter.next().ok_or(ForthErr::Msg("no name".to_string()))?;
                    res.variables.insert(var.clone(), 0);
                    res.new_words
                        .insert(ForthOp::Variable(var.clone()), ForthFunc::Variable);
                    Ok(ForthExp::Op(ForthOp::Variable(var.clone())))
                }
                "constant" => {
                    let var = iter
                        .next()
                        .ok_or(ForthErr::Msg("no name".to_string()))?
                        .clone();
                    res.new_words.insert(
                        ForthOp::UserWord(var.clone()),
                        ForthFunc::ConstantDef(var.clone()),
                    );
                    Ok(ForthExp::Op(ForthOp::UserWord(var.clone())))
                }
                "@" => {
                    let var = prev_token;
                    res.new_words.insert(
                        ForthOp::GetVar(self.cur),
                        ForthFunc::GetVar(var.to_string()),
                    );
                    Ok(ForthExp::Op(ForthOp::GetVar(self.cur)))
                }
                "!" => {
                    let var = prev_token;
                    res.new_words.insert(
                        ForthOp::SetVar(self.cur),
                        ForthFunc::SetVar(var.to_string()),
                    );
                    Ok(ForthExp::Op(ForthOp::SetVar(self.cur)))
                }
                "if" => {
                    let expr = self.parse_if(&mut iter, res)?;
                    res.new_words
                        .insert(ForthOp::IfThenElse(self.cur), ForthFunc::IfThenElse(expr));
                    Ok(ForthExp::Op(ForthOp::IfThenElse(self.cur)))
                }
                "begin" => {
                    let (body1, body2) = self.parse_cycle(&mut iter, res)?;
                    match body2 {
                        None => {
                            res.new_words.insert(
                                ForthOp::BeginUntil(self.cur),
                                ForthFunc::BeginUntil(body1),
                            );
                            Ok(ForthExp::Op(ForthOp::BeginUntil(self.cur)))
                        }
                        Some(body) => {
                            res.new_words.insert(
                                ForthOp::BeginWhile(self.cur),
                                ForthFunc::BeginWhile(body1, body),
                            );
                            Ok(ForthExp::Op(ForthOp::BeginWhile(self.cur)))
                        }
                    }
                }
                t => parse_word(t),
            };
            parsed_tokens.push(parsed_token?);
            prev_token = token;
            self.cur += 1;
        }
        Ok(parsed_tokens)
    }

    fn parse_if(
        &mut self,
        tokens: &mut Iter<String>,
        res: &mut ParserResult,
    ) -> ForthResult<(Vec<ForthExp>, Option<Vec<ForthExp>>)> {
        let mut then: Vec<String> = vec![];
        let mut r#else: Vec<String> = vec![];
        let mut else_found = false;
        while let Some(token) = tokens.next() {
            match token.as_str() {
                "then" => break,
                "else" => else_found = true,
                t => {
                    if else_found {
                        r#else.push(t.to_string())
                    } else {
                        then.push(t.to_string());
                    }
                }
            }
        }
        let then_parsed = self.parse_simple(then, res)?;
        if else_found {
            let else_parsed = self.parse_simple(r#else, res)?;
            Ok((then_parsed, Some(else_parsed)))
        } else {
            Ok((then_parsed, None))
        }
    }

    fn parse_cycle(
        &mut self,
        tokens: &mut Iter<String>,
        res: &mut ParserResult,
    ) -> ForthResult<(Vec<ForthExp>, Option<Vec<ForthExp>>)> {
        let mut body1 = vec![];
        let mut body2 = vec![];
        let mut while_found = false;
        while let Some(token) = tokens.next() {
            match token.as_str() {
                "until" => {
                    if while_found {
                        return Err(ForthErr::Msg("unexpected until".to_string()));
                    } else {
                        break;
                    }
                }
                "repeat" => {
                    if !while_found {
                        return Err(ForthErr::Msg("unexpected repeat".to_string()));
                    } else {
                        break;
                    }
                }
                "while" => while_found = true,
                _ => {
                    if while_found {
                        body2.push(token.to_string());
                    } else {
                        body1.push(token.to_string());
                    }
                }
            }
        }
        let body1_parsed = self.parse_simple(body1, res)?;
        if while_found {
            let body2_parsed = self.parse_simple(body2, res)?;
            Ok((body1_parsed, Some(body2_parsed)))
        } else {
            Ok((body1_parsed, None))
        }
    }
}

fn tokenize(expr: &str) -> Vec<&str> {
    expr.split_whitespace().collect()
}

fn parse_word(token: &str) -> ForthResult<ForthExp> {
    let potential_int: Result<i64, ParseIntError> = token.parse();
    let res = match potential_int {
        Ok(v) => ForthExp::Number(v),
        Err(_) => {
            let op = match token {
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
                "0<" => ForthOp::Lt0,
                "0=" => ForthOp::Eq0,
                "0>" => ForthOp::Gt0,
                word => ForthOp::UserWord(word.to_string()),
            };
            ForthExp::Op(op)
        }
    };
    Ok(res)
}
