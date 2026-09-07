use crate::lixer::ast::expression::Expression;
use crate::lixer::ast::statement::{ElifBranch, FunctionDef, Statement};
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
        Token::Function => parse_function(p),
        Token::Return => parse_return(p),
        Token::Type => parse_type(p),
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
        Token::Ident(_) => {
            let expr = p.parse_expression();
            if matches!(p.peek(), Token::Assign) {
                p.advance();
                let value = p.parse_expression();
                expect_end(p);
                if let Expression::Ident(name) = &expr {
                    return Statement::Assign {
                        name: name.clone(),
                        expr: value,
                    };
                }
                if let Expression::FieldAccess { object, field } = &expr {
                    return Statement::FieldAssign {
                        object: (**object).clone(),
                        field: field.clone(),
                        expr: value,
                    };
                }
                eprintln!("error: cannot assign to this expression");
                std::process::exit(1);
            }
            expect_end(p);
            Statement::Expr(expr)
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
    p.expect(Token::Range);
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

fn parse_type(p: &mut Parser) -> Statement {
    p.advance();
    let name = expect_ident(p);
    p.expect(Token::LBrace);
    p.skip_newlines();

    let mut fields = Vec::new();
    let mut methods = Vec::new();

    while !matches!(p.peek(), Token::RBrace | Token::Eof) {
        match p.peek() {
            Token::Function => {
                p.advance();
                let mname = expect_ident(p);
                p.expect(Token::LParen);
                let mut params = Vec::new();
                if !matches!(p.peek(), Token::RParen) {
                    params.push(expect_ident(p));
                    while matches!(p.peek(), Token::Comma) {
                        p.advance();
                        params.push(expect_ident(p));
                    }
                }
                p.expect(Token::RParen);
                let body = parse_block(p);
                methods.push(FunctionDef {
                    name: mname,
                    params,
                    body,
                });
                p.skip_newlines();
            }
            _ => {
                let field_name = expect_ident(p);
                p.expect(Token::Colon);
                let field_type = expect_ident(p);
                expect_end(p);
                p.skip_newlines();
                fields.push((field_name, field_type));
            }
        }
    }

    p.expect(Token::RBrace);
    Statement::TypeDef {
        name,
        fields,
        methods,
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

fn parse_function(p: &mut Parser) -> Statement {
    p.advance();
    let name = expect_ident(p);
    p.expect(Token::LParen);
    let mut params = Vec::new();
    if !matches!(p.peek(), Token::RParen) {
        params.push(expect_ident(p));
        while matches!(p.peek(), Token::Comma) {
            p.advance();
            params.push(expect_ident(p));
        }
    }
    p.expect(Token::RParen);
    let body = parse_block(p);
    Statement::FunctionDef { name, params, body }
}

fn parse_return(p: &mut Parser) -> Statement {
    p.advance();
    if matches!(
        p.peek(),
        Token::Semicolon | Token::Newline | Token::Eof | Token::RBrace
    ) {
        expect_end(p);
        return Statement::Return(None);
    }
    let expr = p.parse_expression();
    expect_end(p);
    Statement::Return(Some(expr))
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
