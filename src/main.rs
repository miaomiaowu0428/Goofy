#![allow(unused)]
#![allow(non_snake_case)]

use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command as stdCommand;

use clap::{arg, ArgMatches, Command as clapCommand, Parser as clapParser};
use colored::Colorize;
use inkwell::context::Context;

use Qmmc;
use Qmmc::analyze::lex::Lexer;
use Qmmc::analyze::parse::Parser;
use Qmmc::compile::Compiler;
use Qmmc::IR_building::IRBuilder;

static INTRODUCTION: &str =
    "Goofy is a simple CLI tool to build your project written in Qmm language";

fn main() {
    let matches = clapCommand::new("Goofy")
        .version("version 0.0.1b")
        .author("Qmm")
        .about(INTRODUCTION)
        .long_about(INTRODUCTION)
        .subcommands([
            clapCommand::new("build").about("Compile the source file")
                .arg(
                    arg!([source])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Path to the source file to compile or run"),
                )
                .arg(
                    arg!([output])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Output file name"),
                ),
            clapCommand::new("run").about("Compile and Run the source file")
                .arg(
                    arg!([source])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Path to the source file to compile or run"),
                )
                .arg(
                    arg!([output])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Output file name"),
                ),
        ])

        .get_matches();
    match matches.subcommand() {
        Some(("build", build_command)) => {
            let source = build_command.get_one::<String>("source").unwrap();
            let output = build_command.get_one::<String>("output").unwrap();
            compile(&source, &output);
        }
        Some(("run", run_command)) => {
            let source = run_command.get_one::<String>("source").unwrap();
            let output = run_command.get_one::<String>("output").unwrap();
            compile(&source, &output);
            run(&output);
        }
        _ => {
            println!("No subcommand provided");
        }
    }
}

fn compile(source_file_name: &str, res_file_name: &str) {
    let mut file = File::open(Path::new(source_file_name)).expect("Could not open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Could not read file");
    let lexer = Lexer::new(&contents);
    let tokens = lexer.lex();

    let syntax_tree = Parser::new(tokens);
    let expressions = syntax_tree.parse();
    if !syntax_tree.diagnostics.is_empty() {
        println!("Syntax Errors: ");
        syntax_tree.diagnostics.print();
        println!("==============================");
        return;
    }

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

            let llc_output = stdCommand::new("llc")
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

            let clang_output = stdCommand::new("clang")
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
        }
    }
}

fn run(target_file_name: &str) {
    let output = stdCommand::new(target_file_name)
        .output()
        .expect("Failed to execute command");
    let exit_code = output.status.code().unwrap();

    println!("Exit Code of {}(): {}\n\n", target_file_name, exit_code);
}
