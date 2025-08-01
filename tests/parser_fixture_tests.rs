use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use arbores::interpreter::parser::parse_from_string;

/// YAML fixture 文件中的测试用例定义
#[derive(Debug, Deserialize, Serialize)]
struct FixtureFile {
    test_cases: Option<Vec<TestCase>>,
    error_cases: Option<Vec<ErrorCase>>,
}

/// 成功解析的测试用例
#[derive(Debug, Deserialize, Serialize)]
struct TestCase {
    name: String,
    description: String,
    input: String,
    expected: String,
}

/// 应该失败的测试用例
#[derive(Debug, Deserialize, Serialize)]
struct ErrorCase {
    name: String,
    description: String,
    input: String,
    should_fail: bool,
}

/// 从 YAML 文件加载 fixture
fn load_fixture(path: &Path) -> Result<FixtureFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let fixture: FixtureFile = serde_yaml::from_str(&content)?;
    Ok(fixture)
}

/// 运行单个测试用例
fn run_test_case(test_case: &TestCase) -> Result<(), String> {
    // 解析输入字符串
    let parse_output = parse_from_string(&test_case.input);
    
    match &parse_output.result {
        Ok(expressions) => {
            // 对于我们的fixture测试，每个输入应该产生单个表达式
            if expressions.len() != 1 {
                return Err(format!(
                    "Expected single expression for test '{}', got {} expressions\nInput: {}",
                    test_case.name, expressions.len(), test_case.input
                ));
            }
            
            // 使用 pretty format 将 SExpr 转换为字符串
            let actual_output = expressions[0].to_pretty_string();
            
            // YAML 的 "|" 块标量会在末尾保留换行符，我们需要去除以进行比较
            let expected_trimmed = test_case.expected.trim_end();
            let actual_trimmed = actual_output.trim_end();
            
            // 比较实际输出与预期输出
            if actual_trimmed == expected_trimmed {
                Ok(())
            } else {
                Err(format!(
                    "Output mismatch for test '{}'\nExpected: {}\nActual:   {}",
                    test_case.name, test_case.expected, actual_output
                ))
            }
        }
        Err(err) => {
            Err(format!(
                "Parse failed for test '{}': {}\nInput: {}",
                test_case.name, err, test_case.input
            ))
        }
    }
}

/// 运行单个错误测试用例
fn run_error_case(error_case: &ErrorCase) -> Result<(), String> {
    let parse_output = parse_from_string(&error_case.input);
    
    if error_case.should_fail {
        match &parse_output.result {
            Ok(expressions) => {
                Err(format!(
                    "Expected parse failure for test '{}', but got success: {} expressions\nInput: {}",
                    error_case.name, expressions.len(), error_case.input
                ))
            }
            Err(_) => {
                // 预期的失败
                Ok(())
            }
        }
    } else {
        // 这个分支目前不会使用，因为 should_fail 总是 true
        match &parse_output.result {
            Ok(_) => Ok(()),
            Err(err) => {
                Err(format!(
                    "Unexpected parse failure for test '{}': {}\nInput: {}",
                    error_case.name, err, error_case.input
                ))
            }
        }
    }
}

/// 运行指定 fixture 文件中的所有测试
fn run_fixture_tests(fixture_path: &Path) -> Result<(), Vec<String>> {
    let fixture = load_fixture(fixture_path)
        .map_err(|e| vec![format!("Failed to load fixture {}: {}", fixture_path.display(), e)])?;
    
    let mut errors = Vec::new();
    
    // 运行成功测试用例
    if let Some(test_cases) = &fixture.test_cases {
        for test_case in test_cases {
            if let Err(err) = run_test_case(test_case) {
                errors.push(err);
            }
        }
    }
    
    // 运行错误测试用例
    if let Some(error_cases) = &fixture.error_cases {
        for error_case in error_cases {
            if let Err(err) = run_error_case(error_case) {
                errors.push(err);
            }
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[test]
fn test_basic_expressions_fixture() {
    let fixture_path = Path::new("tests/fixtures/parser/basic_expressions.yaml");
    
    match run_fixture_tests(fixture_path) {
        Ok(()) => {
            println!("All basic expression tests passed!");
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error);
            }
            panic!("Basic expression fixture tests failed with {} errors", errors.len());
        }
    }
}

#[test]
fn test_edge_cases_fixture() {
    let fixture_path = Path::new("tests/fixtures/parser/edge_cases.yaml");
    
    match run_fixture_tests(fixture_path) {
        Ok(()) => {
            println!("All edge case tests passed!");
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error);
            }
            panic!("Edge case fixture tests failed with {} errors", errors.len());
        }
    }
}

/// 运行所有 fixture 测试的便利函数
#[test]
fn test_all_parser_fixtures() {
    let fixture_dir = Path::new("tests/fixtures/parser");
    
    if !fixture_dir.exists() {
        panic!("Fixture directory not found: {}", fixture_dir.display());
    }
    
    let mut all_errors = Vec::new();
    let mut test_count = 0;
    
    // 遍历所有 .yaml 文件
    for entry in fs::read_dir(fixture_dir).expect("Failed to read fixture directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            test_count += 1;
            println!("Running fixture: {}", path.display());
            
            match run_fixture_tests(&path) {
                Ok(()) => {
                    println!("  ✓ All tests passed");
                }
                Err(mut errors) => {
                    println!("  ✗ {} tests failed", errors.len());
                    all_errors.append(&mut errors);
                }
            }
        }
    }
    
    if all_errors.is_empty() {
        println!("All {} fixture files passed!", test_count);
    } else {
        for error in &all_errors {
            eprintln!("{}", error);
        }
        panic!("Fixture tests failed with {} total errors across {} files", 
               all_errors.len(), test_count);
    }
}
