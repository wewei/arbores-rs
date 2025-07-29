#!/bin/bash

# 测试增强版 REPL 的功能
echo "Testing Enhanced REPL features..."

# 测试基本表达式
echo "(+ 1 2 3)" | cargo run --quiet

echo "---"

# 测试多个表达式
echo -e "(define x 42)\nx" | cargo run --quiet

echo "---"

# 测试帮助命令
echo ":help" | timeout 2s cargo run --quiet || true

echo "Enhanced REPL tests completed!"
