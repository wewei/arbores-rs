use crate::legacy::types::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// S-Expression 的全局唯一 ID
pub type SExpressionId = u64;

/// S-Expression 在知识库中的存储结构
#[derive(Debug, Clone)]
pub struct StoredSExpression {
    /// 全局唯一 ID
    pub id: SExpressionId,
    /// S-Expression 的代码内容
    pub code: Rc<Value>,
    /// 可选的语义描述
    pub description: Option<String>,
    /// 可选的类型描述
    pub type_description: Option<String>,
    /// 建议的 symbol names
    pub symbol_names: Vec<String>,
    /// 依赖的其他 S-Expression ID 列表
    pub dependencies: Vec<SExpressionId>,
    /// 创建时间戳
    pub created_at: u64,
    /// 最后修改时间戳
    pub modified_at: u64,
}

impl StoredSExpression {
    /// 创建新的 StoredSExpression
    pub fn new(
        id: SExpressionId,
        code: Rc<Value>,
        description: Option<String>,
        type_description: Option<String>,
        symbol_names: Vec<String>,
        dependencies: Vec<SExpressionId>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        StoredSExpression {
            id,
            code,
            description,
            type_description,
            symbol_names,
            dependencies,
            created_at: now,
            modified_at: now,
        }
    }

    /// 更新 S-Expression（保持 ID 和创建时间不变）
    pub fn update(
        &self,
        code: Option<Rc<Value>>,
        description: Option<Option<String>>,
        type_description: Option<Option<String>>,
        symbol_names: Option<Vec<String>>,
        dependencies: Option<Vec<SExpressionId>>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        StoredSExpression {
            id: self.id,
            code: code.unwrap_or_else(|| self.code.clone()),
            description: description.unwrap_or_else(|| self.description.clone()),
            type_description: type_description.unwrap_or_else(|| self.type_description.clone()),
            symbol_names: symbol_names.unwrap_or_else(|| self.symbol_names.clone()),
            dependencies: dependencies.unwrap_or_else(|| self.dependencies.clone()),
            created_at: self.created_at,
            modified_at: now,
        }
    }
}

/// S-Expression 存储的核心接口
pub trait SExpressionStorage {
    /// 存储新的 S-Expression，返回分配的 ID
    fn store(&mut self, expr: StoredSExpression) -> Result<SExpressionId, StorageError>;

    /// 根据 ID 获取 S-Expression
    fn get(&self, id: SExpressionId) -> Result<Option<StoredSExpression>, StorageError>;

    /// 更新已存在的 S-Expression
    fn update(&mut self, id: SExpressionId, expr: StoredSExpression) -> Result<(), StorageError>;

    /// 删除 S-Expression
    fn delete(&mut self, id: SExpressionId) -> Result<(), StorageError>;

    /// 获取所有 S-Expression ID
    fn list_ids(&self) -> Result<Vec<SExpressionId>, StorageError>;

    /// 根据 symbol name 查找 S-Expression
    fn find_by_symbol(&self, symbol: &str) -> Result<Vec<SExpressionId>, StorageError>;

    /// 根据依赖关系查找 S-Expression
    fn find_dependents(&self, id: SExpressionId) -> Result<Vec<SExpressionId>, StorageError>;
}

/// 存储错误类型
#[derive(Debug, Clone)]
pub enum StorageError {
    NotFound(SExpressionId),
    AlreadyExists(SExpressionId),
    InvalidData(String),
    IoError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound(id) => write!(f, "S-Expression with ID {id} not found"),
            StorageError::AlreadyExists(id) => write!(f, "S-Expression with ID {id} already exists"),
            StorageError::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            StorageError::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

/// 简单的内存存储实现
#[derive(Debug, Default)]
pub struct MemoryStorage {
    /// 存储映射：ID -> S-Expression
    expressions: HashMap<SExpressionId, StoredSExpression>,
    /// Symbol name 索引：symbol -> 包含该 symbol 的 S-Expression ID 列表
    symbol_index: HashMap<String, Vec<SExpressionId>>,
    /// 依赖索引：被依赖的 ID -> 依赖它的 S-Expression ID 列表
    dependents_index: HashMap<SExpressionId, Vec<SExpressionId>>,
    /// 下一个可用的 ID
    next_id: SExpressionId,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        MemoryStorage {
            expressions: HashMap::new(),
            symbol_index: HashMap::new(),
            dependents_index: HashMap::new(),
            next_id: 1,
        }
    }

    /// 分配新的 ID
    fn allocate_id(&mut self) -> SExpressionId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 更新 symbol 索引
    fn update_symbol_index(&mut self, expr: &StoredSExpression) {
        // 添加新的 symbol name 索引
        for symbol in &expr.symbol_names {
            self.symbol_index
                .entry(symbol.clone())
                .or_default()
                .push(expr.id);
        }
    }

    /// 移除 symbol 索引
    fn remove_symbol_index(&mut self, expr: &StoredSExpression) {
        for symbol in &expr.symbol_names {
            if let Some(ids) = self.symbol_index.get_mut(symbol) {
                ids.retain(|&id| id != expr.id);
                if ids.is_empty() {
                    self.symbol_index.remove(symbol);
                }
            }
        }
    }

    /// 更新依赖索引
    fn update_dependents_index(&mut self, expr: &StoredSExpression) {
        for &dep_id in &expr.dependencies {
            self.dependents_index
                .entry(dep_id)
                .or_default()
                .push(expr.id);
        }
    }

    /// 移除依赖索引
    fn remove_dependents_index(&mut self, expr: &StoredSExpression) {
        for &dep_id in &expr.dependencies {
            if let Some(dependents) = self.dependents_index.get_mut(&dep_id) {
                dependents.retain(|&id| id != expr.id);
                if dependents.is_empty() {
                    self.dependents_index.remove(&dep_id);
                }
            }
        }
    }
}

impl SExpressionStorage for MemoryStorage {
    fn store(&mut self, mut expr: StoredSExpression) -> Result<SExpressionId, StorageError> {
        // 如果 ID 为 0，分配新 ID
        if expr.id == 0 {
            expr.id = self.allocate_id();
        }

        // 检查 ID 是否已存在
        if self.expressions.contains_key(&expr.id) {
            return Err(StorageError::AlreadyExists(expr.id));
        }

        // 更新索引
        self.update_symbol_index(&expr);
        self.update_dependents_index(&expr);

        // 存储表达式
        let id = expr.id;
        self.expressions.insert(id, expr);

        Ok(id)
    }

    fn get(&self, id: SExpressionId) -> Result<Option<StoredSExpression>, StorageError> {
        Ok(self.expressions.get(&id).cloned())
    }

    fn update(&mut self, id: SExpressionId, expr: StoredSExpression) -> Result<(), StorageError> {
        // 检查 S-Expression 是否存在
        let old_expr = self.expressions.get(&id)
            .ok_or(StorageError::NotFound(id))?.clone();

        // 移除旧的索引
        self.remove_symbol_index(&old_expr);
        self.remove_dependents_index(&old_expr);

        // 更新新的索引
        self.update_symbol_index(&expr);
        self.update_dependents_index(&expr);

        // 更新存储
        self.expressions.insert(id, expr);

        Ok(())
    }

    fn delete(&mut self, id: SExpressionId) -> Result<(), StorageError> {
        // 获取要删除的表达式
        let expr = self.expressions.remove(&id)
            .ok_or(StorageError::NotFound(id))?;

        // 移除索引
        self.remove_symbol_index(&expr);
        self.remove_dependents_index(&expr);

        Ok(())
    }

    fn list_ids(&self) -> Result<Vec<SExpressionId>, StorageError> {
        Ok(self.expressions.keys().cloned().collect())
    }

    fn find_by_symbol(&self, symbol: &str) -> Result<Vec<SExpressionId>, StorageError> {
        Ok(self.symbol_index.get(symbol).cloned().unwrap_or_default())
    }

    fn find_dependents(&self, id: SExpressionId) -> Result<Vec<SExpressionId>, StorageError> {
        Ok(self.dependents_index.get(&id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::types::Value;

    #[test]
    fn test_memory_storage_basic_operations() {
        let mut storage = MemoryStorage::new();
        
        // 创建测试 S-Expression
        let code = Rc::new(Value::Symbol("test".to_string()));
        let expr = StoredSExpression::new(
            0, // 将被自动分配
            code,
            Some("Test expression".to_string()),
            Some("test".to_string()),
            vec!["test-symbol".to_string()],
            vec![],
        );

        // 测试存储
        let id = storage.store(expr.clone()).unwrap();
        assert_eq!(id, 1);

        // 测试获取
        let retrieved = storage.get(id).unwrap().unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.description, Some("Test expression".to_string()));

        // 测试按 symbol 查找
        let found_ids = storage.find_by_symbol("test-symbol").unwrap();
        assert_eq!(found_ids, vec![id]);

        // 测试删除
        storage.delete(id).unwrap();
        assert!(storage.get(id).unwrap().is_none());
    }

    #[test]
    fn test_dependency_tracking() {
        let mut storage = MemoryStorage::new();

        // 创建被依赖的表达式
        let dep_expr = StoredSExpression::new(
            0,
            Rc::new(Value::Symbol("dependency".to_string())),
            Some("Dependency".to_string()),
            None,
            vec!["dep".to_string()],
            vec![],
        );
        let dep_id = storage.store(dep_expr).unwrap();

        // 创建依赖表达式
        let main_expr = StoredSExpression::new(
            0,
            Rc::new(Value::Symbol("main".to_string())),
            Some("Main expression".to_string()),
            None,
            vec!["main".to_string()],
            vec![dep_id],
        );
        let main_id = storage.store(main_expr).unwrap();

        // 测试依赖查找
        let dependents = storage.find_dependents(dep_id).unwrap();
        assert_eq!(dependents, vec![main_id]);
    }
}
