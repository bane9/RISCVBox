#[cfg(windows)]
fn main() {
    embed_resource::compile("assets/manifest.rc", embed_resource::NONE);
}

#[cfg(not(windows))]
fn main() {}
