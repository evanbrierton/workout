use hashbrown::HashMap;
use workout_rs::{
    bar::Bar, bar_kind::BarKind, dumbbell::Dumbbell, gym::Gym, plate::Plate,
    requirement::Requirement,
};

fn main() -> anyhow::Result<()> {
    println!("=== Examining states and testing optimal selection ===\n");

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

    // Use the same setup as main.rs
    let bars = vec![
        Bar::new(2000, 1, BarKind::Dumbbell),
        Bar::new(5000, 2, BarKind::Dumbbell),
        Bar::new(15000, 2, BarKind::Barbell),
    ];

    // Filter for barbell bars like main.rs does
    let barbell_bars: Vec<_> = bars
        .into_iter()
        .filter(|bar| *bar.kind() == BarKind::Barbell)
        .collect();

    let gym = Gym::new(&plates, &barbell_bars);

    // Let's examine specific weight configurations manually
    println!("Manual examination of 25kg configurations:");

    // Create the two possible 25kg configurations using the barbell bar
    let bar = barbell_bars[0];
    let config_double_2_5 = Dumbbell::new(vec![Plate::new(2500, 2), Plate::new(2500, 2)], bar);
    let config_single_5 = Dumbbell::new(vec![Plate::new(5000, 2)], bar);
    let config_20kg = Dumbbell::new(vec![Plate::new(2500, 2)], bar);
    let empty_config = Dumbbell::new(vec![], bar);

    println!("  Configuration 1 (double 2.5): {}", config_double_2_5);
    println!("  Configuration 2 (single 5.0): {}", config_single_5);
    println!("  Target 20kg: {}", config_20kg);
    println!("  Empty: {}", empty_config);
    println!();

    // Test adjacencies to confirm transition costs
    println!("Adjacency tests:");
    println!(
        "  [2.5, 2.5] -> [2.5]: {}",
        config_double_2_5.adjacent(&config_20kg)
    );
    println!(
        "  [5.0] -> [2.5]: {}",
        config_single_5.adjacent(&config_20kg)
    );
    println!("  [5.0] -> []: {}", config_single_5.adjacent(&empty_config));
    println!("  [] -> [2.5]: {}", empty_config.adjacent(&config_20kg));
    println!();

    println!("Expected path analysis:");
    println!("  Path 1: [2.5, 2.5] -> [2.5] = 1 transition (direct)");
    println!("  Path 2: [5.0] -> [] -> [2.5] = 2 transitions");
    println!("  Path 1 should always be chosen as optimal!");
    println!();

    // Test to make sure we're getting the right configuration
    println!("Testing actual gym ordering:");
    let requirements = vec![
        Requirement::new(25000, BarKind::Barbell), // 25kg
        Requirement::new(20000, BarKind::Barbell), // 20kg
    ];

    let result = gym.order(&requirements)?;
    if let Some((_, dumbbells)) = result.iter().next() {
        println!("  Result path:");
        for (i, dumbbell) in dumbbells.iter().enumerate() {
            println!("    Step {}: {}", i + 1, dumbbell);
        }

        // Check if we used the optimal path
        if dumbbells.len() >= 2 {
            let kg_25_config = &dumbbells[1];
            if kg_25_config.plates().len() == 2 {
                println!("  ✓ Used optimal [2.5, 2.5] configuration for 25kg");
            } else if kg_25_config.plates().len() == 1 {
                println!("  ⚠ Used [5.0] configuration - this might indicate suboptimal selection");
            }
        }
    }

    // Test with a case where plate complexity should only be a tiebreaker
    println!("");
    println!("{}", "=".repeat(60));
    println!("Testing tie-breaking scenario:");

    // Create a scenario where multiple configurations have the same transition cost
    // Let's try 30kg -> 27.5kg (if both exist)
    let requirements_tie = vec![
        Requirement::new(30000, BarKind::Barbell), // 30kg
        Requirement::new(27500, BarKind::Barbell), // 27.5kg
    ];

    match gym.order(&requirements_tie) {
        Ok(result) => {
            if let Some((_, dumbbells)) = result.iter().next() {
                println!("  30kg -> 27.5kg path:");
                for (i, dumbbell) in dumbbells.iter().enumerate() {
                    println!("    Step {}: {}", i + 1, dumbbell);
                }
            }
        }
        Err(e) => {
            println!("  Could not find 30kg -> 27.5kg path: {}", e);
        }
    }

    Ok(())
}
