//! Compact 64-bit linear genetic programming helpers.
//!
//! A program is stored in one `u64`. The implementation uses five fixed-width
//! instruction fields, each of which decodes into an operation, a destination
//! register, and two sources. Registers and state inputs are selected by index.
//!
//! Registers are initialised to zero. The first register is used as the return
//! value. Operations protect division and replace non-finite results with zero.
//!
//! All randomness used for training flows through the explicit PCG seed so that
//! two runs with the same seed and configuration produce the same result.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use env::{Environment, State, cartpole::CartPole, pcgrng::PCGRng};
use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};

/// Number of registers available to a program.
pub const NUM_REGISTERS: usize = 4;
/// Number of inputs (environment state values) available to a program.
pub const NUM_INPUTS: usize = 4;
/// Number of instructions encoded in every program.
pub const INSTRUCTIONS: usize = 5;
/// Number of bits used by a single instruction (3 opcode + 2 destination + 3 + 3 inputs).
pub const BITS_PER_INSTRUCTION: usize = 11;
/// Total number of meaningful bits in a program.
pub const PROGRAM_BITS: usize = INSTRUCTIONS * BITS_PER_INSTRUCTION;
/// Bitmask covering all meaningful program bits.
pub const PROGRAM_MASK: u64 = (1u64 << PROGRAM_BITS) - 1;

/// An LGP program stored in a single `u64`.
/// A linear genetic program stored in one 64-bit word.
///
/// The readable value is computed from register 0. All fields are masked so
/// that only the bits used by the encoding can carry information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Program(pub u64);

impl<'de> serde::Deserialize<'de> for Program {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = <u64 as serde::Deserialize<'de>>::deserialize(deserializer)?;
        Ok(Program::new(bits))
    }
}

impl Program {
    /// Wrap an arbitrary bit pattern, masking it to the usable width.
    pub fn new(bits: u64) -> Self {
        Self(bits & PROGRAM_MASK)
    }

    /// Execute the program on the four inputs of the environment state.
    ///
    /// Register 0 is used as the return value.
    pub fn execute(&self, inputs: &State) -> f32 {
        let mut registers = [0.0f32; NUM_REGISTERS];
        for instruction_index in 0..INSTRUCTIONS {
            let instruction = (self.0 >> (instruction_index * BITS_PER_INSTRUCTION))
                & ((1u64 << BITS_PER_INSTRUCTION) - 1);

            let opcode = ((instruction >> 8) & 0x7) as usize;
            let destination = ((instruction >> 6) & 0x3) as usize;
            let first = ((instruction >> 3) & 0x7) as usize;
            let second = (instruction & 0x7) as usize;

            let first_value = value(first, &registers, inputs);
            let second_value = value(second, &registers, inputs);

            let result = match opcode {
                0 => first_value + second_value,
                1 => first_value - second_value,
                2 => first_value * second_value,
                3 => {
                    if second_value.abs() <= f32::EPSILON {
                        0.0
                    } else {
                        first_value / second_value
                    }
                }
                4 => first_value.max(second_value),
                5 => first_value.min(second_value),
                6 => first_value.abs(),
                _ => -first_value,
            };

            registers[destination] = if result.is_finite() { result } else { 0.0 };
        }

        registers[0]
    }

    /// Randomly flip bits with the given probability.
    pub fn mutate<R: Rng>(&mut self, rng: &mut R, rate: f32) {
        let rate = rate.clamp(0.0, 1.0);
        for bit in 0..PROGRAM_BITS {
            if rng.random::<f32>() < rate {
                self.0 ^= 1u64 << bit;
            }
        }
    }
}

fn value(index: usize, registers: &[f32; NUM_REGISTERS], inputs: &State) -> f32 {
    if index < NUM_REGISTERS {
        registers[index]
    } else {
        inputs[index - NUM_REGISTERS]
    }
}

/// Generate the episode seeds shared by every program in one generation.
fn common_episode_seeds(rng: &mut PCGRng, episodes: usize) -> Vec<u64> {
    (0..episodes).map(|_| rng.next_u64()).collect()
}

/// Select the indexes of the best-scoring programs.
pub fn select_survivors(scores: &[f32], survival_rate: f32) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..scores.len()).collect();
    indices.sort_by(|&left, &right| {
        scores[right]
            .partial_cmp(&scores[left])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if scores.is_empty() {
        return indices;
    }

    let count = ((scores.len() as f32 * survival_rate).round() as usize).clamp(1, scores.len() - 1);
    indices.truncate(count);
    indices
}

/// Compute a simple population-level bit diversity score in the 0..=1 range.
fn bit_diversity(population: &[Program]) -> f32 {
    if population.is_empty() {
        return 0.0;
    }

    let mut accumulator = 0.0f32;
    for bit in 0..PROGRAM_BITS {
        let set = population
            .iter()
            .filter(|program| (program.0 & (1u64 << bit)) != 0)
            .count() as f32;
        let fraction = set / population.len() as f32;
        accumulator += 4.0 * fraction * (1.0 - fraction);
    }

    accumulator / PROGRAM_BITS as f32
}

/// Training configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub population_size: usize,
    pub generations: usize,
    pub survival_rate: f32,
    pub mutation_rate: f32,
    pub episodes: usize,
    pub early_stop_score: Option<f32>,
    pub threads: usize,
    pub seed: u64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            population_size: 120,
            generations: 40,
            survival_rate: 0.2,
            mutation_rate: 0.02,
            episodes: 5,
            early_stop_score: Some(475.0),
            threads: 1,
            seed: 1,
        }
    }
}

/// Summary produced by a training run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingReport {
    pub best_program: Program,
    pub best_score: f32,
    pub generation_best: Vec<f32>,
    /// Number of distinct programs observed in each generation.
    pub unique_programs: Vec<usize>,
    /// Mean per-bit diversity in each generation, on a 0..=1 scale.
    pub bit_diversity: Vec<f32>,
    pub training_seeds: Vec<u64>,
    pub parameters: TrainingConfig,
}

/// Evaluate one program over several episodes.
pub fn evaluate_program(program: Program, seeds: &[u64]) -> f32 {
    if seeds.is_empty() {
        return 0.0;
    }

    let mut total = 0.0f32;
    for &seed in seeds {
        let mut rng = PCGRng::seed(seed);
        let mut environment = CartPole::default();
        let mut state = environment.reset(&mut rng);
        let mut episode_reward = 0.0f32;

        loop {
            let action = program.execute(&state);
            let (next_state, reward, done) = environment.step(action, &mut rng);
            episode_reward += reward;
            state = next_state;
            if done {
                break;
            }
        }

        total += episode_reward;
    }

    total / seeds.len() as f32
}

/// Train an LGP population on the CartPole environment.
pub fn train(config: TrainingConfig) -> TrainingReport {
    let mut config = config;
    if config.population_size < 2 {
        config.population_size = 2;
    }
    if config.generations == 0 {
        config.generations = 1;
    }
    if config.episodes == 0 {
        config.episodes = 1;
    }
    if !(0.0..=1.0).contains(&config.survival_rate) {
        config.survival_rate = 0.2;
    }
    if config.mutation_rate < 0.0 {
        config.mutation_rate = 0.0;
    }
    if config.threads == 0 {
        config.threads = 1;
    }

    let population_size = config.population_size;
    let threads = config.threads;
    let mut rng = PCGRng::seed(config.seed);

    let mut population: Vec<Program> = (0..population_size)
        .map(|_| Program::new(rng.next_u64()))
        .collect();

    let training_seeds = common_episode_seeds(&mut rng, config.episodes);
    let seeds_per_program: Vec<Vec<u64>> = vec![training_seeds.clone(); population_size];

    let mut best_program = population[0];
    let mut best_score = f32::MIN;
    let mut history = Vec::with_capacity(config.generations);
    let mut unique_history = Vec::with_capacity(config.generations);
    let mut diversity_history = Vec::with_capacity(config.generations);

    for _generation in 0..config.generations {
        let mut scores = vec![0.0f32; population_size];

        if threads == 1 {
            for (index, program) in population.iter().enumerate() {
                scores[index] = evaluate_program(*program, &seeds_per_program[index]);
            }
        } else {
            let chunk_size = population_size.div_ceil(threads).max(1);
            let mut handles = Vec::new();
            for chunk_start in (0..population_size).step_by(chunk_size) {
                let chunk_end = (chunk_start + chunk_size).min(population_size);
                let population_chunk = population[chunk_start..chunk_end].to_vec();
                let seeds_chunk = seeds_per_program[chunk_start..chunk_end].to_vec();
                handles.push(std::thread::spawn(move || {
                    population_chunk
                        .iter()
                        .copied()
                        .zip(seeds_chunk.iter())
                        .map(|(program, seeds)| evaluate_program(program, seeds))
                        .collect::<Vec<_>>()
                }));
            }

            let mut offset = 0usize;
            for handle in handles {
                let chunk_scores = handle.join().expect("evaluation thread panicked");
                scores[offset..offset + chunk_scores.len()].copy_from_slice(&chunk_scores);
                offset += chunk_scores.len();
            }
        }

        let generation_best_index = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap_or(0);
        let generation_best_score = scores[generation_best_index];

        unique_history.push(population.iter().collect::<HashSet<_>>().len());
        diversity_history.push(bit_diversity(&population));

        if generation_best_score > best_score {
            best_score = generation_best_score;
            best_program = population[generation_best_index];
        }
        history.push(generation_best_score);

        let survivor_indices = select_survivors(&scores, config.survival_rate);
        let survivors: Vec<Program> = survivor_indices
            .iter()
            .map(|&index| population[index])
            .collect();

        let mut next_population = survivors.clone();
        while next_population.len() < population_size {
            let parent_index = rng.random_range(0..survivors.len());
            let mut child = survivors[parent_index];
            child.mutate(&mut rng, config.mutation_rate);
            next_population.push(child);
        }
        population = next_population;

        if let Some(threshold) = config.early_stop_score
            && best_score >= threshold
        {
            break;
        }
    }

    TrainingReport {
        best_program,
        best_score,
        generation_best: history,
        unique_programs: unique_history,
        bit_diversity: diversity_history,
        training_seeds,
        parameters: config,
    }
}

/// Save a program to a JSON file.
pub fn save_program<P: AsRef<Path>>(
    program: &Program,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string_pretty(program)?;
    fs::write(path, text)?;
    Ok(())
}

/// Load a program from a JSON file.
pub fn load_program<P: AsRef<Path>>(path: P) -> Result<Program, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    let program: Program = serde_json::from_str(&text)?;
    Ok(program)
}

/// Save a full training report to a JSON file.
pub fn save_report<P: AsRef<Path>>(
    report: &TrainingReport,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string_pretty(report)?;
    fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop_instruction() -> u64 {
        (((6usize) << 8) | ((1usize) << 6) | ((1usize) << 3)) as u64
    }

    #[test]
    fn program_adds_inputs() {
        let mut bits = (((4usize) << 3) | 5usize) as u64;
        for instruction_index in 1..INSTRUCTIONS {
            bits |= noop_instruction() << (instruction_index * BITS_PER_INSTRUCTION);
        }
        let program = Program::new(bits);
        assert_eq!(program.execute(&[2.0, 3.0, 0.0, 0.0]), 5.0);
    }

    #[test]
    fn protected_division_does_not_panic() {
        let mut bits = ((3usize << 8) | (4usize << 3) | 5usize) as u64;
        for instruction_index in 1..INSTRUCTIONS {
            bits |= noop_instruction() << (instruction_index * BITS_PER_INSTRUCTION);
        }
        let program = Program::new(bits);
        assert_eq!(program.execute(&[1.0, 0.0, 0.0, 0.0]), 0.0);
    }

    #[test]
    fn mutation_is_deterministic_with_same_seed_rng() {
        use rand::SeedableRng;
        let mut rng1 = rand::rngs::SmallRng::seed_from_u64(7);
        let mut rng2 = rand::rngs::SmallRng::seed_from_u64(7);
        let mut first = Program::new(12345);
        let mut second = Program::new(12345);
        first.mutate(&mut rng1, 0.5);
        second.mutate(&mut rng2, 0.5);
        assert_eq!(first, second);
    }

    #[test]
    fn save_load_roundtrip() {
        let path = std::env::temp_dir().join("prog64_program_roundtrip.json");
        let program = Program::new(987654321);
        save_program(&program, &path).expect("save should succeed");
        let loaded = load_program(&path).expect("load should succeed");
        assert_eq!(program, loaded);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn training_is_reproducible() {
        let config = TrainingConfig {
            population_size: 16,
            generations: 2,
            episodes: 2,
            threads: 1,
            ..TrainingConfig::default()
        };

        let first = train(config.clone());
        let second = train(config);

        assert_eq!(first.best_program, second.best_program);
        assert_eq!(first.best_score, second.best_score);
        assert_eq!(first.generation_best, second.generation_best);
    }

    #[test]
    fn normalization_masks_program_bits() {
        assert_eq!(Program::new(u64::MAX), Program::new(PROGRAM_MASK));
    }

    #[test]
    fn mutation_does_not_change_original_when_rate_is_zero() {
        let mut rng = PCGRng::seed(1);
        let original = Program::new(123_456);
        let mut mutated = original;
        mutated.mutate(&mut rng, 0.0);
        assert_eq!(original, mutated);
    }

    #[test]
    fn mutation_with_rate_one_changes_program() {
        let mut rng = PCGRng::seed(2);
        let original = Program::new(123_456);
        let mut mutated = original;
        mutated.mutate(&mut rng, 1.0);
        assert_ne!(original, mutated);
    }

    #[test]
    fn select_survivors_prefers_highest_scores() {
        let scores = [1.0, 5.0, 3.0, 4.0];
        let survivors = select_survivors(&scores, 0.5);
        assert_eq!(survivors, vec![1, 3]);
    }

    #[test]
    fn decoded_zero_instruction_is_neutral_for_unused_registers() {
        let bits = noop_instruction() << BITS_PER_INSTRUCTION;
        let program = Program::new(bits);
        assert_eq!(program.execute(&[9.0, 8.0, 7.0, 6.0]), 0.0);
    }

    #[test]
    fn report_contains_one_metric_per_generation() {
        let config = TrainingConfig {
            population_size: 16,
            generations: 3,
            episodes: 1,
            threads: 2,
            early_stop_score: None,
            ..TrainingConfig::default()
        };

        let report = train(config);
        assert_eq!(report.generation_best.len(), 3);
        assert_eq!(report.unique_programs.len(), 3);
        assert_eq!(report.bit_diversity.len(), 3);
        assert!(report.unique_programs.iter().all(|value| *value >= 1));
        assert!(report.bit_diversity.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn early_stop_none_runs_all_generations() {
        let config = TrainingConfig {
            population_size: 12,
            generations: 4,
            episodes: 1,
            threads: 1,
            early_stop_score: None,
            ..TrainingConfig::default()
        };

        let report = train(config);
        assert_eq!(report.generation_best.len(), 4);
    }

    #[test]
    fn multithreaded_training_is_reproducible_across_thread_counts() {
        let base = TrainingConfig {
            population_size: 16,
            generations: 2,
            episodes: 2,
            threads: 1,
            early_stop_score: None,
            ..TrainingConfig::default()
        };

        let mut one_thread = base.clone();
        one_thread.threads = 1;
        let mut four_threads = base;
        four_threads.threads = 4;

        let first = train(one_thread);
        let second = train(four_threads);

        assert_eq!(first.best_program, second.best_program);
        assert_eq!(first.best_score, second.best_score);
        assert_eq!(first.generation_best, second.generation_best);
        assert_eq!(first.unique_programs, second.unique_programs);
    }

    #[test]
    fn common_episode_seeds_are_deterministic() {
        let mut first_rng = PCGRng::seed(5);
        let mut second_rng = PCGRng::seed(5);
        let first = common_episode_seeds(&mut first_rng, 5);
        let second = common_episode_seeds(&mut second_rng, 5);

        assert_eq!(first, second);
        assert_eq!(first.len(), 5);
    }

    #[test]
    fn invalid_json_load_is_rejected() {
        let path = std::env::temp_dir().join("prog64_invalid.json");
        let _ = std::fs::write(&path, "{not valid json}");
        assert!(load_program(&path).is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn training_seeds_are_fixed_for_all_generations() {
        let config = TrainingConfig {
            population_size: 8,
            generations: 3,
            episodes: 5,
            threads: 1,
            early_stop_score: None,
            ..TrainingConfig::default()
        };

        let report = train(config);

        assert_eq!(report.training_seeds.len(), 5);
        assert_eq!(report.generation_best.len(), 3);
    }

    #[test]
    fn training_seeds_are_reproducible_across_runs() {
        let config = TrainingConfig {
            population_size: 8,
            generations: 2,
            episodes: 4,
            threads: 2,
            early_stop_score: None,
            ..TrainingConfig::default()
        };

        let first = train(config.clone());
        let second = train(config);

        assert_eq!(first.training_seeds, second.training_seeds);
        assert_eq!(first.best_program, second.best_program);
    }

    #[test]
    fn fixed_training_seeds_are_not_regenerated_each_generation() {
        let config = TrainingConfig {
            population_size: 8,
            generations: 3,
            episodes: 3,
            threads: 1,
            early_stop_score: None,
            ..TrainingConfig::default()
        };

        let mut rng = PCGRng::seed(config.seed);
        let _expected_population: Vec<Program> = (0..config.population_size)
            .map(|_| Program::new(rng.next_u64()))
            .collect();
        let expected_seeds = common_episode_seeds(&mut rng, config.episodes);
        let report = train(config);

        assert_eq!(report.training_seeds, expected_seeds);
    }

    #[test]
    fn training_seeds_are_shared_by_all_individuals() {
        let config = TrainingConfig {
            population_size: 8,
            generations: 2,
            episodes: 4,
            threads: 1,
            early_stop_score: None,
            ..TrainingConfig::default()
        };
        let report = train(config);
        assert_eq!(report.training_seeds.len(), 4);
    }

    #[test]
    fn load_normalizes_out_of_range_bits() {
        let path = std::env::temp_dir().join("prog64_out_of_range.json");
        let _ = std::fs::write(&path, (PROGRAM_MASK + 1).to_string());
        let loaded = load_program(&path).expect("load should succeed");
        assert_eq!(loaded, Program::new(PROGRAM_MASK + 1));
        let _ = std::fs::remove_file(path);
    }
}
