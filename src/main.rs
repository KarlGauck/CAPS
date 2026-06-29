use clap::{Parser, Subcommand};

mod assignments;
mod utils;

use crate::assignments::*;

#[derive(Parser)]
#[command(
    name = "assignments",
    about = "Run assignment exercises.\nWhen running from cargo, execute 'cargo run -- <ASSIGNMENT> <EX>', e.g. 'cargo run -- a6 ex2'.\nTo get the available exercises for an assignment set, call 'cargo run -- <ASSIGNMENT> --help'"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    A1 {
        #[arg(value_enum)]
        ex: A1Ex,
    },
    A2 {
        #[arg(value_enum)]
        ex: A2Ex,
    },
    A3 {
        #[arg(value_enum)]
        ex: A3Ex,
    },
    A3Cms,
    A4,
    A5 {
        #[arg(value_enum)]
        ex: A5Ex,
    },
    A6 {
        #[arg(value_enum)]
        ex: A6Ex,
    },
    A8 {
        #[arg(value_enum)]
        ex: A8Ex,
    },
    RenderTest,
}

#[derive(clap::ValueEnum, Clone)]
enum A1Ex {
    Ex1,
    Ex2,
    Ex3,
    Ex4,
}
#[derive(clap::ValueEnum, Clone)]
enum A2Ex {
    Ex1,
    Ex3,
}
#[derive(clap::ValueEnum, Clone)]
enum A3Ex {
    Ex1,
    Ex2,
}
#[derive(clap::ValueEnum, Clone)]
enum A5Ex {
    Ex2_1,
    Ex2_2a,
    Ex2_2b,
}
#[derive(clap::ValueEnum, Clone)]
enum A6Ex {
    RenderPath,
    Ex2,
}

#[derive(clap::ValueEnum, Clone)]
enum A8Ex {
    Ex1,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::A1 { ex } => match ex {
            A1Ex::Ex1 => assignment1::ex1("solutions/01/img"),
            A1Ex::Ex2 => assignment1::ex2(),
            A1Ex::Ex3 => assignment1::ex3(),
            A1Ex::Ex4 => assignment1::ex4(),
        },
        Commands::A2 { ex } => match ex {
            A2Ex::Ex1 => assignment2::ex1(),
            A2Ex::Ex3 => assignment2::ex3(),
        },
        Commands::A3 { ex } => match ex {
            A3Ex::Ex1 => assignment3::ex1(),
            A3Ex::Ex2 => assignment3::ex2(),
        },
        Commands::A3Cms => assignment3_cms::ex1(),
        Commands::A4 => assignment4::ex2(),
        Commands::A5 { ex } => match ex {
            A5Ex::Ex2_1 => assignment5::ex21(),
            A5Ex::Ex2_2a => assignment5::ex2_2a(),
            A5Ex::Ex2_2b => assignment5::ex2_2b(),
        },
        Commands::A6 { ex } => match ex {
            A6Ex::RenderPath => assignment6::render_path(),
            A6Ex::Ex2 => assignment6::ex2(),
        },
        Commands::A8 { ex } => match ex {
            A8Ex::Ex1 => assignment8::ex1(),
        }
        Commands::RenderTest => render_test::run(),
    }

    println!("done");
}
