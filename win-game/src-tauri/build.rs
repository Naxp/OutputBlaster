use std::path::Path;
use std::fs;

fn main() {
    // Ensure frontend is built
    let dist = Path::new("../dist/index.html");
    if !dist.exists() {
        println!("cargo:warning=[build.rs] dist/ not found -- building frontend...");
        let status = std::process::Command::new("cmd")
            .args(&["/c", "cd /d .. && npm install && npm run build"])
            .status()
            .expect("Failed to run npm build");
        if !status.success() {
            panic!("\n=============================================================\nERROR: Frontend build failed.\nMake sure Node.js is installed and run:\n    cd win-game\n    npm install\n    npm run build\nThen retry: cargo build --release\n=============================================================\n");
        }
        println!("cargo:warning=[build.rs] Frontend build complete");
    }

    // Read dist files
    let html = fs::read_to_string("../dist/index.html").expect("dist/index.html not found");
    let css = fs::read_to_string("../dist/styles.css").expect("dist/styles.css not found");

    let js_dir = fs::read_dir("../dist/assets").expect("dist/assets/ not found");
    let js_path = js_dir
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "js").unwrap_or(false))
        .expect("No JS bundle found in dist/assets/");
    let js = fs::read_to_string(js_path.path()).expect("Failed to read JS bundle");

    // Combine into single HTML
    let combined = html
        .replace(
            "<link rel=\"stylesheet\" href=\"/styles.css\" />",
            &format!("<style>{}</style>", css),
        )
        .replace(
            "<script type=\"module\" crossorigin src=\"/assets/index-",
            "<script>",
        )
        .replace("\"></script>", "</script>")
        + &format!("<script>{}</script>", js);

    // Generate .rs file with the HTML as a raw string constant
    // Use r##"..."## to avoid escaping issues with quotes in the HTML
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let rs_content = format!(
        "pub const HTML: &str = r###\"{}\"###;\n",
        combined
    );
    fs::write(Path::new(&out_dir).join("generated.rs"), &rs_content)
        .expect("Failed to write generated.rs");

    println!(
        "cargo:warning=[build.rs] Embedded frontend: {} bytes",
        combined.len()
    );
    println!("cargo:rerun-if-changed=../dist/index.html");
    println!("cargo:rerun-if-changed=../dist/styles.css");

    tauri_build::build()
}
