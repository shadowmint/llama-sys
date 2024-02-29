use cmake::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    // Configure
    let mut config = Config::new("external/llama.cpp");
    config
        .build_target("preinstall")
        .define("LLAMA_STATIC", "ON")
        .define("LLAMA_STANDALONE", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF")
        .define("LLAMA_BUILD_TESTS", "OFF");

    #[cfg(target_os = "macos")]
    {
        config
            .define("LLAMA_METAL", "ON")
            .define("LLAMA_ACCELERATE", "true")
            .define("LLAMA_METAL_EMBED_LIBRARY", "ON");
    }

    #[cfg(feature = "cuda")]
    config.define("LLAMA_CUDA", "ON");

    #[cfg(feature = "cuda_f16")]
    config.define("LLAMA_CUDA_FP16", "ON");

    #[cfg(feature = "native")]
    config.define("LLAMA_NATIVE", "ON");

    // Build
    let dst = config.very_verbose(true).build();

    // Link
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=llama");

    // FIXME: Figure out how to make linux use clang. We may have to define
    // the CC environment variable before running the `.build()` method above
    // or the API may have a way to set the compiler.
    #[cfg(not(target_os = "linux"))]
    println!("cargo:rustc-link-lib=dylib=c++");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalKit");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    }

    #[cfg(feature = "cuda")]
    {
        // FIXME: Cublas isn't necessarily used by llama.cpp, but it is used if
        // it is found. It might be better to make this a feature flag and
        // manually set the associated cmake flags.
        println!("cargo:rustc-link-lib=dylib=cublas");
        println!("cargo:rustc-link-lib=dylib=cudart");
        println!("cargo:rustc-link-lib=dylib=cuda");
    }

    println!("cargo:rerun-if-changed=external/llama.cpp/*.h");
    println!("cargo:rerun-if-changed=external/llama.cpp/*.c");
    println!("cargo:rerun-if-changed=external/llama.cpp/*.cpp");

    let bindings = bindgen::Builder::default()
        .header("external/llama.cpp/llama.h")
        .allowlist_function("llama_.*")
        .allowlist_type("llama_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}
