use std::{
    env,
    path::PathBuf,
};

use bindgen::CargoCallbacks;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Buscar librga mediante pkg-config.
    let library = pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("librga")
        .expect("Unable to locate librga via pkg-config.");

    // Configurar bindgen.
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")

        // Regenerar bindings cuando cambien los headers.
        .parse_callbacks(Box::new(CargoCallbacks::new()))

        // Calidad de los bindings.
        .generate_comments(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .layout_tests(false)

        // Mejor representación de tipos C.
        .size_t_is_usize(true)

        // Rust moderno.
        .rustified_enum("*")

        // Mantener documentación.
        .generate_cstr(true);

    // Añadir automáticamente todos los include paths.
    for include in &library.include_paths {
        builder = builder.clang_arg(format!("-I{}", include.display()));
    }

    // --------------------------------------------------------
    // OPCIONAL
    //
    // Cuando quieras reducir muchísimo el tamaño de los bindings,
    // simplemente descomenta estas líneas.
    //
    // builder = builder
    //     .allowlist_function("^im.*")
    //     .allowlist_type("^rga.*")
    //     .allowlist_var("^(IM_|RK_).*");
    // --------------------------------------------------------

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to write bindings.");
}