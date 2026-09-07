#![allow(clippy::while_let_loop)]
#![allow(dead_code)]

mod bytecode;
mod lixer;
mod lixvm;
mod version;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let prog = env::args().next().unwrap_or_default();
    let bin_name = Path::new(&prog)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("julix");

    let args: Vec<String> = env::args().skip(1).collect();

    match bin_name {
        "lixvm" | "lixvm.exe" => run_lixvm(&args),
        "jujit" | "jujit.exe" => run_jujit(&args),
        _ => run_julix(&args),
    }
}

fn run_julix(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: julix <file.jlx>");
        eprintln!("       julix compile <file.jlx>");
        process::exit(1);
    }

    let first = &args[0];
    match first.as_str() {
        "--version" => {
            println!("Julix {}", version::JULIX);
            println!("LixVM {}", version::LIXVM);
            println!("JuJIT {}", version::JUJIT);
            return;
        }
        "--julix-version" => {
            println!("Julix {}", version::JULIX);
            return;
        }
        "--lixvm-version" => {
            println!("LixVM {}", version::LIXVM);
            return;
        }
        "--jujit-version" => {
            println!("JuJIT {}", version::JUJIT);
            return;
        }
        "compile" => {
            if args.len() < 2 {
                eprintln!("usage: julix compile <file.jlx>");
                process::exit(1);
            }
            compile_file(&args[1]);
            return;
        }
        _ => {}
    }

    let path = first;
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", path, e);
            process::exit(1);
        }
    };

    let tokens = lixer::lexer::token::lex(&source);
    let statements = lixer::parser::parse(&tokens);
    let (chunk, functions) = lixer::compiler::emitter::compile(&statements);
    let mut machine = lixvm::machine::Machine::new(chunk, functions);
    machine.run();
}

fn compile_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", path, e);
            process::exit(1);
        }
    };

    let tokens = lixer::lexer::token::lex(&source);
    let statements = lixer::parser::parse(&tokens);
    let (chunk, _functions) = lixer::compiler::emitter::compile(&statements);

    let out_path = Path::new(path).with_extension("jlxr");
    match bytecode::chunk::serialize(&chunk, &out_path) {
        Ok(()) => {
            println!("compiled {} -> {}", path, out_path.display());
        }
        Err(e) => {
            eprintln!("error: cannot write {}: {}", out_path.display(), e);
            process::exit(1);
        }
    }
}

fn run_lixvm(args: &[String]) {
    if !args.is_empty() && args[0] == "--version" {
        println!("LixVM {}", version::LIXVM);
        return;
    }
    if args.is_empty() {
        eprintln!("LixVM {}", version::LIXVM);
        eprintln!("usage: lixvm <file.jlxr>");
        process::exit(1);
    }
    let chunk = match bytecode::chunk::deserialize(&args[0]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot load {}: {}", args[0], e);
            process::exit(1);
        }
    };
    let functions = bytecode::chunk::FunctionTable::new();
    let mut machine = lixvm::machine::Machine::new(chunk, functions);
    machine.run();
}

fn run_jujit(args: &[String]) {
    if !args.is_empty() && args[0] == "--version" {
        println!("JuJIT {}", version::JUJIT);
        return;
    }
    eprintln!("JuJIT {}: not yet built (phase 2)", version::JUJIT);
}
