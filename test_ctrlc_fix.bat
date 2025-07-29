@echo off
echo 测试 REPL Ctrl+C 处理...
echo.
echo 1. 编译项目...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo 编译失败！
    exit /b 1
)

echo.
echo 2. 启动 REPL 测试...
echo 请在 REPL 提示符处按 Ctrl+C 测试优雅退出
echo 应该看到 "Goodbye!" 而不是错误信息
echo.
target\release\arbores.exe

echo.
echo 测试完成。如果看到 "Goodbye!" 消息而不是错误，则修复成功！
