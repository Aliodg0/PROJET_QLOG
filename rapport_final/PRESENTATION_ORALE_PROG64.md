# Présentation orale — prog64

## Plan détaillé — 10 à 15 minutes

### Partie 1 — Introduction (1 minute)

Afficher : titre du projet, l'environnement CartPole, un tableau simple.

Dire :
"prog64 est un projet Rust qui fait évoluer une représentation compacte sur 64 bits dans l'environnement CartPole. Mon objectif principal est d'obtenir et de vérifier une stratégie de contrôle."

Notions à retenir :
- CartPole: chariot et pendule.
- objectif: éviter l'échec le plus longtemps possible.
- maximum: 500.

Transition :
"Avant d'expliquer comment le programme agit, je présente l'encodage."

### Partie 2 — Encodage sur 64 bits (2 minutes)

Afficher : schéma binaire du bloc d'instruction.

Dire :
"Le projet utilise un `u64`, avec cinq instructions de 11 bits, soit 55 bits significatifs. Une instruction se décompose en opcode, destination, source et source. Les registres 0 à 3 sont des mémoires internes; les sources 4 à 7 correspondent aux entrées CartPole."

Notions :
- opcode 3 bits;
- destination 2 bits;
- deux sources de 3 bits;
- mask `PROGRAM_MASK`.

Transition :
"Avec ce programme, l'ordinateur effectue des passes linéaires."

### Partie 3 — Algorithme et graines (3 minutes)

Afficher : schéma du cycle de l'algorithme.

Dire :
"L'algorithme initialise la population, évalue chaque programme, sélectionne les survivants, clone, mute et recommence sur 100 générations. Les graines d'épisode sont communes: toutes les solutions reçoivent les mêmes conditions. Cela donne une comparaison juste et une reproductibilité."

Notions :
- sélection par score;
- pas de croisement;
- mutation;
- graines.

Transition :
"Je passe maintenant au problème physique."

### Partie 4 — CartPole (2 minutes)

Afficher : schéma de CartPole et limites.

Dire :
"CartPole possède quatre variables: position, vitesse, angle et vitesse angulaire. Les actions possibles sont gauche ou droite. Le programme pousse avec une force de 10. Chaque pas donne 1 si la situation reste valide. Un épisode se termine à 500 pas."

Notions :
- seuils 2.4 et 12 degrés;
- pas de 0.02;
- terminal stable.

Transition :
"Le projet n'est pas seulement une simulation; il inclut des contrôles de qualité."

### Partie 5 — Fiabilité et tests (2 minutes)

Afficher : liste des commandes et des tests.

Dire :
"La reproduction est fondée sur un PCG déterministe. Les graines communes et l'état terminal sont testés. Mono-thread et multithread donnent le même programme et les mêmes résultats. Le projet inclut vingt tests unitaires et un test d'intégration."

Notions :
- reproducibilité;
- terminal state;
- thread safety logique.

Transition :
"Enfin, je présente les résultats."

### Partie 6 — Résultats expérimentaux (3 minutes)

Afficher : table des moyennes indépendantes.

Dire :
"Sur 100 graines jamais utilisées à l'entraînement, seed 7 obtient 500 sur 100 épisodes, moyenne 500 et écart-type 0. Les seeds 1 et 42 obtiennent 479.240 avec 88 réussites complètes et un minimum de 241. La politique aléatoire obtient 22.510. Cela montre un gain net, sans démontrer une généralisation universelle."

Notions :
- 500 maximum;
- 88/100;
- baseline aléatoire;
- limites.

Transition :
"Pour conclure."

### Partie 7 — Conclusion (1 à 2 minutes)

Afficher : limites et perspectives.

Dire :
"Le projet présente un pipeline complet et vérifiable, avec une preuve expérimentale solide et des limitations franches. Les extensions possibles concernent la diversité comportementale et l'exploration d'autres politiques."

Notions :
- pipeline complet;
- réussite expérimentale de seed 7;
- limites honnêtes.

Fin :
"Merci pour votre attention. Je peux maintenant répondre aux questions."