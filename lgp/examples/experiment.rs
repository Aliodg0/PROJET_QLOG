use env::{Environment, cartpole::CartPole, pcgrng::PCGRng};
use prog64::{TrainingConfig, evaluate_program, save_report, train};
use rand::Rng;
use rand::RngExt;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct IndependentEntry {
    name: String,
    training_seed: u64,
    threads: usize,
    program: u64,
    eval_seed_seed: u64,
    scores: Vec<f32>,
    mean: f32,
    std: f32,
    min: f32,
    max: f32,
}

#[derive(Serialize)]
struct IndependentEvalReport {
    eval_seed_seed: u64,
    eval_seeds: Vec<u64>,
    programs: Vec<IndependentEntry>,
}

fn training_config(seed: u64, threads: usize) -> TrainingConfig {
    TrainingConfig {
        population_size: 100,
        generations: 100,
        survival_rate: 0.2,
        mutation_rate: 0.02,
        episodes: 5,
        early_stop_score: None,
        threads,
        seed,
    }
}

fn summary(name: &str, scores: &[f32]) -> (f32, f32, f32, f32) {
    let n = scores.len() as f32;
    let mean = scores.iter().copied().sum::<f32>() / n;
    let variance = scores
        .iter()
        .copied()
        .map(|score| (score - mean).powi(2))
        .sum::<f32>()
        / (n - 1.0);
    let std = variance.sqrt();
    let min = scores.iter().copied().fold(f32::INFINITY, f32::min);
    let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    println!("{name}: mean={mean:.3} std={std:.3} min={min:.3} max={max:.3}");
    (mean, std, min, max)
}

fn independent_scores(program: prog64::Program, seeds: &[u64]) -> Vec<f32> {
    seeds
        .iter()
        .copied()
        .map(|seed| evaluate_program(program, &[seed]))
        .collect()
}

fn random_policy_scores(seeds: &[u64]) -> Vec<f32> {
    seeds
        .iter()
        .copied()
        .map(|seed| {
            let mut rng = PCGRng::seed(seed);
            let mut environment = CartPole::default();
            let mut _state = environment.reset(&mut rng);
            let mut reward_total = 0.0f32;
            loop {
                let action = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
                let (next, reward, done) = environment.step(action, &mut rng);
                reward_total += reward;
                _state = next;
                if done {
                    break;
                }
            }
            reward_total
        })
        .collect()
}

fn main() {
    fs::create_dir_all("final_reports").ok();

    let configs = [(1_u64, "seed1"), (7_u64, "seed7"), (42_u64, "seed42")];
    let mut reports: Vec<(u64, usize, String, prog64::TrainingReport)> = Vec::new();
    for (seed, label) in configs {
        let config = training_config(seed, 1);
        let report = train(config);
        let file_label = label.to_string();
        let path = format!("final_pop100_{}.json", file_label);
        save_report(&report, &path).expect("save training report");
        println!(
            "{file_label}: best_score={} bits={}",
            report.best_score, report.best_program.0
        );
        reports.push((seed, 1_usize, file_label, report));
    }

    let repeat_config = training_config(1, 1);
    let repeat_report = train(repeat_config);
    save_report(&repeat_report, "final_pop100_seed1_repeat.json").expect("save repeat");
    reports.push((1, 1, "seed1_repeat".to_string(), repeat_report));

    let four_thread_config = training_config(1, 4);
    let four_thread_report = train(four_thread_config);
    save_report(&four_thread_report, "final_pop100_seed1_threads4.json").expect("save 4 threads");
    reports.push((1, 4, "seed1_threads4".to_string(), four_thread_report));

    let eval_seed_seed = 10_000_u64;
    let mut eval_rng = PCGRng::seed(eval_seed_seed);
    let eval_seeds: Vec<u64> = (0..100).map(|_| eval_rng.next_u64()).collect();
    fs::write(
        "final_independent_eval_seeds.json",
        serde_json::to_string_pretty(&eval_seeds).expect("serialize eval seeds"),
    )
    .expect("save eval seeds");

    let mut entries = Vec::new();
    for (seed, threads, label, report) in reports {
        let scores = independent_scores(report.best_program, &eval_seeds);
        let (mean, std, min, max) = summary(&label, &scores);
        entries.push(IndependentEntry {
            name: label.clone(),
            training_seed: seed,
            threads,
            program: report.best_program.0,
            eval_seed_seed,
            scores,
            mean,
            std,
            min,
            max,
        });
    }

    let random_scores = random_policy_scores(&eval_seeds);
    let (mean, std, min, max) = summary("random", &random_scores);
    entries.push(IndependentEntry {
        name: "random_policy".to_string(),
        training_seed: 0,
        threads: 1,
        program: 0,
        eval_seed_seed,
        scores: random_scores,
        mean,
        std,
        min,
        max,
    });

    let independent = IndependentEvalReport {
        eval_seed_seed,
        eval_seeds,
        programs: entries,
    };
    fs::write(
        "final_independent_eval_100.json",
        serde_json::to_string_pretty(&independent).expect("serialize independent report"),
    )
    .expect("save independent report");

    println!("done");
}
