fn main() {
    let dist = std::path::Path::new("../dist/index.html");
    if !dist.exists() {
        println!("cargo:warning=[build.rs] dist/index.html not found -- building frontend...");
        let status = std::process::Command::new("cmd")
            .args(&["/c", "cd /d .. && npm run build"])
            .status()
            .expect("Failed to run npm build");

        if !status.success() {
            let msg = "\n
=============================================================
ERROR: Frontend build failed.
Make sure Node.js is installed and run:
    cd win-game
    npm install
    npm run build
Then retry: cargo build --release
=============================================================
";
            panic!("{}", msg);
        }
        println!("cargo:warning=[build.rs] Frontend build complete");
    }
    tauri_build::build()
}
