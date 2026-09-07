mod ast;
mod lexer;
mod lixvm;
mod parser;
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

    let tokens = lexer::token::lex(&source);
    let statements = parser::parse(&tokens);
    lixvm::interpreter::run(&statements);
}

fn run_lixvm(args: &[String]) {
    if !args.is_empty() && args[0] == "--version" {
        println!("LixVM {}", version::LIXVM);
        return;
    }
    eprintln!(
        "LixVM {}: standalone mode not yet implemented",
        version::LIXVM
    );
    eprintln!("usage: julix <file.jlx>");
}

fn run_jujit(args: &[String]) {
    if !args.is_empty() && args[0] == "--version" {
        println!("JuJIT {}", version::JUJIT);
        return;
    }
    eprintln!("JuJIT {}: not yet built (phase 2)", version::JUJIT);
}
