use arbores::Repl;
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("arbores")
        .version("0.1.0")
        .author("weiwei")
        .about("Arbores Scheme Interpreter")
        .arg(
            Arg::new("repl-mode")
                .long("repl")
                .value_name("MODE")
                .help("REPL mode to use")
                .value_parser(["simple", "enhanced"])
                .default_value("enhanced")
        )
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

    // 否则启动 REPL
    let repl_mode = matches.get_one::<String>("repl-mode").unwrap();
    
    match repl_mode.as_str() {
        "enhanced" => {
            // 尝试启动增强版 REPL
            if let Err(e) = arbores::repl::enhanced::run_enhanced_repl() {
                eprintln!("Enhanced REPL failed to start: {}", e);
                eprintln!("Falling back to simple REPL...");
                let mut repl = Repl::new();
                repl.run();
            }
        }
        "simple" => {
            let mut repl = Repl::new();
            repl.run();
        }
        _ => {
            eprintln!("Unknown REPL mode: {}", repl_mode);
            std::process::exit(1);
        }
    }
}

fn execute_file(file_path: &str) {
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            let mut repl = Repl::new();
            match repl.eval(&content) {
                Ok(result) => println!("{}", result),
                Err(e) => {
                    eprintln!("Error executing file: {}", e);
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
