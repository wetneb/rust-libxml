use std::{env, fs, path::{Path, PathBuf}};

/// Finds libxml2 and optionally return a list of header
/// files from which the bindings can be generated.
fn find_libxml2() -> Option<Vec<PathBuf>> {
  #![allow(unreachable_code)] // for platform-dependent dead code

  if let Ok(ref s) = std::env::var("LIBXML2") {
    // println!("{:?}", std::env::vars());
    // panic!("set libxml2.");
    let p = std::path::Path::new(s);
    let fname = std::path::Path::new(
      p.file_name()
        .unwrap_or_else(|| panic!("no file name in LIBXML2 env ({s})")),
    );
    assert!(
      p.is_file(),
      "{}",
      &format!("not a file in LIBXML2 env ({s})")
    );
    println!(
      "cargo:rustc-link-lib={}",
      fname
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .strip_prefix("lib")
        .unwrap()
    );
    println!(
      "cargo:rustc-link-search={}",
      p.parent()
        .expect("no library path in LIBXML2 env")
        .to_string_lossy()
    );
    None
  } else {    
    #[cfg(any(target_family = "unix", target_os = "macos"))]
    {
      let env_var = std::env::var("BINDGEN_EXTRA_CLANG_ARGS_ARMHF").unwrap_or("undefined".to_owned());
      println!("BINDGEN_EXTRA_CLANG_ARGS_ARMHF: {env_var}");
      let env_var = std::env::var("BINDGEN_EXTRA_CLANG_ARGS_armhf").unwrap_or("undefined".to_owned());
      println!("BINDGEN_EXTRA_CLANG_ARGS_armhf: {env_var}");
      let libxml = pkg_config::Config::new()
        .probe("libxml-2.0")
        .expect("Couldn't find libxml2 via pkg-config");
      let libicu = pkg_config::Config::new()
        .probe("icu-uc")
        .expect("Couldn't find icu-uc via pkg-config");
      return Some([
        libxml.include_paths,
       // libicu.include_paths,
      ].into_iter().flatten().collect())
    }

    #[cfg(windows)]
    {
      if vcpkg_dep::find() {
        return None
      }
    }
    
    panic!("Could not find libxml2.")
  }
}

fn generate_bindings(header_dirs: Vec<PathBuf>, output_path: &Path) {
  std::env::set_var("PKG_CONFIG_ALLOW_SYSTEM_CFLAGS", "1");
  let bindings = bindgen::Builder::default()
    .header("src/wrapper.h")
    // invalidate build as soon as the wrapper changes
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .layout_tests(true)
    // .clang_args(&["-DPKG-CONFIG"])
    // .clang_arg("-I/usr/lib/llvm-6.0/lib/clang/6.0.0/include/")
    .clang_args(
      header_dirs.iter()
        .map(|dir| format!("-I{}", dir.display()))
    );
  println!("full bindgen invocation: {}", bindings.command_line_flags().join(" "));
  bindings
    .generate()
    .expect("failed to generate bindings with bindgen")
    .write_to_file(output_path)
    .expect("Failed to write bindings.rs");
}

fn main() {
  let bindings_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs");
  if let Some(header_dirs) = find_libxml2() {
    // if we could find header files, generate fresh bindings from them
    println!("all include paths: {}", header_dirs.iter().map(|dir| format!("{}", dir.display())).collect::<Vec<_>>().join(" "));
    generate_bindings(header_dirs, &bindings_path);
  } else {
    // otherwise, use the default bindings on platforms where pkg-config isn't available
    fs::copy(PathBuf::from("src/default_bindings.rs"), bindings_path)
      .expect("Failed to copy the default bindings to the build directory");
  }
}

#[cfg(target_family = "windows")]
mod vcpkg_dep {
  pub fn find() -> bool {
    if vcpkg::find_package("libxml2").is_ok() {
      return true;
    }
    false
  }
}
