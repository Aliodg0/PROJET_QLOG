# Questions du professeur — prog64

## 1. Compréhension générale

1. Quel est l'objectif du projet ?
Réponse : apprendre une politique de contrôle pour CartPole avec une représentation volontairement compacte de 55 bits. Le travail met l'accent sur le pipeline expérimental complet.

2. Pourquoi utiliser un algorithme génétique ?
Réponse : parce qu'il maintient une population de candidats et améliore leurs résultats par sélection et mutation, sans exiger un modèle analytique.

3. Qu'est-ce qui constitue le score d'un individu ?
Réponse : la moyenne du score cumulé sur plusieurs épisodes CartPole. Le score d'un épisode est la somme des récompenses de 1.

4. Que démontre réellement le projet ?
Réponse : la capacité à trouver et tester un programme de contrôle reproductible avec validation mono-thread/multithread et comparaison au hasard.

5. Le projet fournit-il une intelligence artificielle générale ?
Réponse : non. Il se limite à une tâche précise et à une représentation compacte.

## 2. Rust et implementation

6. Pourquoi Rust est-il pertinent ici ?
Réponse : Rust impose une gestion sûre des types et des décalages, ce qui est utile pour un encodage sur 64 bits.

7. Quelle différence y a-t-il entre 64 bits et 55 bits ?
Réponse : 64 bits désignent le conteneur; 55 bits désignent la partie sémantiquement significative. Les neuf bits de plus haut poids sont masqués.

8. Quelles sont les composantes d'une instruction ?
Réponse : 3 bits opcode, 2 bits destination, 3 bits source, 3 bits source.

9. Quel est le rôle des registres ?
Réponse : quatre registres sont remis à zéro; les sources 0 à 3 désignent les registres, 4 à 7 désignent les entrées.

10. Les opérations arithmétiques sont-elles protégées ?
Réponse : la division protège un dénominateur proche de zéro en retournant 0; les résultats non finis sont aussi remplacés par 0.

## 3. Programmation génétique
11. Le code utilise-t-il le croisement ?
Réponse : non. Seuls la sélection et la mutation sont utilisés.

12. Que fait `select_survivors` ?
Réponse : il trie les scores de manière décroissante et garde le nombre sélectionné, borné entre 1 et population_size - 1.

13. Pourquoi un tri décroissant ?
Réponse : dans CartPole, plus de reward est meilleur; nous gardons donc les plus performants.

14. Comment sont créés les enfants ?
Réponse : par clonage d'un survivant choisi au hasard, puis mutation avec probabilité 0.02.

15. Que signifie l'élitisme ?
Réponse : les survivants exacts sont copiés au début de la population suivante, tandis que les enfants mutés complètent.

16. Que fait la mutation ?
Réponse : chaque bit parmi 55 peut être inversé avec probabilité `rate`, qui est bornée dans [0, 1].

17. Que signifie un taux 0.02 ?
Réponse : chaque bit a 2 % de chance indépendante d'être inversé à chaque mutation.

18. Peut-on perdre le meilleur programme ?
Réponse : les survivants sont copiés, mais la sélection ne force pas un seul programme à survivre jusqu'à la fin si la population diversifie.

## 4. CartPole et physique

19. Comment CartPole est-il simulé ?
Réponse : par les équations présentes dans `cartpole.rs`, avec gravité 9.8, masses, demi-longueur 0.5, pas 0.02 et force 10.

20. Quand un épisode se termine-t-il ?
Réponse : si le chariot sort, si le pendule dépasse l'angle, ou après 500 pas.

21. Qu'est-ce que la récompense d'un épisode ?
Réponse : elle vaut 1 par pas réussi. L'arrêt terminal ne donne pas de récompense supplémentaire.

22. Quelle est la différence entre état terminal et reset ?
Réponse : le terminal state est stable si le pas continue, tandis que `reset` recommence un nouvel épisode.

23. Comment l'état devient-il une action ?
Réponse : le programme calcule `execute(state)`, puis l'action est vers la droite si le résultat est strictement positif, sinon vers la gauche.

## 5. Reproductibilité et statistiques
24. Pourquoi les graines communes sont-elles importantes ?
Réponse : elles garantissent que tous les individus sont évalués sur la même distribution, rendant les comparaisons plus justes.

25. Comment la graine principale détermine-t-elle la population ?
Réponse : la population initiale et les cinq seeds épisodiques sont tirées du même PCG initialisé avec la seed principale.

26. Les exécutions mono-thread et multithread sont-elles réellement équivalentes ?
Réponse : oui. Même programme final, score, métriques, graines et scores indépendants. Seul le nombre de threads change.

27. Comment Seed 7 atteint-il 500 sur 100 ?
Réponse : sa stratégie réussit sur toutes les conditions initiales testées, ce qui donne une moyenne de 500 et un écart-type nul.

28. Que signifie une moyenne de 479.240 ?
Réponse : le contrôle est globalement excellent, avec 88 réussites complètes sur 100 et un minimum de 241.

29. Pourquoi une politique aléatoire est-elle utile comme baseline ?
Réponse : elle objectifie le gain de l'évolution : 22,510 contre 479,240 ou 500.

30. Quelles limites scientifiques faut-il signaler ?
Réponse : sensibilité à la seed, 5 épisodes d'entraînement, 55 bits, 100 graines de test, absence de croisement et absence de diversité comportementale explicite.

## 6. Questions difficiles et pièges à éviter

- Ne pas dire que l'agent est universel. Les résultats valident une distribution, pas tous les états physiques.
- Ne pas présenter la diversité bit comme une diversité comportementale.
- Ne pas affirmer l'existence d'un croisement : le code n'en contient pas.
- Ne pas dire que les résultats sont surprenants au point de remplacer une preuve de validité.
- Ne pas prétendre avoir exécuté des tests sans preuve. La preuve disponible est `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, réussis.

Ce document contient 30 questions avec réponses orales de 30 à 60 secondes.