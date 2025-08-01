# Arbores Test Fixture Generator

这个工具用于生成 parser 测试的 fixture 文件。

## 使用方法

```bash
# 生成基本表达式的 fixture
cargo run --bin generate_fixture basic_expressions > tests/fixtures/parser/basic_expressions.yaml

# 生成边界情况的 fixture
cargo run --bin generate_fixture edge_cases > tests/fixtures/parser/edge_cases.yaml
```

## 支持的测试套件

- `basic_expressions`: 基本表达式解析测试（整数、浮点数、字符串、符号、列表、引用等）
- `edge_cases`: 边界情况和错误处理测试（嵌套结构、特殊字符、错误情况等）

## 输出格式

工具会输出标准的 YAML 格式，包含：
- 测试用例名称和描述
- 输入代码
- 期望的 pretty-printed 输出（自动对齐格式）
- 错误情况的标记

生成的 fixture 可以直接用于 `parser_fixture_tests.rs` 中的测试。
