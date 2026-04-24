use std::path::{Path, PathBuf};
use std::process::Command;

fn ip() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ip"))
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn data(name: &str) -> String {
    workspace().join("tests/data").join(name).to_string_lossy().into_owned()
}

fn config(name: &str) -> String {
    workspace().join("tests/config").join(name).to_string_lossy().into_owned()
}

fn plugin_dir() -> String {
    workspace().join("target/debug").to_string_lossy().into_owned()
}

fn tmp_output(name: &str) -> String {
    std::env::temp_dir().join(name).to_string_lossy().into_owned()
}

// --- argument validation ---

#[test]
fn missing_input_file_exits_with_error() {
    let out = ip()
        .args([
            "-i", "/nonexistent/input.png",
            "-o", &tmp_output("ip_test_dummy.png"),
            "-p", "mirror",
            "-d", &config("mirror_h.json"),
            "-l", &plugin_dir(),
        ])
        .output()
        .expect("failed to run ip");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("input file not found"), "stderr: {stderr}");
}

#[test]
fn missing_params_file_exits_with_error() {
    let out = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &tmp_output("ip_test_dummy.png"),
            "-p", "mirror",
            "-d", "/nonexistent/params.json",
            "-l", &plugin_dir(),
        ])
        .output()
        .expect("failed to run ip");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("params file not found"), "stderr: {stderr}");
}

#[test]
fn missing_plugin_dir_exits_with_error() {
    let out = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &tmp_output("ip_test_dummy.png"),
            "-p", "mirror",
            "-d", &config("mirror_h.json"),
            "-l", "/nonexistent/plugins",
        ])
        .output()
        .expect("failed to run ip");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("plugin directory not found"), "stderr: {stderr}");
}

#[test]
fn missing_output_dir_exits_with_error() {
    let out = ip()
        .args([
            "-i", &data("2.png"),
            "-o", "/nonexistent/dir/out.png",
            "-p", "mirror",
            "-d", &config("mirror_h.json"),
            "-l", &plugin_dir(),
        ])
        .output()
        .expect("failed to run ip");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("output directory not found"), "stderr: {stderr}");
}

#[test]
fn no_args_exits_nonzero() {
    let out = ip().output().expect("failed to run ip");
    assert!(!out.status.success());
}

// --- processing (plugins must be built: cargo build --workspace before test) ---

#[test]
fn mirror_horizontal_produces_output_file() {
    let output = tmp_output("ip_test_mirror_h.png");
    let _ = std::fs::remove_file(&output);

    let status = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &output,
            "-p", "mirror",
            "-d", &config("mirror_h.json"),
            "-l", &plugin_dir(),
        ])
        .status()
        .expect("failed to run ip");

    assert!(status.success(), "ip exited with {status}");
    assert!(Path::new(&output).exists(), "output file not created");
}

#[test]
fn mirror_vertical_produces_output_file() {
    let output = tmp_output("ip_test_mirror_v.png");
    let _ = std::fs::remove_file(&output);

    let status = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &output,
            "-p", "mirror",
            "-d", &config("mirror_v.json"),
            "-l", &plugin_dir(),
        ])
        .status()
        .expect("failed to run ip");

    assert!(status.success(), "ip exited with {status}");
    assert!(Path::new(&output).exists());
}

#[test]
fn mirror_all_produces_output_file() {
    let output = tmp_output("ip_test_mirror_all.png");
    let _ = std::fs::remove_file(&output);

    let status = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &output,
            "-p", "mirror",
            "-d", &config("mirror_all.json"),
            "-l", &plugin_dir(),
        ])
        .status()
        .expect("failed to run ip");

    assert!(status.success(), "ip exited with {status}");
    assert!(Path::new(&output).exists());
}

#[test]
fn blur_box_produces_output_file() {
    let output = tmp_output("ip_test_blur_box.png");
    let _ = std::fs::remove_file(&output);

    let status = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &output,
            "-p", "blur",
            "-d", &config("blur_box_mild.json"),
            "-l", &plugin_dir(),
        ])
        .status()
        .expect("failed to run ip");

    assert!(status.success(), "ip exited with {status}");
    assert!(Path::new(&output).exists());
}

#[test]
fn blur_gauss_produces_output_file() {
    let output = tmp_output("ip_test_blur_gauss.png");
    let _ = std::fs::remove_file(&output);

    let status = ip()
        .args([
            "-i", &data("2.png"),
            "-o", &output,
            "-p", "blur",
            "-d", &config("blur_gauss.json"),
            "-l", &plugin_dir(),
        ])
        .status()
        .expect("failed to run ip");

    assert!(status.success(), "ip exited with {status}");
    assert!(Path::new(&output).exists());
}
