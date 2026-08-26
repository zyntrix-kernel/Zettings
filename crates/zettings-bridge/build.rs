// SAFETY (cc_builder below): only appends a forced-include header; no
// cxx-qt-build-set flags are removed or overridden, so its codegen
// invariants hold. The allow is scoped to this function.
#[allow(unsafe_code)]
fn main() {
    let builder = unsafe {
        cxx_qt_build::CxxQtBuilder::new()
            .qt_module("Quick")
            .cc_builder(|cc| {
                cc.flag("-include");
                cc.flag("cxx-qt-lib/qstring.h");
            })
    };
    builder.file("src/app_info.rs").build();
}
