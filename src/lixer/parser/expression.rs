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
        let expr = match self.peek().clone() {
            Token::Int(n) => {
                self.advance();
                Expression::Int(n)
            }
            Token::Float(n) => {
                self.advance();
                Expression::Float(n)
            }
            Token::Str(s) => {
                self.advance();
                Expression::Str(s)
            }
            Token::Bytes(b) => {
                self.advance();
                Expression::Bytes(b)
            }
            Token::FStr(parts) => {
                self.advance();
                Expression::FString(parts)
            }
            Token::True => {
                self.advance();
                Expression::Bool(true)
            }
            Token::False => {
                self.advance();
                Expression::Bool(false)
            }
            Token::Null => {
                self.advance();
                Expression::Null
            }
            Token::Ident(name) => {
                self.advance();
                Expression::Ident(name)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(Token::RParen);
                expr
            }
            Token::LBracket => {
                self.advance();
                let mut items = Vec::new();
                if !matches!(self.peek(), Token::RBracket) {
                    items.push(self.parse_expression());
                    while matches!(self.peek(), Token::Comma) {
                        self.advance();
                        items.push(self.parse_expression());
                    }
                }
                self.expect(Token::RBracket);
                Expression::List(items)
            }
            Token::LBrace => {
                self.advance();
                let mut entries = Vec::new();
                if !matches!(self.peek(), Token::RBrace) {
                    let key = self.parse_expression();
                    self.expect(Token::Colon);
                    let value = self.parse_expression();
                    entries.push((key, value));
                    while matches!(self.peek(), Token::Comma) {
                        self.advance();
                        let key = self.parse_expression();
                        self.expect(Token::Colon);
                        let value = self.parse_expression();
                        entries.push((key, value));
                    }
                }
                self.expect(Token::RBrace);
                Expression::Map(entries)
            }
            t => {
                eprintln!("parse error: expected literal or identifier, got {:?}", t);
                std::process::exit(1);
            }
        };
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, base: Expression) -> Expression {
        let mut expr = base;
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    if self.is_named_args() {
                        let mut named_args = Vec::new();
                        if !matches!(self.peek(), Token::RParen) {
                            named_args.push(self.parse_named_arg());
                            while matches!(self.peek(), Token::Comma) {
                                self.advance();
                                named_args.push(self.parse_named_arg());
                            }
                        }
                        self.expect(Token::RParen);
                        if let Expression::Ident(name) = &expr {
                            expr = Expression::Construct {
                                type_name: name.clone(),
                                fields: named_args,
                            };
                        }
                    } else {
                        let mut args = Vec::new();
                        if !matches!(self.peek(), Token::RParen) {
                            args.push(self.parse_expression());
                            while matches!(self.peek(), Token::Comma) {
                                self.advance();
                                args.push(self.parse_expression());
                            }
                        }
                        self.expect(Token::RParen);
                        if let Expression::FieldAccess { object, field } = &expr {
                            expr = Expression::MethodCall {
                                object: object.clone(),
                                method: field.clone(),
                                args,
                            };
                        } else {
                            expr = Expression::Call {
                                callee: Box::new(expr),
                                args,
                            };
                        }
                    }
                }
                Token::Dot => {
                    self.advance();
                    let field = match self.advance() {
                        Token::Ident(name) => name,
                        t => {
                            eprintln!("parse error: expected field name, got {:?}", t);
                            std::process::exit(1);
                        }
                    };
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expression();
                    self.expect(Token::RBracket);
                    expr = Expression::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        expr
    }

    fn is_named_args(&self) -> bool {
        matches!(
            (self.peek(), self.tokens.get(self.pos + 1)),
            (Token::Ident(_), Some(Token::Colon))
        )
    }

    fn parse_named_arg(&mut self) -> (String, Expression) {
        let name = match self.advance() {
            Token::Ident(name) => name,
            t => {
                eprintln!("parse error: expected argument name, got {:?}", t);
                std::process::exit(1);
            }
        };
        self.expect(Token::Colon);
        let expr = self.parse_expression();
        (name, expr)
    }
}
