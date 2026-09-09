#[cfg(windows)]
fn main() {
    let mut resource = winres::WindowsResource::new();
    resource.set_icon("assets/icon.ico");
    resource.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
