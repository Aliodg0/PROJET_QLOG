use prog64::{TrainingConfig, save_program, save_report, train};
use std::env;
use std::process;

fn print_help() {
    println!(
        "prog64 - train a 64-bit linear genetic program on CartPole\n\n\
         OPTIONS:\n\
           --population <n>     Population size (default: 120)\n\
           --generations <n>    Number of generations (default: 40)\n\
           --survival <f>       Survival rate (default: 0.2)\n\
           --mutation <f>       Bit mutation rate (default: 0.02)\n\
           --episodes <n>       Episodes per evaluation (default: 5)\n\
           --threads <n>        Worker threads (default: 1)\n\
           --seed <u64>         Random seed (default: 1)\n\
           --early-stop <f>     Stop when score reaches this value\n\
           --no-early-stop      Disable early stopping\n\
           --output <path>      Save the best program as JSON\n\
           --help               Show this help"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config = TrainingConfig::default();
    let mut output = None;
    let mut report_path = None;
    let mut i = 1usize;

    while i < args.len() {
        match args[i].as_str() {
            "--population" => {
                config.population_size = parse(&args, &mut i, "--population");
            }
            "--generations" => {
                config.generations = parse(&args, &mut i, "--generations");
            }
            "--survival" => {
                config.survival_rate = parse(&args, &mut i, "--survival");
            }
            "--mutation" => {
                config.mutation_rate = parse(&args, &mut i, "--mutation");
            }
            "--episodes" => {
                config.episodes = parse(&args, &mut i, "--episodes");
            }
            "--threads" => {
                config.threads = parse(&args, &mut i, "--threads");
            }
            "--seed" => {
                config.seed = parse(&args, &mut i, "--seed");
            }
            "--early-stop" => {
                config.early_stop_score = Some(parse(&args, &mut i, "--early-stop"));
            }
            "--no-early-stop" => {
                config.early_stop_score = None;
                i += 1;
            }
            "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("missing value for --output");
                    process::exit(2);
                }
                output = Some(args[i + 1].clone());
                i += 2;
            }
            "--report" => {
                if i + 1 >= args.len() {
                    eprintln!("missing value for --report");
                    process::exit(2);
                }
                report_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                return;
            }
            unknown => {
                eprintln!("unknown argument: {unknown}");
                print_help();
                process::exit(2);
            }
        }
    }

    let report = train(config);
    println!("best_score={}", report.best_score);
    println!("best_program={}", report.best_program.0);
    println!("generations={}", report.generation_best.len());
    println!(
        "unique_programs={}",
        report.unique_programs.last().copied().unwrap_or(0)
    );
    println!(
        "bit_diversity={}",
        report.bit_diversity.last().copied().unwrap_or(0.0)
    );

    if let Some(path) = output {
        match save_program(&report.best_program, &path) {
            Ok(()) => println!("saved_program={path}"),
            Err(error) => {
                eprintln!("could not save program: {error}");
                process::exit(1);
            }
        }
    }

    if let Some(path) = report_path {
        match save_report(&report, &path) {
            Ok(()) => println!("saved_report={path}"),
            Err(error) => {
                eprintln!("could not save report: {error}");
                process::exit(1);
            }
        }
    }
}

fn parse<T>(args: &[String], index: &mut usize, name: &str) -> T
where
    T: std::str::FromStr,
{
    if *index + 1 >= args.len() {
        eprintln!("missing value for {name}");
        process::exit(2);
    }
    *index += 1;
    match args[*index].parse::<T>() {
        Ok(value) => {
            *index += 1;
            value
        }
        Err(_) => {
            eprintln!("invalid value for {name}");
            process::exit(2);
        }
    }
}
