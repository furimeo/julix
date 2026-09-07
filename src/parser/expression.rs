use crate::ast::expression::Expression;
use crate::lexer::token::Token;

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
        let mut left = self.parse_term();
        while matches!(self.peek(), Token::Plus) {
            self.advance();
            let right = self.parse_term();
            left = Expression::Add(Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_term(&mut self) -> Expression {
        match self.advance() {
            Token::Int(n) => Expression::Int(n),
            Token::Str(s) => Expression::Str(s),
            t => {
                eprintln!("parse error: expected literal, got {:?}", t);
                std::process::exit(1);
            }
        }
    }
}
