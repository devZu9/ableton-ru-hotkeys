fn main() {
    #[cfg(windows)]
    embed_resource::compile("resource/resource.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
