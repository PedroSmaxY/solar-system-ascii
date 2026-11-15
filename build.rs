#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "AstroEngine ASCII");
    res.set("FileDescription", "Real-time 3D Solar System Simulator in TrueColor ANSI Half-Block raymarching");
    res.set("LegalCopyright", "Copyright (c) 2026 Pedro Henrique (PedroSmaxY)");
    if let Err(e) = res.compile() {
        eprintln!("Warning: failed to compile Windows resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
