use arbores::repl::Repl;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("arbores")
        .version("0.1.0")
        .author("weiwei")
        .about("Arbores Scheme Interpreter")
        .arg(
            Arg::new("file")
                .help("Scheme file to execute")
                .value_name("FILE")
                .index(1)
        )
        .get_matches();

    // 如果指定了文件，执行文件
    if let Some(file_path) = matches.get_one::<String>("file") {
        execute_file(file_path);
        return;
    }

    // 启动 REPL
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
