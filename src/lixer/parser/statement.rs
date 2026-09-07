use crate::ast::statement::Statement;
use crate::lexer::token::Token;
use crate::parser::expression::Parser;

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
        t => {
            eprintln!("parse error: expected statement, got {:?}", t);
            std::process::exit(1);
        }
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
