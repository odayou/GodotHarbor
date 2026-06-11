fn main() {
    // Fix: MINGW_CHOST may be set to i686-w64-mingw32 by some environments (e.g. MSYS2 32-bit),
    // which causes embed-resource/windres to produce 32-bit COFF output.
    // Override it to match the actual GNU target architecture.
    if std::env::var("TARGET").map(|t| t.starts_with("x86_64")).unwrap_or(false) {
        std::env::set_var("MINGW_CHOST", "x86_64-w64-mingw32");
    }

    tauri_build::build()
}
