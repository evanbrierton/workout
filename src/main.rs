use std::collections::HashMap;

use anyhow::Ok;
use clap::Parser;
use itertools::Itertools;
use workout_rs::{
    bar::Bar, bar_kind::BarKind, gym::Gym, plate::Plate,
    requirement::Requirement,
};

#[derive(Parser)]
struct Args {
    #[arg(value_parser = clap::value_parser!(Requirement))]
    requirements: Vec<Requirement>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let small_plates = Plate::from_weights_map(HashMap::from([(500, 4), (1250, 4), (2500, 4)]), 1);

    let big_plates = Plate::from_weights_map(
        HashMap::from([
            (1250, 8),
            (2500, 12),
            (5000, 2),
            (10000, 2),
            (15000, 2),
            (20000, 2),
        ]),
        2,
    );

    let plates = small_plates
        .into_iter()
        .chain(big_plates)
        .collect::<Vec<_>>();

    let bars = vec![
        Bar::new(2000, 1, BarKind::Dumbbell),
        Bar::new(5000, 2, BarKind::Dumbbell),
        Bar::new(15000, 2, BarKind::Barbell),
    ];

    process_bars(plates, bars, args.requirements)?;

    Ok(())
}

fn process_bars(
    plates: Vec<Plate>,
    bars: Vec<Bar>,
    requirements: Vec<Requirement>,
) -> anyhow::Result<()> {
    let gym = Gym::new(plates, bars);

    match requirements.is_empty() {
        true => {
            let weights = gym.weights();

            println!("Available weights:");
            for (bar, weights) in weights.iter().sorted() {
                println!(
                    "{}: {:?}",
                    bar,
                    weights
                        .iter()
                        .map(|w| *w as f64 / 1000.0)
                        .collect::<Vec<_>>()
                );
            }
        }
        false => {
            let workout = gym.workout(requirements)?;
            for (bar, dumbbells) in workout {
                println!("{bar}");
                for dumbbell in dumbbells {
                    println!("  - {dumbbell}");
                }
            }
        }
    }

    Ok(())
}
