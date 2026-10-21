use std::{
    io::Write,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn runtime_profile_discovery_selection_help_and_default_command_mode() {
    let directory = std::env::temp_dir().join(format!(
        "heart-cli-profiles-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("pirate.json"), r#"{"id":"pirate","instructions":"A brief pirate","greeting":"Ahoy, captain!","status":"Checking charts."}"#).unwrap();
    std::fs::write(directory.join("custom.json"), r#"{"id":"custom","instructions":"Brief","greeting":"Custom greeting.","status":"Checking."}"#).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_heart"))
        .current_dir(&directory)
        .args([
            "--profiles-dir",
            directory.to_str().unwrap(),
            "--personality",
            "custom",
            "--endpoint",
            "http://127.0.0.1:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"/help\n/pirate\n/profile custom\n/profile unknown\nHow long has this computer been running?\n/unknown\n/system unexpected\n/search\n/exit\n").unwrap();
    let output = child.wait_with_output().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Custom greeting.",
        "Ahoy, captain!",
        "/disk [path]",
        "/search <query>",
        "/system",
        "/uptime",
        "Unknown profile",
        "requires --experimental-nl",
        "Unknown command",
        "takes no arguments",
        "Invalid or missing command argument",
    ] {
        assert!(text.contains(expected), "Missing {expected}: {text}");
    }
    assert!(!text.contains("Check Ollama"));
}

#[test]
fn dashboard_rejects_noninteractive_input_clearly_before_initializing_services() {
    let output = Command::new(env!("CARGO_BIN_EXE_heart"))
        .current_dir(std::env::temp_dir())
        .args(["--dashboard", "--endpoint", "not even a URL"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requires an interactive terminal (TTY)")
    );
}
