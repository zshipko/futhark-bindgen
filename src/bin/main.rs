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
struct Main {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Commands {
    Run(Run),
    Libs(Libs),
}

#[derive(Debug, FromArgs)]
#[argh(
    name = "libs",
    description = "List libraries for the selected backend",
    subcommand
)]

struct Libs {
    #[argh(
        option,
        default = "Backend::C",
        from_str_fn(parse_backend),
        description = "futhark backend: c, cuda, opencl, multicore, python, pyopencl"
    )]
    backend: Backend,
}

#[derive(Debug, FromArgs)]
#[argh(name = "run", description = "generate bindings", subcommand)]
struct Run {
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
    let args: Main = argh::from_env();

    match args.command {
        Commands::Run(args) => {
            let out_dir = args
                .output
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
                .to_path_buf();
            let mut compiler = Compiler::new(args.backend, &args.input)
                .with_extra_args(args.futhark_args)
                .with_output_dir(out_dir);
            if let Some(exe) = args.compiler {
                compiler = compiler.with_executable_name(exe);
            }
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
        }
        Commands::Libs(args) => {
            args.backend
                .required_c_libs()
                .iter()
                .for_each(|x| print!("-l{x} "));
            println!("");
        }
    }

    Ok(())
}
