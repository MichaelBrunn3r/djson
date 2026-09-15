use std::io::Write;
use std::process::{Command, Stdio};

/// Runs `djson` with `input` on standard input.
fn run(input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_djson"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start djson");

    child
        .stdin
        .take()
        .expect("djson stdin was not piped")
        .write_all(input.as_bytes())
        .expect("failed to write djson input");

    child.wait_with_output().expect("failed to wait for djson")
}

#[test]
fn evaluates_stdin() {
    let output = run("1 + 2");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"3\n");
}

#[test]
fn reports_evaluation_errors_with_a_source_span() {
    let output = run("result: 1 / 0");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("eval::division_by_zero"), "{stderr}");
    assert!(stderr.contains("division by zero"), "{stderr}");
    assert!(stderr.contains("1 / 0"), "{stderr}");
}
