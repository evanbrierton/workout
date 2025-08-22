use hashbrown::HashMap;
use workout_rs::{bar::Bar, bar_kind::BarKind, gym::Gym, plate::Plate, requirement::Requirement};

fn main() -> anyhow::Result<()> {
    println!("=== Debugging DP algorithm state selection ===\n");

    // Set up test environment
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
        .collect::<HashMap<_, _>>();

    let bar = Bar::new(15000, 2, BarKind::Barbell);

    // Test the original main program to see what it chooses
    println!("Testing with original main program:");
    let result = std::process::Command::new("cargo")
        .args(&["run", "--", "25kg", "20kg"])
        .current_dir("/Users/evanbrierton/src/workout-rs")
        .output()
        .expect("Failed to run original program");

    println!("  Original program output:");
    println!("{}", String::from_utf8_lossy(&result.stdout));

    if !result.stderr.is_empty() {
        println!("  Stderr:");
        println!("{}", String::from_utf8_lossy(&result.stderr));
    }

    Ok(())
}
