use std::env;
use std::path::Path;
use npm_rs::*;

fn main() -> Result<(), std::io::Error>{
    //let out_dir = env::var_os("OUT_DIR").unwrap();
    //let dest_path = Path::new(&out_dir).join("style.css");

    let exit_status = NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .init_env()
        .install(None)
        .run("build_css")
        .exec()?;
    if !exit_status.success() { panic!("NPM postcss build failed!"); }

    println!("cargo:rerun-if-changed=src/style.css");
    Ok(())
}