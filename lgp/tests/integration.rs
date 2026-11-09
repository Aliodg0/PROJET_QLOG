use prog64::{TrainingConfig, train};

#[test]
fn integration_training_run_completes() {
    let config = TrainingConfig {
        population_size: 24,
        generations: 3,
        episodes: 2,
        threads: 2,
        early_stop_score: None,
        ..TrainingConfig::default()
    };
    let report = train(config);

    assert!(report.best_score.is_finite());
    assert!(report.best_score >= 0.0);
    assert_eq!(report.generation_best.len(), 3);
}
