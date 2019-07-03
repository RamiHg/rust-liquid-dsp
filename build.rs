use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write as _;
use std::mem;
use std::os::raw::{c_int, c_uint};
use std::path::{Path, PathBuf};

use serde::Deserialize;

type ResultBox<T> = std::result::Result<T, Box<std::error::Error>>;

const SIMD_OVERRIDE: bool = cfg!(feature = "simd-override");
const FFT_OVERRIDE: bool = cfg!(feature = "fft-override");

#[derive(Debug, Deserialize)]
struct MakeRules {
    modules: Vec<Module>,
    dotprod: HashMap<String, Module>,
}

#[derive(Debug, Deserialize)]
struct Module {
    name: String,
    files: Vec<PathBuf>,
    #[serde(default)]
    include_dirs: Vec<PathBuf>,
}

impl Module {
    pub fn compile(&self, config_dir: &Path) -> ResultBox<()> {
        let add_path_prefix = |p: &_| Path::new("./liquid-dsp").join(p);
        let mut cc = cc::Build::new();
        cc.files(self.files.iter().map(add_path_prefix));
        cc.include(&config_dir);
        cc.include("./liquid-dsp/include");
        for dir in self.include_dirs.iter() {
            cc.include(&add_path_prefix(&dir));
        }
        cc.warnings(false);
        // For some reason this still shows up even with warnings off?
        cc.flag("-Wno-format-extra-args");
        cc.compile(&self.name);
        Ok(())
    }
}

fn create_config_h(config_dir: &PathBuf) -> ResultBox<()> {
    let mut f = fs::File::create(config_dir.join("config.h"))?;
    writeln!(f, "#ifndef __LIQUID_CONFIG_H__")?;
    writeln!(f, "#define __LIQUID_CONFIG_H__")?;
    writeln!(f, "#define SIZEOF_INT ({})", mem::size_of::<c_int>())?;
    writeln!(
        f,
        "#define SIZEOF_UNSIGNED_INT ({})",
        mem::size_of::<c_uint>()
    )?;
    if SIMD_OVERRIDE {
        writeln!(f, "#define LIQUID_SIMDOVERRIDE 1")?;
    }
    if FFT_OVERRIDE {
        writeln!(f, "#define LIQUID_FFTOVERRIDE 1")?;
    }
    // Write the SSE defines.
    let mut define_if = |feature, cond| {
        if cond {
            writeln!(f, "#define {} 1", feature).unwrap();
        }
    };
    let mut define_sse = |feature, header, cond| {
        define_if(feature, cond);
        define_if(header, cond);
    };
    define_sse("HAVE_MMX", "HAVE_MMINTRIN_H", cfg!(target_feature = "fxsr"));
    define_sse("HAVE_SSE", "HAVE_XMMINTRIN_H", cfg!(target_feature = "sse"));
    define_sse(
        "HAVE_SSE2",
        "HAVE_EMMINTRIN_H",
        cfg!(target_feature = "sse2"),
    );
    define_sse(
        "HAVE_SSE3",
        "HAVE_PMMINTRIN_H",
        cfg!(target_feature = "sse3"),
    );

    writeln!(f, "#endif")?;
    Ok(())
}

fn get_modules_to_compile() -> HashMap<String, bool> {
    let mut modules = HashMap::new();
    // Insert required dependencies.
    modules.insert("buffer".to_string(), true);
    modules.insert("dotprod".to_string(), true);
    modules.insert("math".to_string(), true);
    modules.insert("utility".to_string(), true);
    // And now optional modules.
    modules.insert("filter".to_string(), cfg!(feature = "filter"));
    modules
}

fn main() -> ResultBox<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let config_dir = out_dir.join("include");
    // Create the include directory needed for the build.
    fs::create_dir_all(&config_dir)?;
    // Create the config.h file.
    create_config_h(&config_dir)?;
    // Read the make rules.
    let rules: MakeRules = toml::from_str(&fs::read_to_string("make_rules.toml")?)?;
    let modules_to_compile = get_modules_to_compile();
    for module in rules.modules {
        if modules_to_compile.contains_key(&module.name) {
            module.compile(&config_dir)?;
        }
    }
    // Compile either the portable or SSE dotprod depending on platform and override.
    if !SIMD_OVERRIDE && cfg!(target_feature = "sse") {
        &rules.dotprod["sse"]
    } else {
        &rules.dotprod["portable"]
    }
    .compile(&config_dir)?;
    // Create the bindgens.
    let bindings = bindgen::Builder::default()
        .header("./liquid-dsp/include/liquid.h")
        .whitelist_function("msresamp_rrrf.*")
        .generate_comments(true)
        .generate()
        .expect("Unable to generate bindings.");
    bindings.write_to_file(out_dir.join("bindings.rs"))?;
    Ok(())
}
