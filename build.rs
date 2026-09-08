fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set_language(0x0409);
        res.set("ProductName", "Recenter Mouse");
        res.set("FileDescription", "Recenter Mouse Cursor and Window Utility");
        res.set("LegalCopyright", "Copyright (C) 2026");
        res.compile().expect("Failed to compile Windows resource");
    }
}
