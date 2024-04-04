#![allow(unused_imports)]
#![allow(unused)]
#![allow(non_snake_case)]

use std::{env, fs};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::ptr::read;

use inkwell::context::Context;
use colored::Colorize;

use Qmmc;
use Qmmc::analyze::lex::Lexer;
use Qmmc::analyze::parse::{Expression, Parser};
use Qmmc::compile::{CheckedExpression, Compiler};
use Qmmc::IR_building::IRBuilder;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("Usage: {} [build|run] <file_name> <target>", args[0]);
        return;
    }
    let command = &args[1];
    let FILE_NAME = &args[2];
    let RES_PATH = &args[3];


    let mut file = File::open(Path::new(FILE_NAME))
        .expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");
    let lexer = Lexer::new(&contents);
    let tokens = lexer.lex();

    // println!("Tokens: {:#?}", tokens);

    let syntax_tree = Parser::new(tokens);
    let expressions = syntax_tree.parse();
    if !syntax_tree.diagnostics.is_empty() {
        println!("Syntax Errors: ");
        syntax_tree.diagnostics.print();
        println!("==============================");
    }

    // show_input(&expressions);

    compile(command, expressions, FILE_NAME, RES_PATH);
}

fn compile(command: &str, expressions: Vec<Expression>, source_file_name: &str, res_file_name: &str) {
    let compiler = Compiler::new();
    let checked_expressions = compiler.analyse(expressions);

    if !compiler.diagnostics.is_empty() {
        println!("Static Analysis Diagnostics: ");
        compiler.diagnostics.print();
        println!("==============================");
    } else {
        let context = Context::create();
        let module = context.create_module(source_file_name);
        let builder = context.create_builder();
        let ir_builder = IRBuilder::new(&context, module, builder);

        // ir_builder.import_lib_func();

        ir_builder.build_irs(checked_expressions);
        if ir_builder.diagnostics.is_empty() == false {
            println!("Build Failed Due TO: ");
            ir_builder.diagnostics.print();
            println!("==============================");
        } else {
            let llc_source = format!("{}{}", res_file_name, ".ll");
            ir_builder.save_as(&llc_source);

            let llc_output = Command::new("llc")
                .arg(llc_source.clone())
                .output()
                .expect("Failed to execute llc command");

            let clang_source = format!("{}{}", res_file_name, ".s");

            if llc_output.status.success() {
                println!(
                    "{}",
                    format!("{:<26}: {}", "successfully compiled to", clang_source).green()
                );
            } else {
                eprintln!(
                    "llc command failed: {}",
                    String::from_utf8_lossy(&llc_output.stderr)
                );
            }

            let clang_output = Command::new("clang")
                .arg(clang_source.clone())
                .arg("-o")
                .arg(res_file_name)
                .output()
                .expect("Failed to execute clang command");

            if clang_output.status.success() {
                println!(
                    "{}",
                    format!("{:<26}: {}", "successfully compiled to", res_file_name).green()
                );
            } else {
                eprintln!(
                    "clang command failed: {}",
                    String::from_utf8_lossy(&clang_output.stderr)
                );
            }

            if clang_output.status.success() {
                if let Err(e) = fs::remove_file(&llc_source) {
                    eprintln!("Error removing file {}: {}", llc_source, e);
                }
                if let Err(e) = fs::remove_file(&clang_source) {
                    eprintln!("Error removing file {}: {}", clang_source, e);
                }
            } else {
                println!("Failed to compile to {}", res_file_name);
                return;
            }

            if command == "run" {
                println!("\n\nRunning: {}", res_file_name);
                let output = Command::new(res_file_name)
                    .output()
                    .expect("Failed to execute command");
                let exit_code = output.status.code().unwrap();

                println!("Exit Code of {}(): {}\n\n", res_file_name, exit_code);
            }
        }
    }
}

fn show_input(expressions: &Vec<Expression>) {
    println!("Input Expressions: ");
    for expression in expressions {
        println!("{}", expression)
    }
    println!("==============================");
}

fn show_static_scope(static_analyzer: &Compiler) {
    println!("\n\nScope:\n{:#?}", static_analyzer.scope);
}

fn show_ByteCode(checked_expressions: &Vec<CheckedExpression>) {
    println!("\n\nByteCode:");
    for expression in checked_expressions {
        println!("{:#?}", expression);
    }
}