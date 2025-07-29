use crate::types::Position;

/// 调用栈帧
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: Option<String>,
    pub position: Position,
    pub expression: String,
}

/// 链式不可变执行上下文
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub current_position: Option<Position>,
    pub function_name: Option<String>,
    pub parent: Option<Box<EvaluationContext>>, // 链式结构
}

impl EvaluationContext {
    /// 创建根上下文
    pub fn new() -> Self {
        Self {
            current_position: None,
            function_name: None,
            parent: None,
        }
    }
    
    /// 进入新的调用层级，返回新的上下文
    pub fn enter_call(&self, position: Option<Position>, function_name: Option<String>) -> Self {
        Self {
            current_position: position,
            function_name,
            parent: Some(Box::new(self.clone())),
        }
    }
    
    /// 获取完整调用栈
    pub fn call_stack(&self) -> Vec<CallFrame> {
        let mut stack = Vec::new();
        let mut current = Some(self);
        
        while let Some(ctx) = current {
            if let Some(pos) = ctx.current_position {
                stack.push(CallFrame {
                    function_name: ctx.function_name.clone(),
                    position: pos,
                    expression: String::new(),
                });
            }
            current = ctx.parent.as_ref().map(|p| p.as_ref());
        }
        
        stack.reverse(); // 从最顶层开始
        stack
    }
    
    /// 格式化调用栈为字符串
    pub fn format_call_stack(&self) -> String {
        let stack = self.call_stack();
        if stack.is_empty() {
            return String::new();
        }
        
        let mut result = String::from("Call stack:\n");
        for (i, frame) in stack.iter().enumerate() {
            result.push_str(&format!(
                "  {}. {} at {}\n",
                i + 1,
                frame.function_name.as_deref().unwrap_or("<anonymous>"),
                frame.position
            ));
        }
        result
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Position;

    #[test]
    fn test_evaluation_context_new() {
        let ctx = EvaluationContext::new();
        assert!(ctx.current_position.is_none());
        assert!(ctx.function_name.is_none());
        assert!(ctx.parent.is_none());
    }

    #[test]
    fn test_enter_call() {
        let root = EvaluationContext::new();
        let pos = Position::new(1, 1);
        let child = root.enter_call(Some(pos), Some("test".to_string()));
        
        assert_eq!(child.current_position, Some(pos));
        assert_eq!(child.function_name, Some("test".to_string()));
        assert!(child.parent.is_some());
    }

    #[test]
    fn test_call_stack() {
        let root = EvaluationContext::new();
        let child1 = root.enter_call(Some(Position::new(1, 1)), Some("func1".to_string()));
        let child2 = child1.enter_call(Some(Position::new(2, 5)), Some("func2".to_string()));
        
        let stack = child2.call_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].function_name, Some("func1".to_string()));
        assert_eq!(stack[1].function_name, Some("func2".to_string()));
    }

    #[test]
    fn test_format_call_stack() {
        let root = EvaluationContext::new();
        let child = root.enter_call(Some(Position::new(1, 1)), Some("test".to_string()));
        
        let formatted = child.format_call_stack();
        assert!(formatted.contains("Call stack:"));
        assert!(formatted.contains("test"));
        assert!(formatted.contains("line 1, column 1"));
    }
}
