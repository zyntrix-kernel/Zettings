fn main() {
    cxx_qt_build::CxxQtBuilder::new()
        .file("src/app_info.rs")
        .qt_module("Quick")
        .build();
}
