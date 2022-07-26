use futhark_bindgen::*;

use argh::FromArgs;

fn parse_backend(s: &str) -> Result<Backend, String> {
    let mut s = s.to_string();
    s.make_ascii_lowercase();
    let x = serde_json::from_str(&format!("\"{}\"", s)).expect("Invalid backend");
    Ok(x)
}

#[derive(Debug, FromArgs)]
#[argh(description = "futhark binding generator")]
struct Args {
    #[argh(positional, description = "futhark input file")]
    input: std::path::PathBuf,

    #[argh(positional, description = "output file")]
    output: std::path::PathBuf,

    #[argh(
        option,
        default = "Backend::C",
        from_str_fn(parse_backend),
        description = "futhark backend: c, cuda, opencl, multicore, python, pyopencl"
    )]
    backend: Backend,

    #[argh(option, description = "path to futhark compiler")]
    compiler: Option<String>,

    #[argh(
        option,
        long = "futhark-arg",
        short = 'f',
        description = "arguments to be passed to the futhark compiler"
    )]
    futhark_args: Vec<String>,
}

fn main() -> Result<(), Error> {
    let args: Args = argh::from_env();

    let mut compiler = Compiler::new(args.backend, &args.input);
    if let Some(exe) = args.compiler {
        compiler.set_executable_name(exe);
    }
    compiler.set_extra_args(args.futhark_args);
    let lib = match compiler.compile()? {
        Some(l) => l,
        None => {
            let py_file = args.input.with_extension("py");
            if args.output != py_file {
                std::fs::copy(py_file, args.output)?;
            }
            return Ok(());
        }
    };

    let mut config = Config::new(args.output)?;
    let mut gen = config.detect().expect("Unable to detect output language");
    gen.generate(&lib, &mut config)?;

    Ok(())
}
