use crate::types::{Value, SchemeError, Result};
use crate::storage::{MemoryStorage, SExpressionStorage, StoredSExpression, SExpressionId};
use crate::eval::Evaluator;
use std::rc::Rc;
use std::cell::RefCell;

/// Arbores 知识库系统的主实例
pub struct Arbores {
    /// S-Expression 存储
    storage: RefCell<MemoryStorage>,
    /// Scheme 求值器
    evaluator: Evaluator,
}

impl Arbores {
    /// 创建新的 Arbores 实例
    pub fn new() -> Self {
        Arbores {
            storage: RefCell::new(MemoryStorage::new()),
            evaluator: Evaluator::new(),
        }
    }

    /// arb:create - 创建新的 S-Expression
    pub fn create(
        &self,
        code_str: &str,
        dependencies: Vec<SExpressionId>,
        description: Option<String>,
        type_desc: Option<String>,
        symbol_names: Vec<String>,
    ) -> Result<SExpressionId> {
        // 解析代码
        let code = crate::parse(code_str)
            .map_err(|e| SchemeError::RuntimeError(format!("Failed to parse code: {}", e)))?;

        // 创建 StoredSExpression
        let stored_expr = StoredSExpression::new(
            0, // 将被自动分配
            Rc::new(code),
            description,
            type_desc,
            symbol_names,
            dependencies,
        );

        // 存储到知识库
        self.storage.borrow_mut().store(stored_expr)
            .map_err(|e| SchemeError::RuntimeError(format!("Storage error: {}", e)))
    }

    /// arb:get-metadata - 根据 ID 查询 S-Expression 的元数据
    pub fn get_metadata(&self, id: SExpressionId) -> Result<Value> {
        let expr = self.storage.borrow().get(id)
            .map_err(|e| SchemeError::RuntimeError(format!("Storage error: {}", e)))?
            .ok_or_else(|| SchemeError::RuntimeError(format!("S-Expression with ID {} not found", id)))?;

        // 构造元数据的 association list 格式
        let mut metadata = Vec::new();

        // ("id" . id)
        metadata.push(Value::Cons(
            Rc::new(Value::String("id".to_string())),
            Rc::new(Value::Integer(id as i64)),
        ));

        // ("description" . description)
        if let Some(desc) = &expr.description {
            metadata.push(Value::Cons(
                Rc::new(Value::String("description".to_string())),
                Rc::new(Value::String(desc.clone())),
            ));
        }

        // ("type" . type)
        if let Some(type_desc) = &expr.type_description {
            metadata.push(Value::Cons(
                Rc::new(Value::String("type".to_string())),
                Rc::new(Value::String(type_desc.clone())),
            ));
        }

        // ("symbol-names" . (list of symbols))
        if !expr.symbol_names.is_empty() {
            let symbols: Vec<Value> = expr.symbol_names.iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            let symbols_list = Self::vec_to_list(symbols);
            metadata.push(Value::Cons(
                Rc::new(Value::String("symbol-names".to_string())),
                Rc::new(symbols_list),
            ));
        }

        // ("dependencies" . (list of ids))
        if !expr.dependencies.is_empty() {
            let deps: Vec<Value> = expr.dependencies.iter()
                .map(|&id| Value::Integer(id as i64))
                .collect();
            let deps_list = Self::vec_to_list(deps);
            metadata.push(Value::Cons(
                Rc::new(Value::String("dependencies".to_string())),
                Rc::new(deps_list),
            ));
        }

        // ("code" . code)
        metadata.push(Value::Cons(
            Rc::new(Value::String("code".to_string())),
            expr.code,
        ));

        Ok(Self::vec_to_list(metadata))
    }

    /// arb:get-dependencies - 根据 ID 查询 S-Expression 依赖的 S-Expression ID 列表
    pub fn get_dependencies(&self, id: SExpressionId) -> Result<Value> {
        let expr = self.storage.borrow().get(id)
            .map_err(|e| SchemeError::RuntimeError(format!("Storage error: {}", e)))?
            .ok_or_else(|| SchemeError::RuntimeError(format!("S-Expression with ID {} not found", id)))?;

        let deps: Vec<Value> = expr.dependencies.iter()
            .map(|&id| Value::Integer(id as i64))
            .collect();

        Ok(Self::vec_to_list(deps))
    }

    /// arb:search-by-symbol - 根据 symbol name 查询 S-Expressions
    pub fn search_by_symbol(&self, pattern: &str, match_mode: Option<&str>) -> Result<Value> {
        let mode = match_mode.unwrap_or("prefix");

        let found_ids = match mode {
            "exact" => {
                self.storage.borrow().find_by_symbol(pattern)
                    .map_err(|e| SchemeError::RuntimeError(format!("Storage error: {}", e)))?
            }
            "prefix" => {
                self.find_by_prefix(pattern)?
            }
            _ => {
                return Err(SchemeError::RuntimeError(format!("Unsupported match mode: {}", mode)));
            }
        };

        // 构造结果列表
        let mut results = Vec::new();
        for id in found_ids {
            if let Ok(Some(expr)) = self.storage.borrow().get(id) {
                let mut result_entry = Vec::new();

                // ("id" . id)
                result_entry.push(Value::Cons(
                    Rc::new(Value::String("id".to_string())),
                    Rc::new(Value::Integer(id as i64)),
                ));

                // ("symbol-names" . (list of symbols))
                if !expr.symbol_names.is_empty() {
                    let symbols: Vec<Value> = expr.symbol_names.iter()
                        .map(|s| Value::String(s.clone()))
                        .collect();
                    let symbols_list = Self::vec_to_list(symbols);
                    result_entry.push(Value::Cons(
                        Rc::new(Value::String("symbol-names".to_string())),
                        Rc::new(symbols_list),
                    ));
                }

                // ("description" . description)
                if let Some(desc) = &expr.description {
                    result_entry.push(Value::Cons(
                        Rc::new(Value::String("description".to_string())),
                        Rc::new(Value::String(desc.clone())),
                    ));
                }

                results.push(Self::vec_to_list(result_entry));
            }
        }

        Ok(Self::vec_to_list(results))
    }

    /// arb:semantic-search - 基础语义搜索（暂时简化为描述文本匹配）
    pub fn semantic_search(&self, query: &str) -> Result<Value> {
        let all_ids = self.storage.borrow().list_ids()
            .map_err(|e| SchemeError::RuntimeError(format!("Storage error: {}", e)))?;

        let mut results = Vec::new();
        
        for id in all_ids {
            if let Ok(Some(expr)) = self.storage.borrow().get(id) {
                if let Some(desc) = &expr.description {
                    // 简单的文本匹配（后续可以替换为更复杂的语义搜索）
                    if desc.to_lowercase().contains(&query.to_lowercase()) {
                        let mut result_entry = Vec::new();

                        // ("id" . id)
                        result_entry.push(Value::Cons(
                            Rc::new(Value::String("id".to_string())),
                            Rc::new(Value::Integer(id as i64)),
                        ));

                        // ("score" . score)
                        result_entry.push(Value::Cons(
                            Rc::new(Value::String("score".to_string())),
                            Rc::new(Value::Float(0.8)),
                        ));

                        // ("description" . description)
                        result_entry.push(Value::Cons(
                            Rc::new(Value::String("description".to_string())),
                            Rc::new(Value::String(desc.clone())),
                        ));

                        results.push(Self::vec_to_list(result_entry));
                    }
                }
            }
        }

        Ok(Self::vec_to_list(results))
    }

    /// 求值 Scheme 表达式（用于测试和演示）
    pub fn eval(&self, input: &str) -> Result<Value> {
        self.evaluator.eval_string(input)
    }

    /// 前缀匹配的辅助方法
    fn find_by_prefix(&self, prefix: &str) -> Result<Vec<SExpressionId>> {
        let all_ids = self.storage.borrow().list_ids()
            .map_err(|e| SchemeError::RuntimeError(format!("Storage error: {}", e)))?;

        let mut matching_ids = Vec::new();

        for id in all_ids {
            if let Ok(Some(expr)) = self.storage.borrow().get(id) {
                for symbol in &expr.symbol_names {
                    if symbol.starts_with(prefix) {
                        matching_ids.push(id);
                        break; // 找到一个匹配就足够了
                    }
                }
            }
        }

        Ok(matching_ids)
    }

    /// 将 Vec<Value> 转换为 Scheme 列表
    fn vec_to_list(items: Vec<Value>) -> Value {
        items.into_iter().rev().fold(Value::Nil, |acc, item| {
            Value::Cons(Rc::new(item), Rc::new(acc))
        })
    }
}

impl Default for Arbores {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbores_basic_operations() {
        let arbores = Arbores::new();

        // 测试创建 S-Expression
        let id = arbores.create(
            "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))",
            vec![],
            Some("计算阶乘的递归函数".to_string()),
            Some("function".to_string()),
            vec!["factorial".to_string(), "fact".to_string()],
        ).unwrap();

        assert_eq!(id, 1);

        // 测试获取元数据
        let metadata = arbores.get_metadata(id).unwrap();
        assert!(matches!(metadata, Value::Cons(_, _)));

        // 测试依赖查询
        let deps = arbores.get_dependencies(id).unwrap();
        assert!(matches!(deps, Value::Nil)); // 没有依赖

        // 测试符号搜索
        let search_results = arbores.search_by_symbol("fact", Some("prefix")).unwrap();
        assert!(matches!(search_results, Value::Cons(_, _)));

        // 测试语义搜索
        let semantic_results = arbores.semantic_search("阶乘").unwrap();
        assert!(matches!(semantic_results, Value::Cons(_, _)));
    }

    #[test]
    fn test_arbores_symbol_search_modes() {
        let arbores = Arbores::new();

        // 创建测试数据
        arbores.create(
            "(define (test-func x) x)",
            vec![],
            Some("Test function".to_string()),
            Some("function".to_string()),
            vec!["test-func".to_string()],
        ).unwrap();

        arbores.create(
            "(define (another-test y) y)",
            vec![],
            Some("Another test".to_string()),
            Some("function".to_string()),
            vec!["another-test".to_string()],
        ).unwrap();

        // 测试前缀匹配
        let prefix_results = arbores.search_by_symbol("test", Some("prefix")).unwrap();
        assert!(matches!(prefix_results, Value::Cons(_, _)));

        // 测试精确匹配
        let exact_results = arbores.search_by_symbol("test-func", Some("exact")).unwrap();
        assert!(matches!(exact_results, Value::Cons(_, _)));
    }

    #[test]
    fn test_arbores_with_dependencies() {
        let arbores = Arbores::new();

        // 创建基础函数
        let helper_id = arbores.create(
            "(define (add x y) (+ x y))",
            vec![],
            Some("加法辅助函数".to_string()),
            Some("function".to_string()),
            vec!["add".to_string()],
        ).unwrap();

        // 创建依赖函数
        let main_id = arbores.create(
            "(define (add-one x) (add x 1))",
            vec![helper_id],
            Some("加一函数".to_string()),
            Some("function".to_string()),
            vec!["add-one".to_string()],
        ).unwrap();

        // 测试依赖查询
        let deps = arbores.get_dependencies(main_id).unwrap();
        // 应该返回包含 helper_id 的列表
        assert!(matches!(deps, Value::Cons(_, _)));
    }
}
