use super::types::*;
use std::collections::HashMap;
use std::num::ParseIntError;

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
    fn next(&mut self) -> ForthResult<String> {
        if self.cur < self.tokens.len() {
            self.cur += 1;
            Ok(self.tokens[self.cur - 1].clone())
        } else {
            Err(ForthErr::Msg("No next token".to_string()))
        }
    }
    fn get_cur(&self) -> String {
        self.tokens[self.cur].clone()
    }
    pub fn get_var_name(&self) -> ForthResult<String> {
        if self.cur > 1 {
            Ok(self.tokens[self.cur - 2].clone())
        } else {
            Err(ForthErr::Msg("No var name".to_string()))
        }
    }
    pub fn parse_str(&mut self, expr: &str) -> ForthResult<ParserResult> {
        let parsed_exp = self.parse(&tokenize(expr))?;

        Ok(parsed_exp)
    }
    fn parse(&mut self, tokens: &[&str]) -> ForthResult<ParserResult> {
        for token in tokens {
            self.tokens.push(token.to_string())
        }
        let mut res = ParserResult {
            program: vec![],
            new_words: HashMap::new(),
            variables: HashMap::new(),
        };
        while let Ok(token) = self.next() {
            match self.state {
                ParserState::Normal => match token.as_str() {
                    ":" => {
                        if self.state == ParserState::WordBody {
                            return Err(ForthErr::Msg("Unexpected :".to_string()));
                        }
                        self.state = ParserState::WordName;
                    }
                    ";" => {
                        return Err(ForthErr::Msg("Unexpected ;".to_string()));
                    }
                    "variable" => {
                        let var = self.next()?;
                        res.variables.insert(var.clone(), 0);
                        res.new_words
                            .insert(ForthOp::Variable(var.clone()), ForthFunc::Variable);
                    }
                    "constant" => {
                        let var = self.get_cur();
                        self.next()?;
                        res.new_words.insert(
                            ForthOp::UserWord(var.clone()),
                            ForthFunc::ConstantDef(var.clone()),
                        );
                        res.program
                            .push(ForthExp::Op(ForthOp::UserWord(var.clone())));
                        res.variables.insert(var, 0);
                    }
                    "@" => {
                        res.new_words.insert(
                            ForthOp::GetVar(self.cur),
                            ForthFunc::GetVar(self.get_var_name()?),
                        );
                        res.program.push(ForthExp::Op(ForthOp::GetVar(self.cur)));
                    }
                    "!" => {
                        res.new_words.insert(
                            ForthOp::SetVar(self.cur),
                            ForthFunc::SetVar(self.get_var_name()?),
                        );
                        res.program.push(ForthExp::Op(ForthOp::SetVar(self.cur)));
                    }
                    "if" => {
                        let expr = self.parse_if()?;
                        res.program
                            .push(ForthExp::Op(ForthOp::IfThenElse(self.cur)));
                        res.new_words
                            .insert(ForthOp::IfThenElse(self.cur), ForthFunc::IfThenElse(expr));
                    }
                    t => res.program.push(parse_word(t)?),
                },
                ParserState::WordName => {
                    self.word_name = token;
                    self.state = ParserState::WordBody
                }
                ParserState::WordBody => {
                    if token.to_ascii_lowercase().as_str() == "variable" {
                        let var = self.next()?;
                        res.variables.insert(var.clone(), 0);
                        res.new_words
                            .insert(ForthOp::Variable(var.clone()), ForthFunc::Variable);
                    } else if token == "if" {
                        let expr = self.parse_if()?;
                        self.new_word
                            .push(ForthExp::Op(ForthOp::IfThenElse(self.cur)));
                        res.new_words
                            .insert(ForthOp::IfThenElse(self.cur), ForthFunc::IfThenElse(expr));
                    } else if token == ";" {
                        self.state = ParserState::Normal;
                        res.new_words.insert(
                            ForthOp::UserWord(self.word_name.clone()),
                            ForthFunc::User(self.new_word.clone()),
                        );
                        self.word_name.clear();
                        self.new_word.clear();
                    } else if token == "@" {
                        res.new_words.insert(
                            ForthOp::GetVar(self.cur),
                            ForthFunc::GetVar(self.get_var_name()?),
                        );
                        self.new_word.push(ForthExp::Op(ForthOp::GetVar(self.cur)));
                    } else if token == "!" {
                        res.new_words.insert(
                            ForthOp::SetVar(self.cur),
                            ForthFunc::SetVar(self.get_var_name()?),
                        );
                        self.new_word.push(ForthExp::Op(ForthOp::SetVar(self.cur)));
                    } else {
                        self.new_word.push(parse_word(token.as_str())?)
                    }
                }
            }
        }
        Ok(res)
    }

    fn parse_simple(&mut self, tokens: Vec<String>) -> ForthResult<Vec<ForthExp>> {
        let mut res = vec![];
        for token in tokens {
            res.push(parse_word(token.as_str())?)
        }
        Ok(res)
    }

    fn parse_if(&mut self) -> ForthResult<(Vec<ForthExp>, Option<Vec<ForthExp>>)> {
        let mut then: Vec<String> = vec![];
        let mut r#else: Vec<String> = vec![];
        let mut else_found = false;
        while let Ok(token) = self.next() {
            match token.as_str() {
                "then" => break,
                "else" => else_found = true,
                _ => {
                    if else_found {
                        r#else.push(token)
                    } else {
                        then.push(token);
                    }
                }
            }
        }
        let then_parsed = self.parse_simple(then)?;
        if else_found {
            let else_parsed = self.parse_simple(r#else)?;
            Ok((then_parsed, Some(else_parsed)))
        } else {
            Ok((then_parsed, None))
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
