use hashbrown::HashMap;
use workout_rs::{
    bar::Bar, bar_kind::BarKind, dumbbell::Dumbbell, gym::Gym, plate::Plate,
    requirement::Requirement,
};

fn main() -> anyhow::Result<()> {
    println!("=== Debugging adjacency and transition costs ===\n");

    // Set up test environment - same as main.rs
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
    let bars = vec![bar];

    // Test adjacency logic first
    println!("1. Testing adjacency logic:");
    test_adjacency(&bar);

    // Test gym ordering multiple times
    println!("\n2. Testing gym ordering consistency (multiple runs):");
    test_gym_consistency(&plates, &bars)?;

    Ok(())
}

fn test_adjacency(bar: &Bar) {
    // Create test dumbbells using plates that match the bar gauge (gauge 2 for barbell)
    let empty = Dumbbell::new(vec![], *bar);
    let single_1_25 = Dumbbell::new(vec![Plate::new(1250, 2)], *bar); // [1.25]
    let single_2_5 = Dumbbell::new(vec![Plate::new(2500, 2)], *bar); // [2.5]
    let single_5 = Dumbbell::new(vec![Plate::new(5000, 2)], *bar); // [5]
    let double_2_5 = Dumbbell::new(vec![Plate::new(2500, 2), Plate::new(2500, 2)], *bar); // [2.5, 2.5]
    let mixed = Dumbbell::new(vec![Plate::new(2500, 2), Plate::new(5000, 2)], *bar); // [2.5, 5]

    println!("  Barbell configurations (using gauge 2 plates):");
    println!("    Empty: {}", empty);
    println!("    Single 1.25kg: {}", single_1_25);
    println!("    Single 2.5kg: {}", single_2_5);
    println!("    Single 5kg: {}", single_5);
    println!("    Double 2.5kg: {}", double_2_5);
    println!("    Mixed (2.5+5): {}", mixed);
    println!();

    // Test adjacencies - these should demonstrate the core logic
    println!("  Adjacency tests:");
    println!(
        "    Empty <-> Single 2.5kg: {}",
        empty.adjacent(&single_2_5)
    );
    println!("    Empty <-> Single 5kg: {}", empty.adjacent(&single_5));
    println!(
        "    Single 2.5kg <-> Double 2.5kg: {}",
        single_2_5.adjacent(&double_2_5)
    );
    println!(
        "    Single 2.5kg <-> Mixed: {}",
        single_2_5.adjacent(&mixed)
    );
    println!("    Single 5kg <-> Mixed: {}", single_5.adjacent(&mixed));
    println!(
        "    Empty <-> Double 2.5kg: {}",
        empty.adjacent(&double_2_5)
    ); // Should be false (2 plate difference)
    println!(
        "    Single 5kg <-> Single 2.5kg: {}",
        single_5.adjacent(&single_2_5)
    ); // Should be false (no overlap)
    println!();
}

fn test_gym_consistency(plates: &HashMap<Plate, usize>, bars: &[Bar]) -> anyhow::Result<()> {
    let gym = Gym::new(plates, bars);

    // First, let's see what weights are actually available for barbell
    println!("  Available weights for barbell bar:");
    if let Some(weights) = gym.weights().get(&bars[0]) {
        let kg_weights: Vec<f64> = weights.iter().map(|w| *w as f64 / 1000.0).collect();
        println!("    {:?}", kg_weights);
    }
    println!();

    // Test case that should demonstrate the inconsistency issue:
    // 25kg -> 20kg barbell transition
    // 25kg can be made with [2.5, 2.5] (15kg bar + 2*2.5kg*2 sides = 15 + 10 = 25kg)
    // 20kg can be made with [2.5] (15kg bar + 1*2.5kg*2 sides = 15 + 5 = 20kg)
    // OR 20kg can be made with [] (just 15kg bar + some plates... let's check what's available)

    let requirements = vec![
        Requirement::new(25000, BarKind::Barbell), // 25kg
        Requirement::new(20000, BarKind::Barbell), // 20kg
    ];

    println!("  Requirements: 25kg -> 20kg (Barbell)");
    println!("  This should show the path inconsistency if it exists");
    println!();

    // Run multiple times to check for consistency
    for run in 1..=5 {
        println!("  Run {}:", run);
        let result = gym.order(&requirements)?;

        if let Some((bar, dumbbells)) = result.iter().next() {
            println!("    Bar: {}", bar);
            for (i, dumbbell) in dumbbells.iter().enumerate() {
                println!("      Step {}: {}", i + 1, dumbbell);
            }
        }
        println!();
    }

    Ok(())
}
