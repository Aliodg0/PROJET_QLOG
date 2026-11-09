# RÉSUMÉ EXÉCUTIF — PROJET PROG64

`prog64` est un projet Rust qui fait évoluer des programmes génétiques linéaires de 55 bits (dans un `u64`) sur CartPole. Pipeline complet: encodage, évolution, sélection, mutation, multi-threads, sérialisation, tests de reproductibilité, évaluation indépendante.

Résultats confirme : sur 100 graines indépendantes, seed 7 obtient 500/500 avec moyenne 500.000, std 0.000. Seeds 1 et 42 obtiennent moyenne 479.240 (88/100 réussies, min 241). Politique aléatoire: 22.510. Validation technique OK : format, clippy sans erreur, tests workspace vert.

Forces : reproductibilité, iso-thread, graines communes et une évaluation indépendante. Limites : sensibilité à la seed, budget épisodes de entraînement, représentation courte, absence de croisement. Le présent rapport est uniquement Markdown car aucun convertisseur PDF n'est disponible.