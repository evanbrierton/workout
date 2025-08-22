use hashbrown::HashMap;
use workout_rs::{bar::Bar, bar_kind::BarKind, gym::Gym, plate::Plate, requirement::Requirement};

fn main() -> anyhow::Result<()> {
    println!("=== Debugging the specific 25kg->20kg inconsistency ===\n");

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
    let gym = Gym::new(&plates, &[bar]);

    // Test the specific issue: there are two ways to make 25kg
    // Path 1: [] -> [5.0] -> [] -> [2.5] (3 steps)
    // Path 2: [] -> [2.5, 2.5] -> [2.5] (2 steps)
    // Path 2 should always win, but we're seeing inconsistency

    println!("Testing 25kg -> 20kg path finding:");
    println!("- 25kg can be made with [5.0] OR [2.5, 2.5]");
    println!("- 20kg can be made with [2.5]");
    println!("- Path [2.5, 2.5] -> [2.5] = 1 step");
    println!("- Path [5.0] -> [] -> [2.5] = 2 steps");
    println!("- The 1-step path should ALWAYS win\n");

    // Run the problematic test multiple times
    let requirements = vec![
        Requirement::new(25000, BarKind::Barbell), // 25kg
        Requirement::new(20000, BarKind::Barbell), // 20kg
    ];

    let mut results = Vec::new();
    for run in 1..=10 {
        let result = gym.order(&requirements)?;
        if let Some((_, dumbbells)) = result.iter().next() {
            let path: Vec<String> = dumbbells.iter().map(|d| format!("{}", d)).collect();
            results.push(path.clone());

            // Show which 25kg configuration was chosen
            if dumbbells.len() >= 2 {
                let kg_25_config = &dumbbells[1]; // Second step should be 25kg
                println!("  Run {}: 25kg made with {}", run, kg_25_config);
            }
        }
    }

    // Check if all results are the same
    let first_result = &results[0];
    let all_same = results.iter().all(|r| r == first_result);

    println!("\nConsistency check:");
    if all_same {
        println!("  ✓ All runs produced the same result");
        println!("  Path: {}", first_result.join(" -> "));
    } else {
        println!("  ✗ INCONSISTENCY DETECTED!");
        for (i, result) in results.iter().enumerate() {
            println!("    Run {}: {}", i + 1, result.join(" -> "));
        }
    }

    Ok(())
}
