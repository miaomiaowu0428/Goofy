#![allow(unused)]
#![allow(non_snake_case)]

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command as stdCommand;
use std::ptr::eq;

use clap::{arg, ArgMatches, Command as clapCommand, Parser as clapParser};
use colored::Colorize;
use inkwell::context::Context;
use Goofy_toml::Package;

use Qmmc;
use Qmmc::analyze::lex::Lexer;
use Qmmc::analyze::parse::Parser;
use Qmmc::compile::Compiler;
use Qmmc::IR_building::IRBuilder;
use crate::Goofy_toml::GoofyToml;

mod Goofy_toml;

static INTRODUCTION: &str =
    "Goofy is a simple CLI tool to build your project written in Qmm-lang.\n\
    to use Goofy, you need to make sure that you have installed the LLVM and Clang on your machine.";

fn main() {
    let matches = clapCommand::new("Goofy")
        .version("version 0.0.1b")
        .author("Qmm")
        .about(INTRODUCTION)
        .long_about(INTRODUCTION)
        .subcommands([
            clapCommand::new("build")
                .version("0.0.1b")
                .author("Qmm")
                .about("Compile the source file")
                .long_about("Compile the source file")
                .arg(
                    arg!([source])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Path to the source file"),
                )
                .arg(
                    arg!([output])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Output file name"),
                ),
            clapCommand::new("run")
                .version("0.0.1b")
                .author("Qmm")
                .about("Compile and Run the source file")
                .long_about("Compile and Run the source file")
                .arg(
                    arg!([source])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Path to the source file"),
                )
                .arg(
                    arg!([output])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Output file name"),
                ),
            clapCommand::new("new")
                .version("0.0.1b")
                .author("Qmm")
                .about("Create a new project")
                .long_about("Create a new project")
                .arg(
                    arg!([name])
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Name of the project"),
                ),
            clapCommand::new("test_toml")
                .version("0.0.1b")
                .author("Qmm")
                .about("Test Goofy.toml file")
                .long_about("Test Goofy.toml file"),
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
        Some(("new", new_command)) => {
            let name = new_command.get_one::<String>("name").unwrap();
            creat_project(&name);
        }
        Some(("test_toml", _)) => {
            run_with_Goofy_toml_file();
        }
        _ => {
            println!("No subcommand provided");
        }
    }
}

fn creat_project(name: &str) {
    // 新建项目目录
    let project_dir = format!("./{}", name);
    fs::create_dir(&project_dir).expect("Failed to create project directory");

    // 新建src目录
    let src_dir = format!("{}/src", project_dir);
    fs::create_dir(src_dir).expect("Failed to create src directory");

    // 新建main.qmm文件
    let source_file = format!("{}/src/main.qmm", project_dir);
    let mut file = File::create(&source_file).expect("Failed to create source file");
    file.write_all(b"fun main() {\n    \n}\n")
        .expect("Failed to write to source file");

    // 新建Goofy.toml文件
    let mut toml_file = File::create(format!("{}/Goofy.toml", project_dir))
        .expect("Failed to create Goofy.toml file");
    toml_file
        .write_all(format!("[package]\n name = \"{}\"\n version = \"0.0.1\"\n", name).as_bytes())
        .expect("Failed to write to Goofy.toml file");
    println!("Project {} created successfully", name);
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

fn create_target_release_dir(){
    let target_dir = "target";
    let release_dir = "release";
    let target_release_dir = format!("{}/{}", target_dir, release_dir);
    if !Path::new(&target_dir).exists() {
        fs::create_dir(&target_dir).expect("Failed to create target directory");
    }
    if !Path::new(&target_release_dir).exists() {
        fs::create_dir(&target_release_dir).expect("Failed to create target/release directory");
    }
}

fn run_with_Goofy_toml_file() {
    match fs::read_to_string("Goofy.toml") {
        Ok(toml_str) => {
            match toml::from_str(&toml_str) {
                Ok(Goofy_toml) => {
                    let Goofy_toml: GoofyToml = Goofy_toml;
                    println!("Package Name: {}", Goofy_toml.package.name);
                    println!("Package Version: {}", Goofy_toml.package.version);
                    create_target_release_dir();
                    let target_path = format!("target/release/{}", Goofy_toml.package.name);
                    compile("src/main.qmm", &*target_path);
                    run(&*target_path);
                }
                Err(e) => {
                    println!(
                        "{}",
                        format!(
                            "{}:\n{} {} {} {} {} {} {} {} {}",
                            "Goofy.toml broken".red(),
                            "make sure your",
                            "Goofy.toml".green(),
                            "contains a",
                            "[package]".green(),
                            "label and it has a",
                            "name".green(),
                            "item and a",
                            "version".green(),
                            "item",
                        )
                    );
                }
            }
        },
        Err(_) => {
            println!(
                "{}",
                format!("{}{}", "run failed: ".red(), "Goofy.toml not found")
            );
        }
    }
}

fn run(target_file_name: &str) {
    let output = stdCommand::new(target_file_name)
        .output()
        .expect("Failed to execute command");
    let exit_code = output.status.code().unwrap();

    println!("\nExit Code of {}: {}\n\n", target_file_name, exit_code);
}
