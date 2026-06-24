use std::path::Path;
use std::fs;

fn main() {
    // Always rebuild frontend (force npm build)
    println!("cargo:warning=[build.rs] Building frontend...");
    let status = std::process::Command::new("cmd")
        .args(&["/c", "cd /d .. && npm install && npm run build"])
        .status()
        .expect("Failed to run npm build");
    if !status.success() {
        panic!("\n=============================================================\nERROR: Frontend build failed.\nMake sure Node.js is installed and run:\n    cd win-game\n    npm install\n    npm run build\nThen retry: cargo build --release\n=============================================================\n");
    }
    println!("cargo:warning=[build.rs] Frontend build complete");

    let html = fs::read("../dist/index.html").expect("dist/index.html not found");

    use std::io::Write;
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut f = fs::File::create(Path::new(&out_dir).join("generated.rs")).unwrap();

    write!(f, "pub const HTML_BYTES: &[u8] = &[").unwrap();
    for (i, byte) in html.iter().enumerate() {
        if i > 0 { write!(f, ",").unwrap(); }
        write!(f, "{}", byte).unwrap();
    }
    write!(f, "];\n").unwrap();

    println!("cargo:warning=[build.rs] Embedded frontend: {} bytes", html.len());
    println!("cargo:rerun-if-changed=../index.html");
    println!("cargo:rerun-if-changed=../src/main.js");
    println!("cargo:rerun-if-changed=../src/styles.css");
    tauri_build::build()
}
