#![feature(rustc_private)]

extern crate getopts;

extern crate rustc;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_trans_utils;
extern crate syntax;

mod dimanalysis;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use rustc::ty::{ TypeckTables, TyCtxt };
use rustc::hir::def_id::DefId;
use rustc_driver::*;

fn typeck_tables_of<'a, 'tcx>(ctx: TyCtxt<'a, 'tcx, 'tcx>, id: DefId) -> &'tcx TypeckTables<'tcx> {
    // First, run regular type inference, i.e. the default Providers.typeck_tables_of(ctx, id).
    let mut providers = rustc::ty::maps::Providers::default();
    rustc_driver::driver::default_provide(&mut providers);
    let tables = (providers.typeck_tables_of)(ctx, id);

    let mut analyzer = dimanalysis::DimAnalyzer::new(ctx, tables, id);
    analyzer.analyze();

    tables
}

/// Compiler callbacks.
///
/// Extends compiler behavior with a single type-checking pass.
struct Callbacks {
    default: RustcDefaultCalls,
}
impl Callbacks {
    fn new() -> Self {
        Callbacks {
            default: RustcDefaultCalls
        }
    }
}
impl<'a> rustc_driver::CompilerCalls<'a> for Callbacks {
    fn early_callback(&mut self, matches: &getopts::Matches, sopts: &rustc::session::config::Options, cfg: &syntax::ast::CrateConfig, descriptions: &rustc_errors::registry::Registry, output: rustc::session::config::ErrorOutputType) -> Compilation {
        self.default.early_callback(matches, sopts, cfg, descriptions, output)
    }

    fn no_input(&mut self, matches: &getopts::Matches, sopts: &rustc::session::config::Options, cfg: &syntax::ast::CrateConfig, odir: &Option<PathBuf>, ofile: &Option<PathBuf>, descriptions: &rustc_errors::registry::Registry) -> Option<(rustc::session::config::Input, Option<PathBuf>)> {
        self.default.no_input(matches, sopts, cfg, odir, ofile, descriptions)
    }

    fn late_callback(&mut self, trans_crate: &rustc_trans_utils::trans_crate::TransCrate, matches: &getopts::Matches, sess: &rustc::session::Session, crate_stores: &rustc::middle::cstore::CrateStore, input: &rustc::session::config::Input, odir: &Option<PathBuf>, ofile: &Option<PathBuf>) -> Compilation {
        self.default .late_callback(trans_crate, matches, sess, crate_stores, input, odir, ofile)
    }

    fn build_controller(&mut self, sess: &rustc::session::Session, matches: &getopts::Matches) -> driver::CompileController<'a> {
        let mut controller = self.default.build_controller(sess, matches);
        // Extract `controller.provide` to `old_provide`, replace it with a placeholder.
        let old_provide = std::mem::replace(&mut controller.provide, Box::new(|_| {}));
        let provide : Box<for<'r, 's> std::ops::Fn(&'r mut rustc::ty::maps::Providers<'s>)> = Box::new(move |providers| {
            old_provide(providers);
            // There doesn't seem to be any good way to save the old `typeck_tables_of` provider,
            // so we'll just call it manually from our own `typeck_tables_of`.
            providers.typeck_tables_of = typeck_tables_of;
        });
        controller.provide = provide;
        controller
    }
}

pub fn main() {
    // The following is copied straight from Clippy.
    let sys_root = option_env!("SYSROOT")
        .map(String::from)
        .or_else(|| std::env::var("SYSROOT").ok())
        .or_else(|| {
            let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
            let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
            home.and_then(|home| toolchain.map(|toolchain| format!("{}/toolchains/{}", home, toolchain)))
        })
        .or_else(|| {
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_owned())
        })
        .expect("need to specify SYSROOT env var during clippy compilation, or use rustup or multirust");

    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    let mut orig_args: Vec<String> = env::args().collect();
    if orig_args.len() <= 1 {
        std::process::exit(1);
    }
    if orig_args[1] == "rustc" {
        // we still want to be able to invoke it normally though
        orig_args.remove(1);
    }
    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    let args: Vec<String> = if orig_args.iter().any(|s| s == "--sysroot") {
        orig_args.clone()
    } else {
        orig_args
            .clone()
            .into_iter()
            .chain(Some("--sysroot".to_owned()))
            .chain(Some(sys_root))
            .collect()
    };

    let mut callbacks = Callbacks::new();
    rustc_driver::run(move || {
        rustc_driver::run_compiler(&args, &mut callbacks, None, None)
    });
}