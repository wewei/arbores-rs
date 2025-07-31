use arbores::legacy::repl::Repl;
use clap::{Arg, Command};
use std::io::{self, Read};

fn main() {
    let matches = Command::new("arbores")
        .version("0.1.0")
        .author("weiwei")
        .about("Arbores Scheme Interpreter")
        .after_help("Examples:
  arbores                           Start interactive REPL
  arbores script.scm                Execute Scheme file
  arbores -e '(+ 1 2 3)'            Evaluate expression
  echo '(* 4 5)' | arbores --       Read from stdin")
        .arg(
            Arg::new("file")
                .help("Scheme file to execute (use \"--\" to read from stdin)")
                .value_name("FILE")
                .index(1)
        )
        .arg(
            Arg::new("eval")
                .short('e')
                .long("eval")
                .value_name("EXPRESSION")
                .help("Evaluate expression and exit")
                .action(clap::ArgAction::Set)
        )
        .get_matches();

    // 如果指定了 -e 参数，求值表达式并退出
    if let Some(expression) = matches.get_one::<String>("eval") {
        execute_expression(expression);
        return;
    }

    // 如果指定了文件，执行文件
    if let Some(file_path) = matches.get_one::<String>("file") {
        execute_file(file_path);
        return;
    }

    // 检查 stdin 是否有数据
    if !atty::is(atty::Stream::Stdin) {
        // 从 stdin 读取并执行
        execute_stdin();
        return;
    }

    // 启动交互式 REPL
    start_interactive_repl();
}

fn start_interactive_repl() {
    match Repl::new() {
        Ok(mut repl) => {
            if let Err(e) = repl.run() {
                eprintln!("REPL error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize REPL: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_expression(expression: &str) {
    match Repl::new() {
        Ok(mut repl) => {
            match repl.eval_multiple(expression) {
                Ok(results) => {
                    for result in results {
                        println!("{}", result);
                    }
                },
                Err(e) => {
                    eprintln!("Error evaluating expression: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize REPL: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_stdin() {
    let mut stdin = io::stdin();
    let mut input = String::new();
    
    match stdin.read_to_string(&mut input) {
        Ok(_) => {
            if input.trim().is_empty() {
                return;
            }
            
            match Repl::new() {
                Ok(mut repl) => {
                    match repl.eval_multiple(&input) {
                        Ok(results) => {
                            for result in results {
                                println!("{}", result);
                            }
                        },
                        Err(e) => {
                            eprintln!("Error executing stdin: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to initialize REPL: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_file(file_path: &str) {
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            match Repl::new() {
                Ok(mut repl) => {
                    match repl.eval_multiple(&content) {
                        Ok(results) => {
                            for result in results {
                                println!("{}", result);
                            }
                        },
                        Err(e) => {
                            eprintln!("Error executing file: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to initialize REPL: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    }
}
