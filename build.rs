fn main() {
    if cfg!(target_os = "windows") {
        embed_resource::compile("resources.rc"); // ignore this error
    }
}
