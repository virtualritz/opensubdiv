#![allow(unreachable_code)]
use std::path::PathBuf;

static MAC_OS_BREW_CLANG_PATH: &str = "/usr/local/opt/llvm/bin";

pub fn main() {
    #[cfg(all(target_os = "macos", feature = "cuda"))]
    panic!("The feature `cuda` is not available on macOS.");

    #[cfg(all(not(target_os = "macos"), feature = "metal"))]
    panic!("The feature `metal` is only available on macOS.");

    let glfw = cmake::Config::new("dependencies/glfw")
        .always_configure(false)
        .build();

    // FIXME: check if Homebrew Clang is installed in
    // /usr/local/Cellar/llvm/11.1.0/bin

    let mut open_subdiv = cmake::Config::new("dependencies/OpenSubdiv");

    open_subdiv
        .define("NO_PTEX", "1")
        .define("NO_DOC", "1")
        .define("NO_OPENCL", "1")
        .define("NO_CLEW", "1")
        .define("NO_TBB", "1")
        .define("GLFW_LOCATION", glfw);

    #[cfg(any(target_os = "macos", not(feature = "cuda")))]
    open_subdiv.define("NO_CUDA", "1");

    #[cfg(any(not(target_os = "macos"), not(feature = "metal")))]
    open_subdiv.define("NO_METAL", "1");

    #[cfg(target_os = "macos")]
    {
        // We try to use Homebrew's Clang for building; not Apple Clang.
        // This allows us to build with OpenMP support.
        let clang = PathBuf::from(MAC_OS_BREW_CLANG_PATH).join("clang");
        let clang_pp = PathBuf::from(MAC_OS_BREW_CLANG_PATH).join("clang++");

        if clang.exists() && clang_pp.exists() {
            open_subdiv
                .define(
                    "CMAKE_C_COMPILER",
                    clang,
                )
                .define(
                    "CMAKE_CXX_COMPILER",
                    clang_pp,
                );
        }
        else {
            // No clang installed via Homebrew – we can't build with OpenMP
            // support on macOS as Apple's Clang has no support for it.
            open_subdiv.define("NO_OMP", "1");
        }
    }

    let open_subdiv = open_subdiv.build();

    //println!("cargo:rustc-link-search=native={}",
    // open_subdiv.join("lib").display()); println!("cargo:
    // rustc-link-lib=static=osdCPU");

    let osd_inlude_path = open_subdiv.join("include");
    let osd_lib_path = open_subdiv.join("lib");

    let mut dst_capi = cmake::Config::new("osd-capi");

    dst_capi.define("OSD_INCLUDE_PATH", &osd_inlude_path);

    #[cfg(any(target_os = "macos", not(feature = "cuda")))]
    dst_capi.define("NO_CUDA", "1");

    let dst_capi = dst_capi.build();

    println!("cargo:rustc-link-search=native={}", dst_capi.display());
    println!("cargo:rustc-link-lib=static=osd-capi");

    println!("cargo:rustc-link-search=native={}", osd_lib_path.display());
    println!("cargo:rustc-link-lib=static=osdCPU");

    #[cfg(feature = "cuda")]
    println!("cargo:rustc-link-lib=static=osdGPU");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");
}
