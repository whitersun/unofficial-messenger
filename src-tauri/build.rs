use std::{env, fs, io, path::PathBuf, process::Command};

fn main() {
    bundle_app_chrome_script().expect("failed to bundle app chrome script");
    tauri_build::build()
}

fn bundle_app_chrome_script() -> io::Result<()> {
    let manifest_path = PathBuf::from("src/app_chrome/index.js");
    let script_dir = manifest_path
        .parent()
        .expect("index.js should have a parent");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    let output_path = out_dir.join("app_chrome_index.js");
    let raw_output_path = out_dir.join("app_chrome_index.raw.js");
    let manifest = fs::read_to_string(&manifest_path)?;
    let mut bundle = String::from("(function() {\n");

    println!("cargo:rerun-if-env-changed=TAURI_MESSENGER_MINIFY_APP_CHROME_SCRIPT");
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    for line in manifest.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            bundle.push('\n');
            continue;
        }

        if !trimmed.starts_with("import ") {
            bundle.push_str(line);
            bundle.push('\n');
            continue;
        }

        if let Some((identifier, path)) = parse_default_import(trimmed) {
            let import_path = script_dir.join(path);
            println!("cargo:rerun-if-changed={}", import_path.display());

            let contents = fs::read_to_string(import_path)?;
            bundle.push_str("    const ");
            bundle.push_str(identifier);
            bundle.push_str(" = ");
            bundle.push_str(&js_string_literal(&contents));
            bundle.push_str(";\n");
            continue;
        }

        if let Some(path) = parse_side_effect_import(trimmed) {
            let import_path = script_dir.join(path);
            println!("cargo:rerun-if-changed={}", import_path.display());

            bundle.push_str(&fs::read_to_string(import_path)?);
            if !bundle.ends_with('\n') {
                bundle.push('\n');
            }
            continue;
        }

        panic!("unsupported app chrome script import: {trimmed}");
    }

    bundle.push_str("})();\n");
    fs::write(&raw_output_path, bundle)?;

    if should_minify_app_chrome_script() {
        minify_app_chrome_script(&raw_output_path, &output_path)?;
    } else {
        fs::copy(&raw_output_path, &output_path)?;
    }

    Ok(())
}

fn parse_default_import(line: &str) -> Option<(&str, &str)> {
    let (identifier, path) = line
        .strip_prefix("import ")?
        .strip_suffix(';')?
        .split_once(" from ")?;

    Some((identifier.trim(), unquote_path(path.trim())?))
}

fn parse_side_effect_import(line: &str) -> Option<&str> {
    unquote_path(line.strip_prefix("import ")?.strip_suffix(';')?.trim())
}

fn unquote_path(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
}

fn js_string_literal(value: &str) -> String {
    let mut literal = String::from("\"");

    for ch in value.chars() {
        match ch {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            '\u{2028}' => literal.push_str("\\u2028"),
            '\u{2029}' => literal.push_str("\\u2029"),
            _ => literal.push(ch),
        }
    }

    literal.push('"');
    literal
}

fn should_minify_app_chrome_script() -> bool {
    env::var("PROFILE").is_ok_and(|profile| profile == "release")
        && env::var("TAURI_MESSENGER_MINIFY_APP_CHROME_SCRIPT")
            .map(|value| value != "0")
            .unwrap_or(true)
}

fn minify_app_chrome_script(input_path: &PathBuf, output_path: &PathBuf) -> io::Result<()> {
    let esbuild_path =
        PathBuf::from("..")
            .join("node_modules")
            .join(".bin")
            .join(if cfg!(windows) {
                "esbuild.exe"
            } else {
                "esbuild"
            });

    if !esbuild_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "esbuild was not found at {}. Run the frontend package install before building release, or set TAURI_MESSENGER_MINIFY_APP_CHROME_SCRIPT=0 to disable minify.",
                esbuild_path.display()
            ),
        ));
    }

    let output = Command::new(&esbuild_path)
        .arg(input_path)
        .arg("--minify")
        .arg("--target=es2020")
        .arg(format!("--outfile={}", output_path.display()))
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("esbuild failed to minify app chrome script: {stderr}"),
    ))
}
