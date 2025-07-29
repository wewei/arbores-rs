use std::collections::HashMap;
use crate::eval::Evaluator;
use crate::parser::Parser;
use crate::types::{SchemeError, Value};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result as RustylineResult};

/// REPL (Read-Eval-Print Loop) 实现 - 使用增强模式
pub struct Repl {
    evaluator: Evaluator,
    context: HashMap<String, Value>,
    editor: Editor<()>,
}

impl Repl {
    /// 创建新的 REPL
    pub fn new() -> RustylineResult<Self> {
        let editor = Editor::<()>::new()?;
        Ok(Repl {
            evaluator: Evaluator::new(),
            context: HashMap::new(),
            editor,
        })
    }

    /// 启动 REPL
    pub fn run(&mut self) -> RustylineResult<()> {
        println!("🌲 Arbores Scheme Interpreter v0.1.0 (Enhanced Mode)");
        println!("Type :help for help, :exit to quit, or Ctrl+D to exit.");
        println!("Features: History ✓ Line editing ✓ Multi-line ✓");
        println!();

        let mut multiline_buffer = String::new();

        loop {
            let prompt = if multiline_buffer.is_empty() {
                "arbores> "
            } else {
                "      .. "
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.trim();
                    
                    // 处理空行
                    if line.is_empty() {
                        if !multiline_buffer.is_empty() {
                            continue;
                        } else {
                            continue;
                        }
                    }
                    
                    // 处理特殊命令
                    if line.starts_with(':') {
                        if let Some(output) = self.handle_command(line) {
                            println!("{}", output);
                        }
                        continue;
                    }
                    
                    // 处理多行输入
                    if !multiline_buffer.is_empty() {
                        multiline_buffer.push(' ');
                    }
                    multiline_buffer.push_str(line);
                    
                    // 检查是否是完整的表达式
                    if self.is_complete_expression(&multiline_buffer) {
                        // 添加到历史记录
                        let _ = self.editor.add_history_entry(&multiline_buffer);
                        
                        // 求值
                        let result = self.evaluate(&multiline_buffer);
                        println!("{}", result);
                        
                        // 清空缓冲区
                        multiline_buffer.clear();
                    }
                    // 否则继续等待更多输入
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    multiline_buffer.clear();
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        Ok(())
    }

    /// 检查括号是否匹配（简单的多行输入支持）
    fn is_complete_expression(&self, input: &str) -> bool {
        let mut paren_count = 0;
        let mut in_string = false;
        let mut escaped = false;
        
        for ch in input.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            
            match ch {
                '"' => in_string = !in_string,
                '\\' if in_string => escaped = true,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => paren_count -= 1,
                _ => {}
            }
        }
        
        paren_count == 0 && !in_string
    }

    /// 求值并返回结果
    fn evaluate(&mut self, input: &str) -> String {
        match self.evaluator.eval_string(input, None) {
            Ok(value) => {
                // 如果是定义操作，更新上下文（简单检测）
                if input.trim().starts_with("(define ") {
                    // 这里应该更准确地解析定义，但为简单起见使用字符串匹配
                    if let Value::Symbol(name) = &value {
                        self.context.insert(name.clone(), value.clone());
                    }
                }
                format!("{}", value)
            }
            Err(SchemeError::SyntaxError(msg, _)) => {
                format!("Syntax Error: {}", msg)
            }
            Err(SchemeError::RuntimeError(msg, _)) => {
                format!("Runtime Error: {}", msg)
            }
            Err(e) => {
                format!("Error: {}", e)
            }
        }
    }

    /// 处理特殊命令
    fn handle_command(&mut self, command: &str) -> Option<String> {
        let command = command.trim_start_matches(':').trim();
        
        match command {
            "help" => {
                Some(
                    r#"
🌲 Arbores Scheme Interpreter Commands:
  :help         Show this help message
  :symbols      List available symbols
  :keywords     List Scheme keywords
  :clear        Clear the screen
  :reset        Reset the interpreter state
  :history      Show command history
  :exit         Exit the interpreter

Scheme Special Forms:
  (quote expr)  Return expr without evaluation
  (if test then else)  Conditional expression
  (lambda (params) body)  Create function
  (let ((var val) ...) body)  Local bindings
  (define var val)  Define variable
  (begin expr ...)  Sequential evaluation

Built-in Functions:
  Arithmetic: + - * / = < > <= >= abs max min
  Lists: cons car cdr list null? pair?
  Types: number? string? symbol?

Navigation:
  ↑/↓           Browse command history
  Ctrl+A/E      Move to beginning/end of line
  Ctrl+L        Clear screen
  Ctrl+C        Interrupt
  Ctrl+D        Exit
"#
                    .trim()
                    .to_string(),
                )
            }
            "symbols" => {
                let symbols = self.get_available_symbols();
                Some(format!("Available symbols: {}", symbols.join(", ")))
            }
            "keywords" => {
                let keywords = self.get_scheme_keywords();
                Some(format!("Scheme keywords: {}", keywords.join(", ")))
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen ANSI escape sequence
                None
            }
            "reset" => {
                self.evaluator = Evaluator::new();
                self.context.clear();
                Some("Interpreter state reset.".to_string())
            }
            "history" => {
                // rustyline 内置了历史功能，这里只是提示
                Some("Use ↑/↓ arrows to navigate command history.".to_string())
            }
            "exit" => {
                println!("Goodbye!");
                std::process::exit(0);
            }
            _ => Some(format!("Unknown command: :{}", command)),
        }
    }

    /// 获取 Scheme 关键字列表（用于自动补全）
    fn get_scheme_keywords(&self) -> Vec<&'static str> {
        vec![
            // Special forms
            "quote", "if", "lambda", "let", "begin", "and", "or", "cond", "define", "set!",
            // Built-in functions
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "abs", "max", "min",
            "cons", "car", "cdr", "list", "null?", "pair?", "number?", "string?", "symbol?",
            // Constants
            "#t", "#f", "true", "false",
        ]
    }

    /// 获取当前环境中可用的符号
    fn get_available_symbols(&self) -> Vec<String> {
        let mut symbols = Vec::new();
        
        // 添加内置关键字
        symbols.extend(self.get_scheme_keywords().iter().map(|s| s.to_string()));
        
        // 添加用户定义的变量
        symbols.extend(self.context.keys().cloned());
        
        symbols
    }

    /// 求值多个表达式
    pub fn eval_multiple(&mut self, input: &str) -> Result<Vec<crate::types::Value>, crate::types::SchemeError> {
        let expressions = Parser::parse_multiple(input)?;
        let mut results = Vec::new();
        
        for expr in expressions {
            let global_env = self.evaluator.get_global_env();
            let result = self.evaluator.eval(&expr, &global_env, None)?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// 便利方法：求值单个表达式（用于测试）
    pub fn eval(&mut self, input: &str) -> Result<crate::types::Value, crate::types::SchemeError> {
        self.evaluator.eval_string(input, None)
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // 如果无法创建增强版 REPL，这里应该有一个回退方案
            // 但为了简化，我们假设 rustyline 总是可用的
            panic!("Failed to create enhanced REPL")
        })
    }
}

/// 启动增强版 REPL 的便利函数
pub fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to initialize REPL: {}", e))?;
    
    repl.run()
        .map_err(|e| format!("REPL error: {}", e).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_repl_basic() {
        let mut repl = Repl::new().unwrap();
        
        assert_eq!(repl.eval("42").unwrap(), Value::Integer(42));
        assert_eq!(repl.eval("(+ 1 2)").unwrap(), Value::Integer(3));
        assert_eq!(repl.eval("'hello").unwrap(), Value::Symbol("hello".to_string()));
    }

    #[test]
    fn test_repl_multiple() {
        let mut repl = Repl::new().unwrap();
        
        let results = repl.eval_multiple("1 2 (+ 3 4)").unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Value::Integer(1));
        assert_eq!(results[1], Value::Integer(2));
        assert_eq!(results[2], Value::Integer(7));
    }
}
