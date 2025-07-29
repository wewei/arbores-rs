use std::io::{self, Write};
use crate::eval::Evaluator;
use crate::parser::Parser;

/// REPL (Read-Eval-Print Loop) 实现
pub struct Repl {
    evaluator: Evaluator,
}

impl Repl {
    /// 创建新的 REPL
    pub fn new() -> Self {
        Repl {
            evaluator: Evaluator::new(),
        }
    }

    /// 启动 REPL
    pub fn run(&mut self) {
        // 检查是否在交互式终端中运行
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            self.run_interactive();
        } else {
            self.run_batch();
        }
    }

    /// 交互式模式
    fn run_interactive(&mut self) {
        println!("Arbores Scheme Interpreter v0.1.0");
        println!("Type 'exit' or press Ctrl+C to quit.");
        println!();

        loop {
            // 显示提示符
            print!("arbores> ");
            io::stdout().flush().unwrap();

            // 读取用户输入
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => {
                    // EOF reached
                    println!("\nGoodbye!");
                    break;
                },
                Ok(_) => {
                    let input = input.trim();
                    
                    // 检查退出命令
                    if input.is_empty() {
                        continue;
                    }
                    
                    if input == "exit" || input == "quit" {
                        println!("Goodbye!");
                        break;
                    }

                    // 求值并打印结果
                    self.eval_and_print(input);
                },
                Err(error) => {
                    eprintln!("Error reading input: {error}");
                    break;
                }
            }
        }
    }

    /// 批处理模式（用于管道输入）
    fn run_batch(&mut self) {
        let mut input = String::new();
        
        loop {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => {
                    // EOF reached
                    break;
                },
                Ok(_) => {
                    input.push_str(&line);
                },
                Err(error) => {
                    eprintln!("Error reading input: {error}");
                    std::process::exit(1);
                }
            }
        }
        
        if !input.trim().is_empty() {
            // 按行分割并处理每一行
            for line in input.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                // 检查退出命令
                if line == "exit" || line == "quit" {
                    break;
                }
                
                // 求值并打印结果
                match self.evaluator.eval_string(line) {
                    Ok(result) => println!("{result}"),
                    Err(error) => {
                        eprintln!("Error: {error}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    /// 求值并打印结果
    fn eval_and_print(&mut self, input: &str) {
        match self.evaluate_input(input) {
            Ok(result) => println!("{result}"),
            Err(error) => eprintln!("Error: {error}"),
        }
    }

    /// 求值输入
    fn evaluate_input(&mut self, input: &str) -> Result<crate::types::Value, crate::types::SchemeError> {
        // 求值表达式
        self.evaluator.eval_string(input)
    }

    /// 求值多个表达式
    pub fn eval_multiple(&mut self, input: &str) -> Result<Vec<crate::types::Value>, crate::types::SchemeError> {
        let expressions = Parser::parse_multiple(input)?;
        let mut results = Vec::new();
        
        for expr in expressions {
            let global_env = self.evaluator.get_global_env();
            let result = self.evaluator.eval(&expr, &global_env)?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// 便利方法：求值单个表达式（用于测试）
    pub fn eval(&mut self, input: &str) -> Result<crate::types::Value, crate::types::SchemeError> {
        self.evaluator.eval_string(input)
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_repl_basic() {
        let mut repl = Repl::new();
        
        assert_eq!(repl.eval("42").unwrap(), Value::Integer(42));
        assert_eq!(repl.eval("(+ 1 2)").unwrap(), Value::Integer(3));
        assert_eq!(repl.eval("'hello").unwrap(), Value::Symbol("hello".to_string()));
    }

    #[test]
    fn test_repl_multiple() {
        let mut repl = Repl::new();
        
        let results = repl.eval_multiple("1 2 (+ 3 4)").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Value::Integer(1));
        assert_eq!(results[1], Value::Integer(2));
        assert_eq!(results[2], Value::Integer(7));
    }
}
