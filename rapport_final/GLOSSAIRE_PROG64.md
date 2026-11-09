# Glossaire pédagogique — prog64

## Algorithme
- Définition simple : suite ordonnée d'étapes.
- Définition technique : procédure organisée pour résoudre un problème.
- Exemple dans prog64 : `train()` initialise, évalue, sélectionne et mute la population.

## Intelligence artificielle
- Définition simple : programme qui prend une action utile.
- Définition technique : automatisme fondé sur de l'information.
- Exemple : un `Program` choisit gauche ou droite dans CartPole.

## Algorithme génétique
- Définition simple : faire évoluer des candidats.
- Définition technique : optimisation populationnelle fondée sur sélection et variation.
- Exemple : les survivants de CartPole engendrent la génération suivante.

## Programmation génétique
- Définition simple : chercher un code.
- Définition technique : les individus sont des programmes exécutables.
- Exemple : chaque individu est un `u64`.

## Programmation génétique linéaire (LGP)
- Définition simple : un programme composé d'instructions linéaires.
- Définition technique : chaque instruction lit des registres et des entrées, puis écrit un registre.
- Exemple : cinq instructions de 11 bits.

## Population
- Définition simple : groupe de candidats.
- Définition technique : `Vec<Program>` évaluée à chaque génération.
- Exemple : 100 individus par run.

## Individu
- Définition simple : un candidat isolé.
- Définition technique : une instance de programme.
- Exemple : `Program::new(bits)`.

## Chromosome
- Définition simple : support de l'information.
- Définition technique : mot de bits du programme.
- Exemple : `u64`.

## Gène
- Définition simple : petite unité changeable.
- Définition technique : bit ou champ de l'encodage.
- Exemple : un flip de bit dans `mutate`.

## Instruction
- Définition simple : un calcul élémentaire.
- Définition technique : champ de 11 bits avec opération, destination et sources.
- Exemple : opcode 0 additionne deux valeurs.

## Génération
- Définition simple : un cycle complet.
- Définition technique : une itération de l'algorithme.
- Exemple : 100 générations.

## Sélection
- Définition simple : garder les meilleurs.
- Définition technique : tri décroissant des scores.
- Exemple : `select_survivors`.

## Mutation
- Définition simple : modification petite et aléatoire.
- Définition technique : flip de bits avec probabilité donnée.
- Exemple : mutation_rate 0.02.

## Élitisme
- Définition simple : conserver directement les excellents.
- Définition technique : copie des survivants en tête de la population suivante.
- Exemple : `next_population = survivors.clone`.

## Fonction de récompense
- Définition simple : premier feedback positif à chaque pas.
- Définition technique : score renvoyé par CartPole.
- Exemple : 1.0 par étape réussie.

## Fitness
- Définition simple : qualité d'un individu.
- Définition technique : moyenne du score cumulé sur plusieurs épisodes.
- Exemple : `evaluate_program`.

## Épisode
- Définition simple : une tentative complète.
- Définition technique : du `reset` jusqu'à la terminaison.
- Exemple : maximum 500 pas.

## Environnement
- Définition simple : monde de test.
- Définition technique : état et transitions.
- Exemple : CartPole.

## Graine aléatoire
- Définition simple : numéro déterminant l'aléatoire.
- Définition technique : seed du générateur PCG.
- Exemple : 1, 7, 42.

## Reproductibilité
- Définition simple : même expérience, même résultat.
- Définition technique : déterminisme garanti par les graines et le code.
- Exemple : répétition de seed 1.

## Diversité génétique
- Définition simple : variété représentationnelle.
- Définition technique : répartition des bits dans une population.
- Exemple : 0.0886 pour seed 7.

## Encodage binaire
- Définition simple : stocker l'information en bits.
- Définition technique : masques, décalages et champs.
- Exemple : `PROGRAM_MASK`.

## u64
- Définition simple : entier positif représenté sur 64 bits.
- Définition technique : support du chromosome.
- Exemple : `Program(pub u64)`.

## Évaluation indépendante
- Définition simple : tester sur des données nouvelles.
- Définition technique : utilisation de 100 graines non vues pendant l'entraînement.
- Exemple : `final_independent_eval_100.json`.

## Écart-type
- Définition simple : dispersion.
- Définition technique : sqrt(variance n-1).
- Exemple : 59.274 pour seed 1.

## Moyenne
- Définition simple : score moyen.
- Définition technique : somme divisée par le nombre d'observations.
- Exemple : 479.240.

## Score maximal
- Définition simple : réussite complète d'un épisode.
- Définition technique : 500 pas valides.
- Exemple : seed 7 indépendant.