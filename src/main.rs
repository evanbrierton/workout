use std::collections::HashMap;

use clap::Parser;
use workout_rs::{
    bar::{Bar, BarType},
    dumbbell::Dumbbell,
    plate::Plate,
};

#[derive(Parser, Debug)]
struct Args {
    weight: Option<u32>,
}

fn main() {
    let args = Args::parse();

    let small_plates = Plate::from_weights_map(
        HashMap::from([(500, 4), (1000, 4), (1250, 4), (2500, 4)]),
        1,
    );

    let big_plates = Plate::from_weights_map(
        HashMap::from([
            (1250, 8),
            (2500, 12),
            (5000, 2),
            (10000, 2),
            (15000, 2),
            (25000, 2),
        ]),
        2,
    );

    let plates = small_plates
        .into_iter()
        .chain(big_plates.into_iter())
        .collect::<HashMap<_, _>>();

    let bars = vec![
        Bar::new(1000, 1, BarType::Dumbbell),
        Bar::new(5000, 2, BarType::Dumbbell),
        Bar::new(15000, 2, BarType::Barbell),
    ];

    let dumbbells = Dumbbell::sort(
        bars.into_iter()
            .map(|bar| Dumbbell::available_from_weight_map(plates.clone(), bar.clone()))
            .flatten()
            .collect::<Vec<_>>(),
    );

    match args.weight {
        Some(w) => {
            let filtered_dumbbells: Vec<Dumbbell> = dumbbells
                .clone()
                .into_iter()
                .filter(|d| d.weight() == w * 1000)
                .collect();

            if filtered_dumbbells.is_empty() {
                println!("No dumbbells found for weight: {}", w);
                return;
            }

            for dumbbell in filtered_dumbbells {
                println!("{}", dumbbell);
            }
        }
        None => {
            for dumbbell in dumbbells {
                println!("{}", dumbbell);
            }
        }
    }
}
