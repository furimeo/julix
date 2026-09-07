use crate::bytecode::chunk::Chunk;
use crate::bytecode::instruction::Instruction;
use crate::lixer::ast::expression::Expression;
use crate::lixer::ast::statement::Statement;

pub fn compile(stmts: &[Statement]) -> Chunk {
    let mut chunk = Chunk::new();
    for stmt in stmts {
        compile_statement(stmt, &mut chunk);
    }
    chunk.push(Instruction::Halt);
    patch_jumps(&mut chunk);
    chunk
}

fn compile_statement(stmt: &Statement, chunk: &mut Chunk) {
    match stmt {
        Statement::Print(expr) => {
            compile_expression(expr, chunk);
            chunk.push(Instruction::Print);
        }
        Statement::PrintLn(expr) => {
            compile_expression(expr, chunk);
            chunk.push(Instruction::PrintLn);
        }
        Statement::Let { name, expr } | Statement::Const { name, expr } => {
            compile_expression(expr, chunk);
            chunk.push(Instruction::StoreVar(name.clone()));
        }
        Statement::If {
            condition,
            then_body,
            elif_branches,
            else_body,
        } => {
            compile_expression(condition, chunk);
            let jump_false = chunk.len();
            chunk.push(Instruction::JumpIfFalse(0));
            for s in then_body {
                compile_statement(s, chunk);
            }
            let jump_end = chunk.len();
            chunk.push(Instruction::Jump(0));
            chunk.code[jump_false] = Instruction::JumpIfFalse(chunk.len());
            if let Some(branch) = elif_branches.iter().next() {
                compile_expression(&branch.condition, chunk);
                let jf = chunk.len();
                chunk.push(Instruction::JumpIfFalse(0));
                for s in &branch.body {
                    compile_statement(s, chunk);
                }
                let je = chunk.len();
                chunk.push(Instruction::Jump(0));
                chunk.code[jf] = Instruction::JumpIfFalse(chunk.len());
                if let Some(else_body) = else_body {
                    for s in else_body {
                        compile_statement(s, chunk);
                    }
                }
                chunk.code[je] = Instruction::Jump(chunk.len());
                return;
            }
            if let Some(else_body) = else_body {
                for s in else_body {
                    compile_statement(s, chunk);
                }
            }
            chunk.code[jump_end] = Instruction::Jump(chunk.len());
        }
    }
}

fn compile_expression(expr: &Expression, chunk: &mut Chunk) {
    match expr {
        Expression::Int(n) => chunk.push(Instruction::LoadInt(*n)),
        Expression::Str(s) => chunk.push(Instruction::LoadStr(s.clone())),
        Expression::Bool(b) => chunk.push(Instruction::LoadBool(*b)),
        Expression::Ident(name) => chunk.push(Instruction::LoadVar(name.clone())),
        Expression::Add(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Add);
        }
        Expression::Sub(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Sub);
        }
        Expression::Mul(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Mul);
        }
        Expression::Div(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Div);
        }
        Expression::Mod(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Mod);
        }
        Expression::Eq(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Eq);
        }
        Expression::NotEq(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::NotEq);
        }
        Expression::Lt(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Lt);
        }
        Expression::Gt(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Gt);
        }
        Expression::LtEq(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::LtEq);
        }
        Expression::GtEq(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::GtEq);
        }
        Expression::And(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::And);
        }
        Expression::Or(l, r) => {
            compile_expression(l, chunk);
            compile_expression(r, chunk);
            chunk.push(Instruction::Or);
        }
        Expression::Not(e) => {
            compile_expression(e, chunk);
            chunk.push(Instruction::Not);
        }
    }
}

fn patch_jumps(_chunk: &mut Chunk) {}
