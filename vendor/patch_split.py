p = "src/writer/rust/mod.rs"
s = open(p).read()

old_caller = """    // Add common includes for all objects
    cxx_mod_contents.insert(
        0,
        syn::parse2(signal_boilerplate()).expect("Could not build CXX common block"),
    );
"""
new_caller = """    // Patch note (Zettings): rustc >= 1.97 forbids macro invocations inside
    // unsafe extern blocks, so header-only includes are emitted as a plain
    // extern block separate from the unsafe block holding safe-to-call fns.
    cxx_mod_contents.insert(
        0,
        syn::parse2(signal_common_includes()).expect("Could not build CXX common includes"),
    );
    cxx_mod_contents.insert(
        1,
        syn::parse2(signal_boilerplate()).expect("Could not build CXX common block"),
    );
"""
assert old_caller in s, "caller drifted"
s = s.replace(old_caller, new_caller)

old_ins = """        cxx_mod_contents.insert(
            1,
            parse_quote_spanned! {cxx_mod.span() =>
                extern "C++" {"""
new_ins = """        cxx_mod_contents.insert(
            2,
            parse_quote_spanned! {cxx_mod.span() =>
                extern "C++" {"""
assert old_ins in s, "insert drifted"
s = s.replace(old_ins, new_ins)

old_fn = """fn signal_boilerplate() -> TokenStream {
    quote! {
        extern "C++" {
            include ! (< QtCore / QObject >);
            include!("cxx-qt/connection.h");
        }
        unsafe extern "C++" {"""
new_fn = """fn signal_common_includes() -> TokenStream {
    quote! {
        extern "C++" {
            include ! (< QtCore / QObject >);
            include!("cxx-qt/connection.h");
        }
    }
}

fn signal_boilerplate() -> TokenStream {
    quote! {
        unsafe extern "C++" {"""
assert old_fn in s, "fn shape drifted"
s = s.replace(old_fn, new_fn)

open(p, "w").write(s)
print("SPLIT_OK")
