//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long)]
    vk: String,

    #[clap(long)]
    proof: String,

    #[clap(long)]
    inputs: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    // Parse the command line arguments.
    let args = Args::parse();
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }
    // Setup the prover client.
    let client = ProverClient::new();
    // Load json files
    let vk_json = std::fs::read_to_string(args.vk).unwrap();
    let proof_json = std::fs::read_to_string(args.proof).unwrap();
    let inputs_json = std::fs::read_to_string(args.inputs).unwrap();
    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&vk_json);
    stdin.write(&proof_json);
    stdin.write(&inputs_json);
    // Execute the program or generate a proof.
    if args.execute {
        // Execute the program
        let (mut output, report) = client.execute(ELF, stdin).run().unwrap();
        println!("Program executed successfully.");
        // Read the output.
        let is_valid = output.read::<bool>();
        println!("is valid? {}", is_valid);
        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(ELF);
        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");
        println!("Successfully generated proof!");
        // Save the proof to a file.
        proof.save("proof.bin").expect("failed to save proof");
        println!("Proof saved to proof.bin");
        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
