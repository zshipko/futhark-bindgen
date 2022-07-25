use futhark_bindgen::*;

struct Args {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    backend: Backend,
    #[allow(unused)]
    output_language: String,
    compiler: Option<String>,
}

impl Args {
    fn parse(args: Vec<String>) -> Args {
        let input = std::path::PathBuf::from(&args[0]);
        let mut output = input.with_extension("rs");
        let mut backend = Backend::C;
        let mut output_language = String::from("rs");
        let mut compiler = None;

        let mut skip = false;
        for (i, arg) in args.iter().skip(1).enumerate() {
            if skip {
                skip = false;
                continue;
            }

            match arg.as_str() {
                "--output" | "-o" => {
                    output = std::path::PathBuf::from(&args[i + 2]);
                    skip = true;
                }
                "--language" => {
                    output_language = String::from(&args[i + 2]);
                    output = input.with_extension(&output_language);
                    skip = true;
                }
                "--backend" => {
                    backend = serde_json::from_str(&format!("\"{}\"", &args[i + 2]))
                        .expect("Invalid backend");
                    skip = true;
                }
                "--compiler" => {
                    compiler = Some(String::from(&args[i + 2]));
                    skip = true;
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
        }
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("USAGE: futhark-generate myfile.fut output.ext");
        return Ok(());
    }
    let args = Args::parse(args);

    let mut compiler = Compiler::new(args.backend, args.input);
    if let Some(exe) = args.compiler {
        compiler.set_executable_name(exe);
    }
    let lib = compiler.compile()?;

    let mut config = Config::new(args.output)?;
    let mut gen = config.detect().expect("Invalid output language");
    gen.generate(&lib, &mut config)?;

    Ok(())
}
