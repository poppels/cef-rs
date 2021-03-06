#![deny(warnings)]

//! This build script was based on the original cef-rs/cef-sys crate.
//! This crate is quite different from the original and as such a new name
//! cef_ffi instead of cef-sys seemed appropriate.
//!
//! The build script has since been modified to run bindgen as part of this
//! pre-build script, as recommended by the bindgen manual.
//!
//! The original script used a environment variable pointing to the folder
//! containing the cef shared library file. This updated build script expects a
//! different environment variable, CEF_DIST_ROOT, to point to the root folder
//! of an extracted binary distribution of cef, in order to allow access to both
//! header files (for bindgen) as well as the pre-built shared library file (for
//! linking).
//!
//! TODO: Allow a default location for the cef distribution?
//!         - What would be a good default?
//!
//! TODO: Allow download of cef distribution archive if missing?
//!       (http://opensource.spotify.com/cefbuilds/index.html)
//!         - Pick version and verify checksum. (specified by the user of this
//!           crate?)
//!         - Enable by cargo feature?
//!         - Make compatible with cargo vendor?
//!         - Path to download and extract to?
//!         - Is there a "cargo way" of doing this?
//!
//! TODO: Investigate if build.rs scripts should assist in packaging the final application.
//!         - Generate list of files needed (src, dst) and pass on down the build pipe?

extern crate bindgen;

use std::env;
use std::path::Path;
use std::path::PathBuf;

enum Platform {
    Windows,
    Mac,
    Linux,
}

fn get_platform() -> Platform {
    match std::env::var("TARGET")
              .unwrap()
              .split('-')
              .nth(2)
              .unwrap() {
        "win32" | "windows" => Platform::Windows,
        "darwin" => Platform::Mac,
        "linux" => Platform::Linux,
        other => panic!("Sorry, platform \"{}\" is not supported by CEF.", other),
    }
}

/// Check if path exists, crash on failure. Implement as trait :)
fn assert_path(p: &Path) {
    if !p.exists() {
        panic!("Unable to find path: {}", p.to_str().unwrap());
    }
}

fn main() {
    use std::path::Path;
    let var_name = "CEF_DIST_ROOT";
    let cef_root = std::env::var(var_name)
        .expect(format!("{} needs to point to the folder containing \
                        an extracted CEF distribution archive.  \
                        You can get one here: \
                        http://opensource.spotify.com/cefbuilds/index.html",
                        var_name)
                        .as_str());
    let cef_root = Path::new(&cef_root);
    let release = cef_root.join("Release");
    assert_path(&cef_root);
    assert_path(&release);
    match get_platform() {
        Platform::Mac => {
            println!("cargo:rustc-link-lib=framework=Chromium Embedded Framework"); // seems to be ignored, file bug report.
            println!("cargo:rustc-link-search=framework={}",
                     release.to_str().unwrap());
        }
        Platform::Windows => {
            println!("cargo:rustc-link-lib=libcef");
            println!("cargo:rustc-link-search={}", release.to_str().unwrap());
        }
        Platform::Linux => {
            println!("cargo:rustc-link-lib=cef");
            println!("cargo:rustc-link-search={}", release.to_str().unwrap());
        }
    }
    let bindings = bindgen::Builder::default()
        .no_unstable_rust()
        .derive_debug(false)
        .clang_arg(format!("-I{}", cef_root.to_str().unwrap()))
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
