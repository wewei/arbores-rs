#!/usr/bin/env bash

# 测试脚本：验证 Arbores Scheme 解释器的基本功能

echo "Testing Arbores Scheme Interpreter..."

# 基本算术测试
echo "Testing arithmetic operations:"
echo "(+ 1 2 3)" | cargo run --bin arbores 2>/dev/null | grep -q "6" && echo "✓ Addition works" || echo "✗ Addition failed"
echo "(* 2 3 4)" | cargo run --bin arbores 2>/dev/null | grep -q "24" && echo "✓ Multiplication works" || echo "✗ Multiplication failed"
echo "(- 10 3)" | cargo run --bin arbores 2>/dev/null | grep -q "7" && echo "✓ Subtraction works" || echo "✗ Subtraction failed"

# 列表操作测试
echo -e "\nTesting list operations:"
echo "(cons 1 2)" | cargo run --bin arbores 2>/dev/null | grep -q "(1 . 2)" && echo "✓ Cons works" || echo "✗ Cons failed"
echo "(list 1 2 3)" | cargo run --bin arbores 2>/dev/null | grep -q "(1 2 3)" && echo "✓ List works" || echo "✗ List failed"

# 引用测试
echo -e "\nTesting quote:"
echo "'hello" | cargo run --bin arbores 2>/dev/null | grep -q "hello" && echo "✓ Quote works" || echo "✗ Quote failed"

# 条件测试
echo -e "\nTesting conditionals:"
echo "(if #t 1 2)" | cargo run --bin arbores 2>/dev/null | grep -q "1" && echo "✓ If true works" || echo "✗ If true failed"
echo "(if #f 1 2)" | cargo run --bin arbores 2>/dev/null | grep -q "2" && echo "✓ If false works" || echo "✗ If false failed"

echo -e "\nBasic tests completed!"
