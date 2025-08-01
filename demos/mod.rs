//! 演示程序模块
//! 
//! 本模块包含各种功能的演示代码，用于展示和测试不同组件的功能。

pub mod lexer;
pub mod legacy;

// 导出主要演示功能
pub use lexer::demo::demo_lexer;
pub use lexer::number_demo::run_number_parsing_demo;

/// 运行所有演示程序
pub fn run_all_demos() {
    println!("=== Arbores 演示程序集合 ===\n");
    
    println!("1. 词法分析器演示");
    demo_lexer();
    
    println!("\n2. 数值解析演示");
    run_number_parsing_demo();
    
    println!("\n=== 演示完成 ===");
}
