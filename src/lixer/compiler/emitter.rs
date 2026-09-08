use crate::bytecode::chunk::{Chunk, FunctionDef, FunctionTable};
use crate::bytecode::instruction::Instruction;
use crate::lixer::ast::expression::Expression;
use crate::lixer::ast::statement::Statement;

const BREAK_SENTINEL: usize = usize::MAX;
const CONTINUE_SENTINEL: usize = usize::MAX - 1;

struct SlotCtx {
    slots: std::collections::HashMap<String, u16>,
    count: u16,
}

impl SlotCtx {
    fn new() -> Self {
        SlotCtx {
            slots: std::collections::HashMap::new(),
            count: 0,
        }
    }

    fn get_or_alloc(&mut self, name: &str) -> u16 {
        if let Some(&slot) = self.slots.get(name) {
            return slot;
        }
        let slot = self.count;
        self.count += 1;
        self.slots.insert(name.to_string(), slot);
        slot
    }

    fn get(&self, name: &str) -> Option<u16> {
        self.slots.get(name).copied()
    }
}

pub fn compile(stmts: &[Statement]) -> (Chunk, FunctionTable, u16) {
    let mut chunk = Chunk::new();
    let mut functions = FunctionTable::new();
    let mut slots = SlotCtx::new();
    for stmt in stmts {
        compile_statement(stmt, &mut chunk, &mut functions, &mut slots, None);
    }
    chunk.push(Instruction::Deinit);
    chunk.push(Instruction::Halt);
    (chunk, functions, slots.count)
}

fn compile_statement(
    stmt: &Statement,
    chunk: &mut Chunk,
    functions: &mut FunctionTable,
    slots: &mut SlotCtx,
    loop_info: Option<(usize, usize)>,
) {
    match stmt {
        Statement::Print(expr) => {
            compile_expression(expr, chunk, slots);
            chunk.push(Instruction::Print);
        }
        Statement::PrintLn(expr) => {
            compile_expression(expr, chunk, slots);
            chunk.push(Instruction::PrintLn);
        }
        Statement::Let { name, expr } | Statement::Const { name, expr } => {
            compile_expression(expr, chunk, slots);
            let slot = slots.get_or_alloc(name);
            chunk.push(Instruction::StoreSlot(slot));
        }
        Statement::Assign { name, expr } => {
            compile_expression(expr, chunk, slots);
            let slot = slots.get_or_alloc(name);
            chunk.push(Instruction::StoreSlot(slot));
        }
        Statement::If {
            condition,
            then_body,
            elif_branches,
            else_body,
        } => {
            compile_expression(condition, chunk, slots);
            let jump_false = chunk.len();
            chunk.push(Instruction::JumpIfFalse(0));
            for s in then_body {
                compile_statement(s, chunk, functions, slots, loop_info);
            }
            let jump_end = chunk.len();
            chunk.push(Instruction::Jump(0));
            chunk.code[jump_false] = Instruction::JumpIfFalse(chunk.len());

            for branch in elif_branches {
                compile_expression(&branch.condition, chunk, slots);
                let jf = chunk.len();
                chunk.push(Instruction::JumpIfFalse(0));
                for s in &branch.body {
                    compile_statement(s, chunk, functions, slots, loop_info);
                }
                let je = chunk.len();
                chunk.push(Instruction::Jump(0));
                chunk.code[jf] = Instruction::JumpIfFalse(chunk.len());
                chunk.code[je] = Instruction::Jump(chunk.len());
            }

            if let Some(else_body) = else_body {
                for s in else_body {
                    compile_statement(s, chunk, functions, slots, loop_info);
                }
            }
            chunk.code[jump_end] = Instruction::Jump(chunk.len());
        }
        Statement::While { condition, body } => {
            let loop_start = chunk.len();
            compile_expression(condition, chunk, slots);
            let jump_exit = chunk.len();
            chunk.push(Instruction::JumpIfFalse(0));
            for s in body {
                compile_statement(s, chunk, functions, slots, Some((loop_start, 0)));
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
            let var_slot = slots.get_or_alloc(var);
            let end_name = format!("__end_{}", var);
            let end_slot = slots.get_or_alloc(&end_name);

            compile_expression(start, chunk, slots);
            chunk.push(Instruction::StoreSlot(var_slot));
            compile_expression(end, chunk, slots);
            chunk.push(Instruction::StoreSlot(end_slot));
            let loop_start = chunk.len();
            chunk.push(Instruction::LoadSlot(var_slot));
            chunk.push(Instruction::LoadSlot(end_slot));
            chunk.push(Instruction::Lt);
            let jump_exit = chunk.len();
            chunk.push(Instruction::JumpIfFalse(0));
            for s in body {
                compile_statement(s, chunk, functions, slots, Some((loop_start, 0)));
            }
            let continue_target = chunk.len();
            chunk.push(Instruction::LoadSlot(var_slot));
            chunk.push(Instruction::LoadInt(1));
            chunk.push(Instruction::Add);
            chunk.push(Instruction::StoreSlot(var_slot));
            chunk.push(Instruction::Jump(loop_start));
            let loop_end = chunk.len();
            chunk.code[jump_exit] = Instruction::JumpIfFalse(loop_end);
            patch_loop_jumps(chunk, loop_start, continue_target, loop_end);
        }
        Statement::FunctionDef { name, params, body } => {
            let mut func_chunk = Chunk::new();
            let mut func_slots = SlotCtx::new();
            for (i, param) in params.iter().enumerate() {
                func_slots.get_or_alloc(param);
                let _ = i;
            }
            for s in body {
                compile_statement(s, &mut func_chunk, functions, &mut func_slots, None);
            }
            func_chunk.push(Instruction::Deinit);
            func_chunk.push(Instruction::Return);
            let param_slots: Vec<u16> = params
                .iter()
                .map(|p| func_slots.get(p).unwrap_or(0))
                .collect();
            functions.insert(
                name.clone(),
                FunctionDef {
                    chunk: func_chunk,
                    params: param_slots,
                    slot_count: func_slots.count,
                },
            );
        }
        Statement::Return(expr) => {
            if let Some(e) = expr {
                compile_expression(e, chunk, slots);
            } else {
                chunk.push(Instruction::LoadBool(false));
            }
            chunk.push(Instruction::Deinit);
            chunk.push(Instruction::Return);
        }
        Statement::Expr(expr) => {
            compile_expression(expr, chunk, slots);
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
                let mut func_slots = SlotCtx::new();
                for param in &method.params {
                    func_slots.get_or_alloc(param);
                }
                for s in &method.body {
                    compile_statement(s, &mut func_chunk, functions, &mut func_slots, None);
                }
                func_chunk.push(Instruction::Deinit);
                func_chunk.push(Instruction::Return);
                let param_slots: Vec<u16> = method
                    .params
                    .iter()
                    .map(|p| func_slots.get(p).unwrap_or(0))
                    .collect();
                functions.insert(
                    mangled,
                    FunctionDef {
                        chunk: func_chunk,
                        params: param_slots,
                        slot_count: func_slots.count,
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
            compile_expression(object, chunk, slots);
            compile_expression(expr, chunk, slots);
            chunk.push(Instruction::SetField(field.clone()));
            if let Expression::Ident(name) = object {
                let slot = slots.get_or_alloc(name);
                chunk.push(Instruction::StoreSlot(slot));
            }
        }
        Statement::IndexAssign {
            object,
            index,
            expr,
        } => {
            compile_expression(object, chunk, slots);
            compile_expression(index, chunk, slots);
            compile_expression(expr, chunk, slots);
            chunk.push(Instruction::IndexSet);
            if let Expression::Ident(name) = object {
                let slot = slots.get_or_alloc(name);
                chunk.push(Instruction::StoreSlot(slot));
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
            compile_expression(expr, chunk, slots);
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
                compile_statement(s, chunk, functions, slots, loop_info);
            }
            let jump_end_pos = chunk.len();
            chunk.push(Instruction::Jump(0));
            let catch_pos = chunk.len();
            if let Some(var) = catch_var {
                let slot = slots.get_or_alloc(var);
                chunk.push(Instruction::StoreSlot(slot));
            } else {
                chunk.push(Instruction::Pop);
            }
            for s in catch_body {
                compile_statement(s, chunk, functions, slots, loop_info);
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

fn compile_expression(expr: &Expression, chunk: &mut Chunk, slots: &mut SlotCtx) {
    match expr {
        Expression::Int(n) => chunk.push(Instruction::LoadInt(*n)),
        Expression::Float(n) => chunk.push(Instruction::LoadFloat(*n)),
        Expression::Null => chunk.push(Instruction::LoadNull),
        Expression::Str(s) => chunk.push(Instruction::LoadStr(s.clone())),
        Expression::Bool(b) => chunk.push(Instruction::LoadBool(*b)),
        Expression::Ident(name) => {
            let slot = slots.get_or_alloc(name);
            chunk.push(Instruction::LoadSlot(slot));
        }
        Expression::Add(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Add);
        }
        Expression::Sub(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Sub);
        }
        Expression::Mul(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Mul);
        }
        Expression::Div(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Div);
        }
        Expression::Mod(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Mod);
        }
        Expression::Eq(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Eq);
        }
        Expression::NotEq(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::NotEq);
        }
        Expression::Lt(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Lt);
        }
        Expression::Gt(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Gt);
        }
        Expression::LtEq(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::LtEq);
        }
        Expression::GtEq(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::GtEq);
        }
        Expression::And(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::And);
        }
        Expression::Or(l, r) => {
            compile_expression(l, chunk, slots);
            compile_expression(r, chunk, slots);
            chunk.push(Instruction::Or);
        }
        Expression::Not(e) => {
            compile_expression(e, chunk, slots);
            chunk.push(Instruction::Not);
        }
        Expression::Call { callee, args } => {
            for arg in args {
                compile_expression(arg, chunk, slots);
            }
            if let Expression::Ident(name) = callee.as_ref() {
                chunk.push(Instruction::Call(name.clone(), args.len()));
            } else {
                eprintln!("error: cannot call non-identifier");
                std::process::exit(1);
            }
        }
        Expression::FieldAccess { object, field } => {
            compile_expression(object, chunk, slots);
            chunk.push(Instruction::GetField(field.clone()));
        }
        Expression::MethodCall {
            object,
            method,
            args,
        } => {
            compile_expression(object, chunk, slots);
            for arg in args {
                compile_expression(arg, chunk, slots);
            }
            chunk.push(Instruction::MethodCall(method.clone(), args.len()));
        }
        Expression::Construct { type_name, fields } => {
            let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
            for (_, e) in fields {
                compile_expression(e, chunk, slots);
            }
            chunk.push(Instruction::Construct(type_name.clone(), field_names));
        }
        Expression::Bytes(b) => {
            chunk.push(Instruction::LoadBytes(b.clone()));
        }
        Expression::List(items) => {
            for item in items {
                compile_expression(item, chunk, slots);
            }
            chunk.push(Instruction::NewList(items.len()));
        }
        Expression::Map(entries) => {
            for (key, value) in entries {
                compile_expression(key, chunk, slots);
                compile_expression(value, chunk, slots);
            }
            chunk.push(Instruction::NewMap(entries.len()));
        }
        Expression::Index { object, index } => {
            compile_expression(object, chunk, slots);
            compile_expression(index, chunk, slots);
            chunk.push(Instruction::IndexGet);
        }
        Expression::IndexSet {
            object,
            index,
            value,
        } => {
            compile_expression(object, chunk, slots);
            compile_expression(index, chunk, slots);
            compile_expression(value, chunk, slots);
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
                        compile_expression(&expr, chunk, slots);
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
