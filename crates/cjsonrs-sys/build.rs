fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cjson_src_path = std::env::var("CJSON_SRC_PATH").unwrap_or("cJSON".to_string());
    let cjson_include_path = std::env::var("CJSON_INCLUDE_PATH").unwrap_or("cJSON".to_string());
    let cjson_lib_path = std::env::var("CJSON_LIB_PATH");
    let vendored = std::env::var("CARGO_FEATURE_VENDORED");
    let std = std::env::var("CARGO_FEATURE_STD");
    let kind = if vendored.is_ok() { "static" } else { "dylib" };

    println!("cargo::metadata=KIND={kind}");
    println!("cargo::metadata=CJSON_INCLUDE_PATH={cjson_include_path}");

    if let Ok(cjson_lib_path) = cjson_lib_path {
        println!("cargo::metadata=CJSON_LIB_PATH={cjson_lib_path}");
        println!("cargo::rustc-link-search=native={}", cjson_lib_path);
        println!("cargo::rustc-link-lib={kind}=cJSON");
    } else if vendored.is_ok() {
        println!("cargo::metadata=CJSON_LIB_PATH={cjson_src_path}");
        cc::Build::new()
            .file(format!("{}/cJSON.c", cjson_src_path))
            .file(format!("{}/cJSON_Utils.c", cjson_src_path))
            .files(glob::glob(&format!("{cjson_src_path}/cJSON*.c"))?.flatten())
            .static_flag(true)
            .include(&cjson_include_path)
            .try_compile("cJSON")?;
    }

    let mut builder = bindgen::Builder::default()
        .header(format!("{}/cJSON_Utils.h", cjson_include_path))
        .header(format!("{}/cJSON.h", cjson_include_path))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true);

    if std.is_err() {
        builder = builder.use_core();
    }

    builder
        .generate()?
        .write_to_file(std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("bindings.rs"))?;

    Ok(())
}
