use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

type Result<T> = std::result::Result<T, String>;

const USAGE: &str = "usage: cargo buildsrc <task>

tasks:
  ci      format check, clippy, tests
  dist    release build packaged into dist/";

fn main() -> ExitCode {
    let outcome = match env::args().nth(1).as_deref() {
        Some("ci") => ci(),
        Some("dist") => dist(),
        Some(task) => Err(format!("unknown task: {task}\n\n{USAGE}")),
        None => Err(USAGE.to_string()),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn ci() -> Result<()> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo_with_profile(&["test", "--workspace"], "test")
}

fn dist() -> Result<()> {
    let root = project_root();
    let version = package_version(&root)?;
    let dist = root.join("dist");
    let assets = root.join("assets");
    let config = root.join("config");

    cargo(&["build", "--release", "--package", "boot"])?;

    let binary = root.join("target/release/econbox");
    reject_dynamic_linking(&binary)?;

    if dist.exists() {
        fs::remove_dir_all(&dist).map_err(|error| format!("clean dist: {error}"))?;
    }

    let contents = dist.join("econbox.app/Contents");
    let macos = contents.join("MacOS");
    create_dir(&macos)?;
    create_dir(&contents.join("Resources"))?;
    copy_file(&binary, &macos.join("econbox"))?;
    copy_dir(&assets, &macos.join("assets"))?;
    copy_dir(&config, &macos.join("config"))?;
    fs::write(contents.join("Info.plist"), info_plist(&version))
        .map_err(|error| format!("write Info.plist: {error}"))?;

    let plain = dist.join(format!("econbox-{version}"));
    create_dir(&plain)?;
    copy_file(&binary, &plain.join("econbox"))?;
    copy_dir(&assets, &plain.join("assets"))?;
    copy_dir(&config, &plain.join("config"))?;

    archive(
        &dist.join("econbox.app"),
        &dist.join(format!("econbox-{version}-macos-app.zip")),
    )?;
    archive(&plain, &dist.join(format!("econbox-{version}-macos.zip")))?;

    println!("packaged {} into {}", version, dist.display());
    Ok(())
}

fn reject_dynamic_linking(binary: &Path) -> Result<()> {
    let output = Command::new("otool")
        .arg("-L")
        .arg(binary)
        .output()
        .map_err(|error| format!("run otool: {error}"))?;

    if String::from_utf8_lossy(&output.stdout).contains("libbevy_dylib") {
        return Err(
            "release binary links libbevy_dylib: the fast-link feature must stay off for dist"
                .to_string(),
        );
    }
    Ok(())
}

fn info_plist(version: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>econbox</string>
  <key>CFBundleDisplayName</key><string>econbox</string>
  <key>CFBundleIdentifier</key><string>com.wangxiang.econbox</string>
  <key>CFBundleExecutable</key><string>econbox</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"#
    )
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the project root")
        .to_path_buf()
}

fn package_version(root: &Path) -> Result<String> {
    let manifest = fs::read_to_string(root.join("Cargo.toml"))
        .map_err(|error| format!("read Cargo.toml: {error}"))?;

    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[workspace.package]";
        } else if in_package
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "version"
        {
            return Ok(value.trim().trim_matches('"').to_string());
        }
    }
    Err("Cargo.toml has no version under [workspace.package]".to_string())
}

fn cargo_with_profile(args: &[&str], profile: &str) -> Result<()> {
    let program = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(program)
        .current_dir(project_root())
        .env("ECONBOX_PROFILE", profile)
        .args(args)
        .status()
        .map_err(|error| format!("run cargo: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", args.join(" ")))
    }
}

fn cargo(args: &[&str]) -> Result<()> {
    let program = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(program)
        .current_dir(project_root())
        .args(args)
        .status()
        .map_err(|error| format!("run cargo: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", args.join(" ")))
    }
}

fn archive(source: &Path, target: &Path) -> Result<()> {
    let status = Command::new("ditto")
        .args(["-c", "-k", "--sequesterRsrc", "--keepParent"])
        .arg(source)
        .arg(target)
        .status()
        .map_err(|error| format!("run ditto: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("ditto failed for {}", source.display()))
    }
}

fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|error| format!("create {}: {error}", path.display()))
}

fn copy_file(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)
        .map(drop)
        .map_err(|error| format!("copy {} to {}: {error}", from.display(), to.display()))
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    create_dir(to)?;
    let entries =
        fs::read_dir(from).map_err(|error| format!("read {}: {error}", from.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("read {}: {error}", from.display()))?;
        let name = entry.file_name();
        if name == ".DS_Store" {
            continue;
        }

        let source = entry.path();
        let target = to.join(&name);
        if source.is_dir() {
            copy_dir(&source, &target)?;
        } else {
            copy_file(&source, &target)?;
        }
    }
    Ok(())
}
