use crate::lixer::ast::expression::Expression;
use crate::lixer::lexer::token::Token;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    pub fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.pos += 1;
        }
    }

    pub fn expect(&mut self, expected: Token) {
        let got = self.advance();
        if got != expected {
            eprintln!("parse error: expected {:?}, got {:?}", expected, got);
            std::process::exit(1);
        }
    }

    pub fn parse_expression(&mut self) -> Expression {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Expression {
        let mut left = self.parse_and();
        loop {
            match self.peek() {
                Token::OrOr | Token::Or => {
                    self.advance();
                    let right = self.parse_and();
                    left = Expression::Or(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_and(&mut self) -> Expression {
        let mut left = self.parse_not();
        loop {
            match self.peek() {
                Token::AndAnd | Token::And => {
                    self.advance();
                    let right = self.parse_not();
                    left = Expression::And(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_not(&mut self) -> Expression {
        match self.peek() {
            Token::Bang | Token::Not => {
                self.advance();
                let expr = self.parse_not();
                Expression::Not(Box::new(expr))
            }
            _ => self.parse_comparison(),
        }
    }

    fn parse_comparison(&mut self) -> Expression {
        let left = self.parse_additive();
        match self.peek() {
            Token::EqEq => {
                self.advance();
                let right = self.parse_additive();
                Expression::Eq(Box::new(left), Box::new(right))
            }
            Token::NotEq => {
                self.advance();
                let right = self.parse_additive();
                Expression::NotEq(Box::new(left), Box::new(right))
            }
            Token::Lt => {
                self.advance();
                let right = self.parse_additive();
                Expression::Lt(Box::new(left), Box::new(right))
            }
            Token::Gt => {
                self.advance();
                let right = self.parse_additive();
                Expression::Gt(Box::new(left), Box::new(right))
            }
            Token::LtEq => {
                self.advance();
                let right = self.parse_additive();
                Expression::LtEq(Box::new(left), Box::new(right))
            }
            Token::GtEq => {
                self.advance();
                let right = self.parse_additive();
                Expression::GtEq(Box::new(left), Box::new(right))
            }
            _ => left,
        }
    }

    fn parse_additive(&mut self) -> Expression {
        let mut left = self.parse_multiplicative();
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    left = Expression::Add(Box::new(left), Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    left = Expression::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expression {
        let mut left = self.parse_primary();
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_primary();
                    left = Expression::Mul(Box::new(left), Box::new(right));
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_primary();
                    left = Expression::Div(Box::new(left), Box::new(right));
                }
                Token::Percent => {
                    self.advance();
                    let right = self.parse_primary();
                    left = Expression::Mod(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_primary(&mut self) -> Expression {
        match self.advance() {
            Token::Int(n) => Expression::Int(n),
            Token::Str(s) => Expression::Str(s),
            Token::True => Expression::Bool(true),
            Token::False => Expression::Bool(false),
            Token::Ident(name) => Expression::Ident(name),
            Token::LParen => {
                let expr = self.parse_expression();
                self.expect(Token::RParen);
                expr
            }
            t => {
                eprintln!("parse error: expected literal or identifier, got {:?}", t);
                std::process::exit(1);
            }
        }
    }
}
