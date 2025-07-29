# REPL Ctrl+C 退出测试

## 修复内容
- 添加 `ctrlc` 依赖来正确处理 Ctrl+C 信号
- 在 REPL 交互模式中设置信号处理器
- 使用 `AtomicBool` 标志来优雅地检测和处理中断
- 区分中断信号 (Ctrl+C) 和其他 I/O 错误

## 技术细节
- 使用 `ctrlc` crate 处理跨平台信号
- 在主循环中检查中断标志
- 只在交互模式下设置信号处理器，避免测试冲突
- 处理器设置失败时提供警告信息

## 测试步骤
1. 运行: `cargo run --bin arbores`
2. 在 REPL 提示符处按 Ctrl+C
3. 应该看到 "Goodbye!" 并正常退出，而不是错误信息

## 预期行为
- **修复前**: 显示 `error: process didn't exit successfully: target\debug\arbores.exe (exit code: 0xc000013a, STATUS_CONTROL_C_EXIT)`
- **修复后**: 显示 "Goodbye!" 并优雅退出，退出码为 0

## 其他退出方式
- 输入 `exit` 或 `quit`
- 按 Ctrl+D (Unix) 或 Ctrl+Z (Windows) 发送 EOF
