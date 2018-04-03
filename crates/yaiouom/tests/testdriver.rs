extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

    if let Ok(rustc) = std::env::var("RUSTC") {
        let mut path = PathBuf::new();
        path.push(rustc);
        config.rustc_path = path;
    }
    let is_refinement = config.rustc_path.ends_with("yaiouom-checker");
    eprintln!("Are we running with refinement? {}", is_refinement);

    config.mode = match mode {
        "compile-fail-vanilla" => compiletest::common::Mode::CompileFail,
        "compile-fail-refinement" if !is_refinement => return,
        "compile-fail-refinement" /* otherwise */ => compiletest::common::Mode::CompileFail,
        "run-fail-vanilla" if is_refinement => return,
        "run-fail-vanilla" /* otherwise */ => compiletest::common::Mode::RunFail,
        "run-pass" => compiletest::common::Mode::RunPass,
        _ => panic!("Invalid mode")
    };

    eprintln!("Running test suite {}", mode);
    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail-vanilla");
    run_mode("compile-fail-refinement");
    run_mode("run-fail-vanilla");
    run_mode("run-pass");
}
