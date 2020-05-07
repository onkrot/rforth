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
                            ForthOp::Constant(var.clone()),
                            ForthFunc::ConstantDef(var.clone()),
                        );
                        res.program
                            .push(ForthExp::Op(ForthOp::Constant(var.clone())));
                        res.variables.insert(var, 0);
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
                    } else if token == ";" {
                        self.state = ParserState::Normal;
                        res.new_words.insert(
                            ForthOp::UserWord(self.word_name.clone()),
                            ForthFunc::User(self.new_word.clone()),
                        );
                        self.word_name.clear();
                        self.new_word.clear();
                    } else {
                        self.new_word.push(parse_word(token.as_str())?)
                    }
                }
            }
        }
        Ok(res)
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
                "@" => ForthOp::GetVar,
                "!" => ForthOp::SetVar,
                word => ForthOp::UserWord(word.to_string()),
            };
            ForthExp::Op(op)
        }
    };
    Ok(res)
}
