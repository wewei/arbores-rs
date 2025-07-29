#[cfg(test)]
mod cli_tests {
    use std::process::Command;
    use std::io::Write;

    #[test]
    fn test_eval_parameter() {
        let output = Command::new("cargo")
            .args(&["run", "--", "-e", "(+ 1 2 3)"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "6");
    }

    #[test]
    fn test_stdin_input() {
        let mut child = Command::new("cargo")
            .args(&["run"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(b"(* 4 5)\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "20");
    }

    #[test]
    fn test_multiple_expressions_stdin() {
        let mut child = Command::new("cargo")
            .args(&["run"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(b"(+ 1 2)\n(* 3 4)\n").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "3\n12");
    }

    #[test]
    fn test_empty_stdin() {
        let mut child = Command::new("cargo")
            .args(&["run"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(b"").unwrap();
        }

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "");
    }
}
