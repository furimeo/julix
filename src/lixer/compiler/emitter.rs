use crate::bytecode::chunk::{Chunk, FunctionDef, FunctionTable};
use crate::bytecode::instruction::Instruction;
use crate::lixer::ast::expression::Expression;
use crate::lixer::ast::statement::Statement;

const BREAK_SENTINEL: usize = usize::MAX;
const CONTINUE_SENTINEL: usize = usize::MAX - 1;

pub fn compile(stmts: &[Statement]) -> (Chunk, FunctionTable) {
    let mut chunk = Chunk::new();
    let mut functions = FunctionTable::new();
    for stmt in stmts {
        compile_statement(stmt, &mut chunk, &mut functions, None);
    }
    chunk.push(Instruction::Deinit);
    chunk.push(Instruction::Halt);
    (chunk, functions)
}

fn compile_statement(
    stmt: &Statement,
    chunk: &mut Chunk,
    functions: &mut FunctionTable,
    loop_info: Option<(usize, usize)>,
) {
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
        Statement::Assign { name, expr } => {
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
                compile_statement(s, chunk, functions, loop_info);
            }
            let jump_end = chunk.len();
            chunk.push(Instruction::Jump(0));
            chunk.code[jump_false] = Instruction::JumpIfFalse(chunk.len());

            for branch in elif_branches {
                compile_expression(&branch.condition, chunk);
                let jf = chunk.len();
                chunk.push(Instruction::JumpIfFalse(0));
                for s in &branch.body {
                    compile_statement(s, chunk, functions, loop_info);
                }
                let je = chunk.len();
                chunk.push(Instruction::Jump(0));
                chunk.code[jf] = Instruction::JumpIfFalse(chunk.len());
                chunk.code[je] = Instruction::Jump(chunk.len());
            }

            if let Some(else_body) = else_body {
                for s in else_body {
                    compile_statement(s, chunk, functions, loop_info);
                }
            }
            chunk.code[jump_end] = Instruction::Jump(chunk.len());
        }
        Statement::While { condition, body } => {
            let loop_start = chunk.len();
            compile_expression(condition, chunk);
            let jump_exit = chunk.len();
            chunk.push(Instruction::JumpIfFalse(0));
            for s in body {
                compile_statement(s, chunk, functions, Some((loop_start, 0)));
            }
            chunk.push(Instruction::Jump(loop_start));
            let loop_end = chunk.len();
            chunk.code[jump_exit] = Instruction::JumpIfFalse(loop_end);
            patch_loop_jumps(chunk, loop_start, loop_start, loop_end);
        }
        Statement::For {
            var,
            start,
            end,
            body,
        } => {
            compile_expression(start, chunk);
            chunk.push(Instruction::StoreVar(var.clone()));
            compile_expression(end, chunk);
            chunk.push(Instruction::StoreVar(format!("__end_{}", var)));
            let loop_start = chunk.len();
            chunk.push(Instruction::LoadVar(var.clone()));
            chunk.push(Instruction::LoadVar(format!("__end_{}", var)));
            chunk.push(Instruction::Lt);
            let jump_exit = chunk.len();
            chunk.push(Instruction::JumpIfFalse(0));
            for s in body {
                compile_statement(s, chunk, functions, Some((loop_start, 0)));
            }
            let continue_target = chunk.len();
            chunk.push(Instruction::LoadVar(var.clone()));
            chunk.push(Instruction::LoadInt(1));
            chunk.push(Instruction::Add);
            chunk.push(Instruction::StoreVar(var.clone()));
            chunk.push(Instruction::Jump(loop_start));
            let loop_end = chunk.len();
            chunk.code[jump_exit] = Instruction::JumpIfFalse(loop_end);
            patch_loop_jumps(chunk, loop_start, continue_target, loop_end);
        }
        Statement::FunctionDef { name, params, body } => {
            let mut func_chunk = Chunk::new();
            for s in body {
                compile_statement(s, &mut func_chunk, functions, None);
            }
            func_chunk.push(Instruction::Deinit);
            func_chunk.push(Instruction::Return);
            functions.insert(
                name.clone(),
                FunctionDef {
                    chunk: func_chunk,
                    params: params.clone(),
                },
            );
        }
        Statement::Return(expr) => {
            if let Some(e) = expr {
                compile_expression(e, chunk);
            } else {
                chunk.push(Instruction::LoadBool(false));
            }
            chunk.push(Instruction::Deinit);
            chunk.push(Instruction::Return);
        }
        Statement::Expr(expr) => {
            compile_expression(expr, chunk);
            chunk.push(Instruction::Pop);
        }
        Statement::TypeDef {
            name,
            fields,
            methods,
        } => {
            for method in methods {
                let mangled = format!("{}.{}", name, method.name);
                let mut func_chunk = Chunk::new();
                for s in &method.body {
                    compile_statement(s, &mut func_chunk, functions, None);
                }
                func_chunk.push(Instruction::Deinit);
                func_chunk.push(Instruction::Return);
                functions.insert(
                    mangled,
                    FunctionDef {
                        chunk: func_chunk,
                        params: method.params.clone(),
                    },
                );
            }
            let _ = fields;
        }
        Statement::FieldAssign {
            object,
            field,
            expr,
        } => {
            compile_expression(object, chunk);
            compile_expression(expr, chunk);
            chunk.push(Instruction::SetField(field.clone()));
            if let Expression::Ident(name) = object {
                chunk.push(Instruction::StoreVar(name.clone()));
            }
        }
        Statement::IndexAssign {
            object,
            index,
            expr,
        } => {
            compile_expression(object, chunk);
            compile_expression(index, chunk);
            compile_expression(expr, chunk);
            chunk.push(Instruction::IndexSet);
            if let Expression::Ident(name) = object {
                chunk.push(Instruction::StoreVar(name.clone()));
            }
        }
        Statement::Break => {
            if loop_info.is_some() {
                chunk.push(Instruction::Jump(BREAK_SENTINEL));
            } else {
                eprintln!("error: break outside loop");
                std::process::exit(1);
            }
        }
        Statement::Continue => {
            if loop_info.is_some() {
                chunk.push(Instruction::Jump(CONTINUE_SENTINEL));
            } else {
                eprintln!("error: continue outside loop");
                std::process::exit(1);
            }
        }
        Statement::Throw(expr) => {
            compile_expression(expr, chunk);
            chunk.push(Instruction::Throw);
        }
        Statement::Try {
            body,
            catch_var,
            catch_body,
        } => {
            let try_catch_pos = chunk.len();
            chunk.push(Instruction::TryCatch(0, 0));
            for s in body {
                compile_statement(s, chunk, functions, loop_info);
            }
            let jump_end_pos = chunk.len();
            chunk.push(Instruction::Jump(0));
            let catch_pos = chunk.len();
            if let Some(var) = catch_var {
                chunk.push(Instruction::StoreVar(var.clone()));
            } else {
                chunk.push(Instruction::Pop);
            }
            for s in catch_body {
                compile_statement(s, chunk, functions, loop_info);
            }
            let end_pos = chunk.len();
            chunk.code[try_catch_pos] = Instruction::TryCatch(catch_pos, end_pos);
            chunk.code[jump_end_pos] = Instruction::Jump(end_pos);
        }
    }
}

fn patch_loop_jumps(chunk: &mut Chunk, loop_start: usize, continue_target: usize, loop_end: usize) {
    for i in loop_start..loop_end {
        match &chunk.code[i] {
            Instruction::Jump(BREAK_SENTINEL) => {
                chunk.code[i] = Instruction::Jump(loop_end);
            }
            Instruction::Jump(CONTINUE_SENTINEL) => {
                chunk.code[i] = Instruction::Jump(continue_target);
            }
            _ => {}
        }
    }
}

fn compile_expression(expr: &Expression, chunk: &mut Chunk) {
    match expr {
        Expression::Int(n) => chunk.push(Instruction::LoadInt(*n)),
        Expression::Float(n) => chunk.push(Instruction::LoadFloat(*n)),
        Expression::Null => chunk.push(Instruction::LoadNull),
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
        Expression::Range(_, _) => {}
        Expression::Call { callee, args } => {
            for arg in args {
                compile_expression(arg, chunk);
            }
            if let Expression::Ident(name) = callee.as_ref() {
                chunk.push(Instruction::Call(name.clone(), args.len()));
            } else {
                eprintln!("error: cannot call non-identifier");
                std::process::exit(1);
            }
        }
        Expression::FieldAccess { object, field } => {
            compile_expression(object, chunk);
            chunk.push(Instruction::GetField(field.clone()));
        }
        Expression::MethodCall {
            object,
            method,
            args,
        } => {
            compile_expression(object, chunk);
            for arg in args {
                compile_expression(arg, chunk);
            }
            chunk.push(Instruction::MethodCall(method.clone(), args.len()));
        }
        Expression::Construct { type_name, fields } => {
            let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
            for (_, e) in fields {
                compile_expression(e, chunk);
            }
            chunk.push(Instruction::Construct(type_name.clone(), field_names));
        }
        Expression::Bytes(b) => {
            chunk.push(Instruction::LoadBytes(b.clone()));
        }
        Expression::List(items) => {
            for item in items {
                compile_expression(item, chunk);
            }
            chunk.push(Instruction::NewList(items.len()));
        }
        Expression::Map(entries) => {
            for (key, value) in entries {
                compile_expression(key, chunk);
                compile_expression(value, chunk);
            }
            chunk.push(Instruction::NewMap(entries.len()));
        }
        Expression::Index { object, index } => {
            compile_expression(object, chunk);
            compile_expression(index, chunk);
            chunk.push(Instruction::IndexGet);
        }
        Expression::IndexSet {
            object,
            index,
            value,
        } => {
            compile_expression(object, chunk);
            compile_expression(index, chunk);
            compile_expression(value, chunk);
            chunk.push(Instruction::IndexSet);
        }
        Expression::FString(parts) => {
            let mut first = true;
            for part in parts {
                match part {
                    crate::lixer::lexer::token::FStrPart::Literal(s) => {
                        chunk.push(Instruction::LoadStr(s.clone()));
                        if !first {
                            chunk.push(Instruction::Add);
                        }
                        first = false;
                    }
                    crate::lixer::lexer::token::FStrPart::Expr(e) => {
                        let tokens = crate::lixer::lexer::token::lex(e);
                        let mut p = crate::lixer::parser::expression::Parser::new(&tokens);
                        let expr = p.parse_expression();
                        compile_expression(&expr, chunk);
                        chunk.push(Instruction::Stringify);
                        if !first {
                            chunk.push(Instruction::Add);
                        }
                        first = false;
                    }
                }
            }
            if first {
                chunk.push(Instruction::LoadStr(String::new()));
            }
        }
    }
}
