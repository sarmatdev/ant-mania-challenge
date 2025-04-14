use std::fs;

use ant_mania::Hiveum;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Hiveum Ant Mania")]
#[command(about = "Runs a multi-ant simulation through colony networks")]
struct Args {
    #[arg(short, long)]
    map: String,

    #[arg(short, long)]
    ants: usize,

    #[arg(short, long)]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();

    println!(
        "Starting simulation with {} ants on map '{}'",
        args.ants, args.map
    );

    let seed = args.seed.unwrap_or(1970);
    let mut hiveum = Hiveum::new(&args.map, args.ants, seed);
    let simulation_time = hiveum.simulate();
    let hive_status = hiveum.hive_status();

    println!("Simulation time: {:?}", simulation_time);

    fs::write(
        "hive_status.log",
        format!(
            "Seed: {}\nAnts: {}\nMap: {}\nSimulation time: {:?}\n\nHive status:\n{}",
            seed, args.ants, args.map, simulation_time, hive_status
        ),
    ).expect("Failed to write hive status log");
}
