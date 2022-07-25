use futhark_bindgen::*;

struct Args {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    backend: Backend,
    #[allow(unused)]
    output_language: String,
    compiler: Option<String>,
    futhark_args: Vec<String>,
}

impl Args {
    fn parse(args: Vec<String>) -> Args {
        if &args[0] == "--help" || &args[0] == "-h" {
            usage();
            std::process::exit(0)
        }
        let input = std::path::PathBuf::from(&args[0]);
        let mut output = input.with_extension("rs");
        let mut backend = Backend::C;
        let mut output_language = String::from("rs");
        let mut compiler = None;
        let mut futhark_args = Vec::new();

        let mut skip = false;
        let mut in_futhark_args = false;
        for (i, arg) in args.iter().skip(1).enumerate() {
            if skip {
                skip = false;
                continue;
            }

            if in_futhark_args {
                futhark_args.push(arg.clone());
                continue;
            }

            match arg.as_str() {
                "--output" | "-o" => {
                    output = std::path::PathBuf::from(&args[i + 2]);
                    skip = true;
                }
                "--language" => {
                    output_language = String::from(&args[i + 2]);
                    output_language.make_ascii_lowercase();
                    output = input.with_extension(&output_language);
                    skip = true;
                }
                "--backend" => {
                    let mut b = args[i + 2].to_string();
                    b.make_ascii_lowercase();
                    backend = serde_json::from_str(&format!("\"{}\"", b)).expect("Invalid backend");
                    skip = true;
                }
                "--compiler" => {
                    compiler = Some(String::from(&args[i + 2]));
                    skip = true;
                }
                "--help" | "-h" => {
                    usage();
                    std::process::exit(0);
                }
                "--" => {
                    in_futhark_args = true;
                }
                s => {
                    panic!("Invalid argument: {}", s);
                }
            }
        }

        Args {
            input,
            output,
            backend,
            output_language,
            compiler,
            futhark_args,
        }
    }
}

fn usage() {
    eprintln!("futhark binding generator");
    eprintln!("usage: futhark-bindgen input.fut [--output filename] [--language language] [--backend futhark-backend] [--compiler path/to/futhark]");
    eprintln!("Supported languages: python, rust, ocaml");
    eprintln!("Supported backends: c, cuda, opencl, multicore, python, pyopencl");
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        return Ok(());
    }
    let args = Args::parse(args);

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
    let mut gen = config.detect().expect("Invalid output language");
    gen.generate(&lib, &mut config)?;

    Ok(())
}
