use futhark_generate::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("USAGE: futhark-generate myfile.fut output.rs");
        return Ok(());
    }

    let compiler = Compiler::new(Backend::C, &args[0]);
    let lib = compiler.compile()?;

    println!("{:?}", lib.manifest);
    lib.manifest.print_c_functions();
    Ok(())
}
