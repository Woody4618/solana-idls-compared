// Debug script to see how Borsh serializes the enum
use counter_program::CounterInstruction;

fn main() {
    let init_instruction = CounterInstruction::InitializeCounter { initial_value: 100 };
    let serialized = borsh::to_vec(&init_instruction).unwrap();

    println!("Serialized InitializeCounter:");
    println!("  Hex: {}", hex::encode(&serialized));
    println!("  Bytes: {:?}", serialized);
    println!("  Length: {}", serialized.len());

    let increment_instruction = CounterInstruction::IncrementCounter;
    let serialized2 = borsh::to_vec(&increment_instruction).unwrap();

    println!("\nSerialized IncrementCounter:");
    println!("  Hex: {}", hex::encode(&serialized2));
    println!("  Bytes: {:?}", serialized2);
    println!("  Length: {}", serialized2.len());

}
