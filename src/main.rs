use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;
use lang_interpreter::interpreter::Interpreter;
use lang_interpreter::interpreter::platform::{DefaultPlatformAPI, PlatformAPI};
use lang_interpreter::lexer::Lexer;
use lang_interpreter::parser::Parser;

fn main() -> ExitCode {
    let mut args = env::args();

    let binary_name = args.next();
    let binary_name = binary_name.as_deref();

    let args = args.collect::<Vec<String>>();

    if args.is_empty() {
        print_help(binary_name);

        return ExitCode::SUCCESS;
    }

    if !args[0].starts_with("-") || args[0] == "-e" || args[0].starts_with("--") || args[0].starts_with("-h") {
        if args[0].starts_with("-h") {
            print_help(binary_name);

            return ExitCode::SUCCESS;
        }

        if args[0].starts_with("--") {
            if args[0] != "--help" {
                eprintln!("Unknown COMMAND \"{}\"", args[0]);
            }

            print_help(binary_name);

            if args[0] != "--help" {
                return ExitCode::FAILURE;
            }

            return ExitCode::SUCCESS;
        }

        let lang_file_execution = args[0] != "-e";
        if !lang_file_execution && args.len() < 2 {
            eprintln!("CODE argument for \"-e\" is missing");

            print_help(binary_name);

            return ExitCode::FAILURE;
        }

        let execution_args_start_index = if lang_file_execution { 1 } else { 2 };
        let mut print_translations = false;
        let mut print_returned_value = false;
        let mut warnings = false;
        let mut lang_args = None;

        for (i, arg) in args[execution_args_start_index..].iter().
                map(|arg| &**arg).
                enumerate() {
            match arg {
                "-printTranslations" => print_translations = true,
                "-printReturnedValue" => print_returned_value = true,
                "-warnings" => warnings = true,
                "-langArgs" | "--" => {
                    lang_args = Some(args[execution_args_start_index + i + 1..].iter().map(|str| Box::from(&**str)).collect());
                    break;
                },
                _ => {
                    eprintln!("Unknown EXECUTION_ARG \"{}\"", arg);

                    print_help(binary_name);

                    return ExitCode::FAILURE;
                },
            }
        }

        return if lang_file_execution {
            execute_lang_file(&args[0], print_translations, print_returned_value, warnings, lang_args)
        }else {
            execute_lang_code(&args[1], print_translations, print_returned_value, warnings, lang_args)
        };
    }

    match &*args[0] {
        "-printTokens" => {
            if args.len() != 2 {
                eprintln!("\"printTokens\" requires exactly one file argument");

                print_help(binary_name);

                return ExitCode::FAILURE;
            }

            let file = File::open(&args[1]);
            let mut file = match file {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("FILE can not be read {e}");

                    return ExitCode::FAILURE;
                },
            };

            let mut code = Vec::new();
            let ret = file.read_to_end(&mut code);
            if let Err(e) = ret {
                eprintln!("FILE can not be read {e}");

                return ExitCode::FAILURE;
            };

            println!("{}", Lexer::new().read_tokens(String::from_utf8_lossy(&code)).iter().
                    map(ToString::to_string).
                    collect::<Vec<_>>().
                    join("\n"));

            ExitCode::SUCCESS
        },

        "-printAST" => {
            if args.len() != 2 {
                eprintln!("\"printAST\" requires exactly one file argument");

                print_help(binary_name);

                return ExitCode::FAILURE;
            }

            let file = File::open(&args[1]);
            let mut file = match file {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("FILE can not be read {e}");

                    return ExitCode::FAILURE;
                },
            };

            let mut code = Vec::new();
            let ret = file.read_to_end(&mut code);
            if let Err(e) = ret {
                eprintln!("FILE can not be read {e}");

                return ExitCode::FAILURE;
            };

            println!("{}", Parser::new().parse_lines(String::from_utf8_lossy(&code)).unwrap());

            ExitCode::SUCCESS
        },

        _ => {
            eprintln!("Unknown COMMAND \"{}\"", args[0]);

            print_help(binary_name);

            ExitCode::FAILURE
        },
    }
}

fn print_help(binary_name: Option<&str>) {
    let binary_name = binary_name.unwrap_or("lang");

    let header_str = format!("langRS version {} (lang-cli {})", Interpreter::VERSION, env!("CARGO_PKG_VERSION"));
    
    println!("{header_str}");
    println!("{}", "=".repeat(header_str.len()));
    println!("Interprets Lang code & files");
    println!();
    println!("Usage: {binary_name} COMMAND [ARGs]... | {binary_name} -e CODE [EXECUTION_ARGs]... [LANG_ARGs]... | {binary_name} FILE [EXECUTION_ARGs]... [LANG_ARGs]...");
    println!();
    println!("COMMANDs");
    println!("--------");
    println!("    -printAST FILE                   Prints the AST of a Lang file to standard output");
    println!("    -printTokens FILE                Prints the tokens of a Lang file to standard output");
    println!();
    println!("    -h, --help                        Prints this help page");
    println!();
    println!("IN-LINE CODE");
    println!("------------");
    println!("    -e CODE                           Executes CODE in the OS shell");
    println!();
    println!("EXECUTION_ARGs");
    println!("--------------");
    println!("    -printTranslations                Prints all Translations after the execution of the Lang file finished to standard output");
    println!("    -printReturnedValue               Prints the returned or thrown value of the Lang file if any");
    println!("    -warnings                         Enables the output of warnings which occur");
    println!("    -langArgs                         Indicates the start of the Lang args arguments (Everything after this argument will be interpreted as Lang args)");
    println!("    --                                Alias for \"-langArgs\"");
}

fn execute_lang_code(lang_code: &str, print_translations: bool, print_returned_value: bool, warnings: bool, lang_args: Option<Vec<Box<str>>>) -> ExitCode {
    let current_dir = env::current_dir().unwrap();

    let mut interpreter = Interpreter::new(
        current_dir.to_str().unwrap(),
        Some(""),
        None,
        Box::new(DefaultPlatformAPI::new()),
        lang_args,
    );

    if warnings {
        //TODO interpreter.setErrorOutputFlag(LangInterpreter.ExecutionFlags.ErrorOutputFlag.ALL);
    }

    interpreter.interpret_lines(lang_code);

    //TODO printPostExecutionOutput(interpreter, printTranslations, printReturnedValue);

    ExitCode::SUCCESS
}

fn execute_lang_file(lang_file: &str, print_translations: bool, print_returned_value: bool, warnings: bool, lang_args: Option<Vec<Box<str>>>) -> ExitCode {
    let file = File::open(lang_file);
    let mut file = match file {
        Ok(file) => file,
        Err(e) => {
            eprintln!("FILE can not be read {e}");

            return ExitCode::FAILURE;
        },
    };

    let mut code = Vec::new();
    let ret = file.read_to_end(&mut code);
    if let Err(e) = ret {
        eprintln!("FILE can not be read {e}");

        return ExitCode::FAILURE;
    };

    let lang_platform_api = DefaultPlatformAPI::new();

    let lang_file = Path::new(lang_file);
    let path = lang_platform_api.get_lang_path(lang_file).unwrap();
    let file_name = lang_platform_api.get_lang_file_name(lang_file).unwrap();

    let mut interpreter = Interpreter::new(
        &path.to_string_lossy(),
        Some(&file_name.to_string_lossy()),
        None,
        Box::new(DefaultPlatformAPI::new()),
        lang_args,
    );

    if warnings {
        //TODO interpreter.setErrorOutputFlag(LangInterpreter.ExecutionFlags.ErrorOutputFlag.ALL);
    }

    interpreter.interpret_lines(String::from_utf8_lossy(&code));

    //TODO printPostExecutionOutput(interpreter, printTranslations, printReturnedValue);

    ExitCode::SUCCESS
}