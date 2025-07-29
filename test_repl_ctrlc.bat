#!/bin/bash
# Windows PowerShell 版本的测试脚本

echo "测试 REPL Ctrl+C 优雅退出功能"
echo "启动 REPL，然后按 Ctrl+C 应该显示 'Goodbye!' 而不是错误信息"
echo ""
echo "按任意键启动 REPL 测试..."
pause

# 运行 REPL
cargo run --bin arbores
