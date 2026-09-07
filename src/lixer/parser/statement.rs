use crate::lixer::ast::statement::{ElifBranch, Statement};
use crate::lixer::lexer::token::Token;
use crate::lixer::parser::expression::Parser;

pub fn parse(tokens: &[Token]) -> Vec<Statement> {
    let mut p = Parser::new(tokens);
    let mut stmts = Vec::new();

    while !p.is_eof() {
        p.skip_newlines();
        if p.is_eof() {
            break;
        }
        stmts.push(parse_statement(&mut p));
    }
    stmts
}

fn parse_statement(p: &mut Parser) -> Statement {
    match p.peek() {
        Token::Print => {
            p.advance();
            p.expect(Token::LParen);
            let expr = p.parse_expression();
            p.expect(Token::RParen);
            expect_end(p);
            Statement::Print(expr)
        }
        Token::PrintLn => {
            p.advance();
            p.expect(Token::LParen);
            let expr = p.parse_expression();
            p.expect(Token::RParen);
            expect_end(p);
            Statement::PrintLn(expr)
        }
        Token::Let => {
            p.advance();
            let name = expect_ident(p);
            p.expect(Token::Assign);
            let expr = p.parse_expression();
            expect_end(p);
            Statement::Let { name, expr }
        }
        Token::Const => {
            p.advance();
            let name = expect_ident(p);
            p.expect(Token::Assign);
            let expr = p.parse_expression();
            expect_end(p);
            Statement::Const { name, expr }
        }
        Token::If => parse_if(p),
        Token::While => parse_while(p),
        Token::For => parse_for(p),
        Token::Break => {
            p.advance();
            expect_end(p);
            Statement::Break
        }
        Token::Continue => {
            p.advance();
            expect_end(p);
            Statement::Continue
        }
        Token::Ident(name) => {
            let name = name.clone();
            p.advance();
            p.expect(Token::Assign);
            let expr = p.parse_expression();
            expect_end(p);
            Statement::Assign { name, expr }
        }
        t => {
            eprintln!("parse error: expected statement, got {:?}", t);
            std::process::exit(1);
        }
    }
}

fn parse_if(p: &mut Parser) -> Statement {
    p.advance();
    p.expect(Token::LParen);
    let condition = p.parse_expression();
    p.expect(Token::RParen);
    let then_body = parse_block(p);

    let mut elif_branches = Vec::new();
    let mut else_body = None;

    loop {
        p.skip_newlines();
        match p.peek() {
            Token::Elif => {
                p.advance();
                p.expect(Token::LParen);
                let cond = p.parse_expression();
                p.expect(Token::RParen);
                let body = parse_block(p);
                elif_branches.push(ElifBranch {
                    condition: cond,
                    body,
                });
            }
            Token::Else => {
                p.advance();
                else_body = Some(parse_block(p));
                break;
            }
            _ => break,
        }
    }

    Statement::If {
        condition,
        then_body,
        elif_branches,
        else_body,
    }
}

fn parse_block(p: &mut Parser) -> Vec<Statement> {
    p.expect(Token::LBrace);
    p.skip_newlines();
    let mut stmts = Vec::new();
    while !matches!(p.peek(), Token::RBrace | Token::Eof) {
        stmts.push(parse_statement(p));
        p.skip_newlines();
    }
    p.expect(Token::RBrace);
    stmts
}

fn parse_while(p: &mut Parser) -> Statement {
    p.advance();
    p.expect(Token::LParen);
    let condition = p.parse_expression();
    p.expect(Token::RParen);
    let body = parse_block(p);
    Statement::While { condition, body }
}

fn parse_for(p: &mut Parser) -> Statement {
    p.advance();
    p.expect(Token::LParen);
    let var = expect_ident(p);
    p.expect(Token::In);
    let start = p.parse_expression();
    p.expect(Token::Dot);
    p.expect(Token::Dot);
    let end = p.parse_expression();
    p.expect(Token::RParen);
    let body = parse_block(p);
    Statement::For {
        var,
        start,
        end,
        body,
    }
}

fn expect_ident(p: &mut Parser) -> String {
    match p.advance() {
        Token::Ident(name) => name,
        t => {
            eprintln!("parse error: expected identifier, got {:?}", t);
            std::process::exit(1);
        }
    }
}

fn expect_end(p: &mut Parser) {
    match p.peek() {
        Token::Semicolon => {
            p.advance();
        }
        Token::Newline => {
            p.advance();
        }
        Token::Eof => {}
        _ => {
            eprintln!("parse error: expected ';' or newline, got {:?}", p.peek());
            std::process::exit(1);
        }
    }
}
