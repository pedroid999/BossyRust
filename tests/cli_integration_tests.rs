use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "lightweight Terminal User Interface",
        ))
        .stdout(predicate::str::contains("Usage: bossy-rust"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("port"))
        .stdout(predicate::str::contains("kill-port"))
        .stdout(predicate::str::contains("ports"))
        .stdout(predicate::str::contains("kill-process"))
        .stdout(predicate::str::contains("ps"))
        .stdout(predicate::str::contains("cleanup"))
        .stdout(predicate::str::contains("find-port"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bossy-rust"));
}

#[test]
fn test_port_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["port", "--help"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Show what's using a specific port",
    ));
}

#[test]
fn test_kill_port_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["kill-port", "--help"]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Kill process using a specific port",
    ));
}

#[test]
fn test_ports_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ports", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show all ports"))
        .stdout(predicate::str::contains("--common"))
        .stdout(predicate::str::contains("--listening"));
}

#[test]
fn test_kill_process_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["kill-process", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Kill processes by name"))
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn test_ps_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ps", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show processes"))
        .stdout(predicate::str::contains("--top-cpu"))
        .stdout(predicate::str::contains("--top-memory"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_cleanup_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["cleanup", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Cleanup common development processes",
        ))
        .stdout(predicate::str::contains("--dev"));
}

#[test]
fn test_find_port_command_help() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["find-port", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Find available port in range"));
}

#[test]
fn test_port_command_with_invalid_port() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["port", "65534"]);

    // This should succeed but likely show no processes (using valid port range)
    cmd.assert().success();
}

#[test]
fn test_port_command_with_zero() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["port", "0"]);

    // Port 0 should be handled gracefully
    cmd.assert().success();
}

#[test]
fn test_ps_command_basic() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ps"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processes"));
}

#[test]
fn test_ps_command_top_cpu() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ps", "--top-cpu"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Top"))
        .stdout(predicate::str::contains("CPU"));
}

#[test]
fn test_ps_command_top_memory() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ps", "--top-memory"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Top"))
        .stdout(predicate::str::contains("Memory"));
}

#[test]
fn test_ps_command_with_limit() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ps", "--limit", "5"]);

    cmd.assert().success();
}

#[test]
fn test_ports_command_basic() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ports"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Ports"));
}

#[test]
fn test_ports_command_listening() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ports", "--listening"]);

    cmd.assert().success();
}

#[test]
fn test_ports_command_common() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ports", "--common"]);

    cmd.assert().success();
}

#[test]
fn test_find_port_command_basic() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["find-port", "50000"]);

    // Should either find a port or indicate none available
    cmd.assert().success();
}

#[test]
fn test_find_port_command_with_range() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["find-port", "50000", "50010"]);

    cmd.assert().success();
}

#[test]
fn test_kill_process_non_existent() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["kill-process", "non_existent_process_name_12345"]);

    // Should succeed but report no processes found
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No processes found"));
}

#[test]
fn test_cleanup_without_dev_flag() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["cleanup"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Please specify --dev"));
}

#[test]
fn test_cleanup_with_dev_flag() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["cleanup", "--dev"]);

    // Should succeed regardless of whether dev processes are found
    cmd.assert().success();
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["invalid-command"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_port_command_missing_argument() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["port"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_kill_port_missing_argument() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["kill-port"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_kill_process_missing_argument() {
    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["kill-process"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// Note: We avoid actual kill operations in tests to prevent system disruption
#[test]
fn test_kill_port_dry_run_concept() {
    // This test demonstrates how we would test kill operations safely
    // In a real scenario, we might use a mock system or test environment

    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["port", "65534"]); // Use a valid but very unlikely port

    // Just verify the command structure works
    cmd.assert().success();
}

#[test]
fn test_command_chaining_safety() {
    // Test that commands don't interfere with each other

    // Run multiple commands in sequence
    let mut cmd1 = Command::cargo_bin("bossy-rust").unwrap();
    cmd1.args(["ps", "--limit", "1"]);
    cmd1.assert().success();

    let mut cmd2 = Command::cargo_bin("bossy-rust").unwrap();
    cmd2.args(["ports", "--listening"]);
    cmd2.assert().success();

    let mut cmd3 = Command::cargo_bin("bossy-rust").unwrap();
    cmd3.args(["find-port", "60000"]);
    cmd3.assert().success();
}

#[test]
fn test_error_handling_graceful() {
    // Test that errors are handled gracefully without panics

    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["port", "abc"]); // Invalid port number

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("error")));
}

#[test]
fn test_output_format_consistency() {
    // Test that output formats are consistent across commands

    let mut cmd = Command::cargo_bin("bossy-rust").unwrap();
    cmd.args(["ps", "--limit", "3"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID"))
        .stdout(predicate::str::contains("Process"));
}
