fn main() {
    cc::Build::new()
        .file("src/backend/tests/log.c")
        .compile("cubeb_log_internal");
}
