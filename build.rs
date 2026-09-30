fn main() {
    #[cfg(windows)]
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("windows/icon.ico");
        resource.set_language(0x0409);
        if let Err(error) = resource.compile() {
            eprintln!("warning: failed to embed Windows icon: {error}");
        }
    }
}
