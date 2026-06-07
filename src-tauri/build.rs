use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn copy_mcp_server_binary() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "x86_64-pc-windows-msvc".to_string());
    let profile = if std::env::var("PROFILE").as_deref() == Ok("release") {
        "release"
    } else {
        "debug"
    };

    let base_exe = if target.contains("windows") {
        "harbor-mcp-server.exe"
    } else {
        "harbor-mcp-server"
    };

    let dst_name = if target.contains("windows") {
        format!("harbor-mcp-server-{}.exe", target)
    } else {
        format!("harbor-mcp-server-{}", target)
    };

    let dst = Path::new("binaries").join(&dst_name);

    let src_candidates = vec![
        Path::new("target").join(&target).join(profile).join(base_exe),
        Path::new("target").join(&target).join("release").join(base_exe),
        Path::new("target").join(&target).join("debug").join(base_exe),
        Path::new("target").join("release").join(base_exe),
        Path::new("target").join("debug").join(base_exe),
    ];

    for src in &src_candidates {
        if src.exists() {
            if let Some(parent) = dst.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut src_file = match fs::File::open(src) {
                Ok(f) => f,
                Err(e) => {
                    println!("cargo:warning=Failed to open source {:?}: {}", src, e);
                    continue;
                }
            };

            let mut data = Vec::new();
            if let Err(e) = src_file.read_to_end(&mut data) {
                println!("cargo:warning=Failed to read source {:?}: {}", src, e);
                continue;
            }
            drop(src_file);

            let mut dst_file = match fs::File::create(&dst) {
                Ok(f) => f,
                Err(e) => {
                    println!("cargo:warning=Failed to create dest {:?}: {}", dst, e);
                    continue;
                }
            };

            if let Err(e) = dst_file.write_all(&data) {
                println!("cargo:warning=Failed to write dest {:?}: {}", dst, e);
                continue;
            }
            drop(dst_file);

            println!("cargo:warning=Copied MCP server from {:?} to binaries/{}", src, dst_name);
            return;
        }
    }

    if !dst.exists() {
        println!("cargo:warning=MCP server binary not found in any candidate path, binaries/{} will be missing", dst_name);
    }
}

fn main() {
    copy_mcp_server_binary();
    tauri_build::build()
}
