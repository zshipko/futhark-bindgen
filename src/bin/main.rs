use futhark_bindgen::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("USAGE: futhark-generate myfile.fut output.rs");
        return Ok(());
    }

    let compiler = Compiler::new(Backend::C, &args[0]);
    let lib = compiler.compile()?;

    let output = lib.c_file.with_extension("rs");
    let mut config = Config::new(output)?;
    let mut gen = config.detect().expect("Invalid output language");
    gen.generate(&lib, &mut config)?;

    Ok(())
}
