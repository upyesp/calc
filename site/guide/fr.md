# Guide de l'utilisateur de epher

Bienvenue ! epher est une calculatrice programmable et scriptable. Vous pouvez
l'utiliser pour un calcul rapide, ou construire vos propres fonctions et
petits programmes — et tout est disponible en six langues.

Ce guide s'adresse aux débutants complets. Il commence par le calcul le plus
simple possible et monte jusqu'à toute la puissance du langage. Chaque
exemple montre ce que vous tapez et ce que epher répond.

Il y a quatre façons d'utiliser epher — choisissez celle qui vous convient :

| Version | Ce que c'est | Quand la choisir |
|---|---|---|
| **Application web** (PWA) | Tourne dans votre navigateur, installable, fonctionne hors ligne | Pour démarrer au plus vite ; sans installation |
| **Application de bureau** | Un programme classique avec sa propre fenêtre | Pour une application classique |
| **Ligne de commande** (CLI) | Commandes texte dans un terminal ; aussi une session interactive | Vous vivez dans un terminal et aimez les scripts |
| **Interface de terminal** (TUI) | Un programme plein écran dans le terminal | Pour une appli terminal avec graphiques et historique |

Les quatre versions comprennent exactement le même langage. Apprenez-le une
fois, utilisez-le partout.

## 1. Le langage de epher

Ce chapitre enseigne le langage commun à toutes les versions de epher. Dans
l'application web ou de bureau, tapez une expression et appuyez sur
**Entrée** (ou cliquez sur le bouton **=**). Dans la CLI, tapez-la après
l'invite `epher>`. Dans la TUI, tapez et appuyez sur **Entrée**. Dans la CLI
vous pouvez aussi écrire `epher "expression"` pour évaluer directement une
expression.

### 1.1 Votre premier calcul

Tapez ceci :

```text
2 + 3 * 4
```

epher répond :

```text
14
```

La multiplication se fait avant l'addition, exactement comme en
mathématiques. Cette règle s'appelle la *précédence des opérateurs*.

### 1.2 Ordre des opérations

L'ordre complet de précédence, du plus fort au plus faible :

1. `!` factorielle
2. `^` puissance
3. `*` et `/` multiplication et division
4. `+` et `-` addition et soustraction

Utilisez des parenthèses pour changer l'ordre :

```text
(2 + 3) * 4
```

```text
20
```

L'opérateur `^` calcule les puissances et fonctionne de droite à gauche :

```text
2 ^ 10
```

```text
1024
```

```text
2 ^ 3 ^ 2
```

```text
512
```

(`2 ^ 3 ^ 2` signifie `2 ^ (3 ^ 2)`, c'est-à-dire `2 ^ 9` = 512.)

Les puissances peuvent être fractionnaires — `2 ^ 0.5` est la racine carrée
de 2 :

```text
2 ^ 0.5
```

```text
1.4142135623730951
```

La soustraction et la division fonctionnent de gauche à droite :

```text
10 - 3 - 2
```

```text
5
```

### 1.3 Les nombres spéciaux pi, e, tau et phi

Les constantes célèbres sont intégrées :

```text
pi
```

```text
3.141592653589793
```

```text
2 * pi
```

```text
6.283185307179586
```

```text
e
```

```text
2.718281828459045
```

Deux autres : `tau` est un tour complet (2 pi) et `phi` est le nombre d'or :

```text
tau
```

```text
6.283185307179586
```

```text
phi
```

```text
1.618033988749895
```

### 1.4 Comparer et logique

Vous pouvez comparer des nombres. Le résultat est `true` (vrai) ou `false`
(faux) :

| Comparaison | Signification |
|---|---|
| `a > b` | a est plus grand que b |
| `a < b` | a est plus petit que b |
| `a >= b` | a est plus grand ou égal à b |
| `a <= b` | a est plus petit ou égal à b |
| `a == b` | a est égal à b (notez le double `=`) |
| `a != b` | a n'est pas égal à b |

```text
3 > 2
```

```text
true
```

```text
1 != 2
```

```text
true
```

Combinez les comparaisons avec `and`, `or` et `not` :

```text
3 > 2 and 2 < 3
```

```text
true
```

```text
not 3 > 2
```

```text
false
```

### 1.5 Variables

Donnez un nom à une valeur avec un seul `=` :

```text
x = 5
```

```text
5
```

epher vous répète la valeur. Désormais, `x` peut être utilisé partout :

```text
x ^ 2
```

```text
25
```

Vous pouvez changer une variable quand vous voulez — elle garde sa valeur
jusqu'à ce que vous la changiez :

```text
x = x + 1
```

```text
6
```

> Les noms peuvent contenir des lettres et des tirets bas, comme `radius` ou
> `my_total`. Ils ne peuvent pas contenir d'espaces ni commencer par un
> chiffre.

### 1.6 Les décisions avec if

`if` choisit entre deux valeurs :

```text
if 3 > 2 then 10 else 20
```

```text
10
```

La forme est toujours `if condition then valeur_si_vrai
else valeur_si_faux`. La partie `else` est obligatoire.

Un exemple plus utile avec une variable :

```text
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> epher n'a pas de valeurs texte — les deux branches d'un `if` doivent être
> des nombres (ou des résultats de comparaisons).

### 1.7 Les boucles avec while

`while` répète une instruction tant qu'une condition est vraie :

```text
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Lisez ce script ainsi : *commence x à 0 ; tant que x est inférieur à 5,
ajoute 1 à x ; puis affiche x.* Le résultat est 5 parce que la boucle s'est
exécutée cinq fois.

> **Filet de sécurité :** epher arrête toute boucle après 100 000 étapes et
> affiche `error: step limit exceeded`. Cela vous protège des boucles qui ne
> se termineraient jamais. Si vous le voyez, votre condition ne devenait
> probablement jamais fausse.

### 1.8 Vos propres fonctions avec def

Une fonction est un calcul avec un nom et des paramètres :

```text
def f(x) = x ^ 2
```

Puis utilisez-la :

```text
f(7)
```

```text
49
```

Les fonctions peuvent avoir plusieurs paramètres :

```text
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

Vous pouvez aussi définir une fonction sans paramètre :

```text
def answer() = 42
answer()
```

```text
42
```

### 1.9 La récursivité : une fonction qui s'appelle elle-même

L'exemple le plus célèbre — les nombres de Fibonacci :

```text
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```text
fib(10)
```

```text
55
```

`fib(10)` est le dixième nombre de Fibonacci. La fonction s'appelle
elle-même avec des arguments plus petits jusqu'à atteindre `n <= 1`. Cela
fonctionne parce que la forme `if ... then ... else ...` ne calcule que la
branche dont elle a besoin.

> Le corps d'une fonction est une seule expression — une ligne. Combinez
> plutôt plusieurs calculs avec `;` dans un script (section suivante).

### 1.10 Les scripts : plusieurs instructions à la fois

Un *script* est plusieurs instructions reliées par `;`, exécutées l'une
après l'autre :

```text
x = 10; y = x + 5; x + y
```

```text
25
```

Les scripts sont la façon de construire de petits programmes : préparez des
variables, faites des boucles, et affichez un résultat final.

### 1.11 Résultats exacts : frac, dec et big

Normalement epher calcule avec des nombres décimaux comme une calculatrice de
poche. Certains nombres sont plus beaux exacts.

**frac(n, d)** crée une fraction exacte :

```text
1 / 3
```

```text
0.3333333333333333
```

```text
frac(1, 3)
```

```text
1/3
```

Les fractions restent exactes à travers les calculs :

```text
frac(1, 3) * 3
```

```text
1
```

**dec(x)** crée un nombre décimal exact. Comparez ces deux-là :

```text
0.1 + 0.2
```

```text
0.30000000000000004
```

```text
dec(0.1) + dec(0.2)
```

```text
0.3
```

Le premier résultat est la petite erreur d'arrondi que tout ordinateur fait
avec les nombres décimaux. `dec()` l'élimine.

**big(x)** crée un nombre entier exact, pour les valeurs trop grandes pour
une calculatrice de poche :

```text
big(10 ^ 20)
```

```text
100000000000000000000
```

### 1.12 Fonctions intégrées

epher possède les fonctions d'une calculatrice scientifique, regroupées par
famille.

La trigonométrie travaille en radians — utilisez `deg` et `rad` pour
convertir :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | fonctions trigonométriques | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | trigonométrie inverse | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | angle du point (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | radians → degrés | `deg(pi)` | `180` |
| `rad(x)` | degrés → radians | `rad(180)` | `3.141592653589793` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | fonctions hyperboliques | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | hyperboliques inverses | `acosh(1)` | `0` |

Puissances, racines et logarithmes (sur une calculatrice `log` est en
base 10) :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `sqrt(x)` | racine carrée | `sqrt(16)` | `4` |
| `cbrt(x)` | racine cubique | `cbrt(-27)` | `-3` |
| `root(n, x)` | racine n-ième | `root(3, 8)` | `2` |
| `exp(x)` | e puissance x | `exp(1)` | `2.718281828459045` |
| `ln(x)` | logarithme népérien | `ln(e)` | `1` |
| `log(x)` | logarithme base 10 | `log(100)` | `2` |
| `log2(x)` | logarithme base 2 | `log2(8)` | `3` |
| `logb(b, x)` | logarithme en base b | `logb(2, 8)` | `3` |
| `hypot(a, b)` | hypoténuse | `hypot(3, 4)` | `5` |
| `5!` (aussi `fact(n)`) | factorielle | `5!` | `120` |

Arrondis, signes et nombres entiers :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `abs(x)` | valeur absolue | `abs(-3)` | `3` |
| `floor(x)` / `ceil(x)` | arrondir en bas / en haut | `floor(2.7)` | `2` |
| `round(x)` | le plus proche, les demis s'éloignent de zéro | `round(2.5)` | `3` |
| `trunc(x)` | supprimer la partie décimale | `trunc(-2.9)` | `-2` |
| `sign(x)` | -1, 0 ou 1 | `sign(-5)` | `-1` |
| `ncr(n, r)` | combinaisons | `ncr(52, 5)` | `2598960` |
| `npr(n, r)` | permutations | `npr(5, 2)` | `20` |
| `gcd(a, b)` / `lcm(a, b)` | diviseurs et multiples communs | `gcd(12, 18)` | `6` |
| `mod(a, b)` | reste | `mod(7, 3)` | `1` |

Les statistiques acceptent un nombre quelconque d'arguments :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `sum(...)` / `product(...)` | totaux | `sum(1, 2, 3)` | `6` |
| `mean(...)` | moyenne | `mean(1, 2, 3)` | `2` |
| `median(...)` | valeur centrale | `median(1, 2, 3, 4)` | `2.5` |
| `min(...)` / `max(...)` | le plus petit / le plus grand | `max(4, 1, 3)` | `4` |
| `variance(...)` / `stdev(...)` | dispersion des valeurs | `stdev(2, 4)` | `1` |

Les couches exactes de la section 1.11 restent :

| Fonction | Signification | Exemple | Résultat |
|---|---|---|---|
| `frac(n, d)` | fraction exacte | `frac(1, 3)` | `1/3` |
| `dec(x)` | décimal exact | `dec(0.1)` | `0.1` |
| `big(x)` | nombre entier exact | `big(10 ^ 20)` | `100000000000000000000` |

Elles se combinent comme tout le reste :

```text
min(sqrt(16), 5)
```

```text
4
```

### 1.13 Lire les erreurs

Quand quelque chose ne va pas, epher vous le dit au lieu de deviner :

```text
1 / 0
```

```text
error: division by zero
```

```text
sqrt(-4)
```

```text
error: domain error: sqrt of negative number -4
```

```text
unknown_name
```

```text
error: unknown name: unknown_name
```

```text
foo(1)
```

```text
error: unknown name: foo
```

Le dernier exemple est important : epher vous dit exactement quel nom il ne
connaît pas, pour que vous puissiez corriger votre expression.

### 1.14 Référence rapide

| Quoi | Syntaxe | Exemple |
|---|---|---|
| Addition, soustraction, multiplication, division | `+ - * /` | `7 / 2` |
| Puissance | `^` (de droite à gauche) | `2 ^ 10` |
| Factorielle | `!` (postfixe) | `5!` |
| Parenthèses | `( )` | `(2 + 3) * 4` |
| Constantes | `pi`, `e`, `tau`, `phi` | `2 * pi` |
| Notation scientifique | `2.5e-3` | `6.02e23` |
| Comparer | `> < >= <= == !=` | `3 >= 2` |
| Logique | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Décision | `if c then a else b` | `if x > 0 then 1 else -1` |
| Boucle | `while c do statement` | `while x < 5 do x = x + 1` |
| Fonction | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | instructions reliées par `;` | `x = 1; x + 1` |
| Fraction exacte | `frac(n, d)` | `frac(1, 3)` |
| Décimal exact | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Nombre entier exact | `big(x)` | `big(10 ^ 20)` |

## 2. L'application web (PWA)

### 2.1 L'ouvrir

L'application web se trouve à l'adresse :

```text
https://upyesp.github.io/epher/pwa/
```

Aucune installation n'est nécessaire — elle fonctionne dans tout navigateur
moderne, sur ordinateur, téléphone ou tablette.

### 2.2 Votre premier calcul

1. Cliquez sur le champ de texte (il est déjà focalisé au chargement).
2. Tapez une expression, par exemple `2 + 3 * 4`.
3. Appuyez sur **Entrée** ou cliquez sur le bouton **=**.

Le résultat apparaît en grand sous le champ. Tout le chapitre 1 fonctionne
ici, y compris variables, fonctions et scripts.

### 2.3 Historique

Chaque calcul est ajouté à la liste d'historique sous le résultat, pour que
vous puissiez remonter et voir ce que vous avez fait. L'historique est
conservé tant que la page est ouverte.

### 2.4 Les graphiques

Tapez `graph` suivi d'une expression et appuyez sur **Entrée** :

```text
graph x ^ 2
```

epher échantillonne la courbe y = f(x) de x = −10 à x = 10 et la dessine
sous le champ de saisie, avec une légende indiquant ce qui est tracé. Vous
pouvez tracer n'importe quelle expression, y compris vos propres
fonctions :

```text
def f(x) = x ^ 3
graph f(x)
```

Les points où l'expression n'a pas de valeur (une division par zéro, par
exemple) sont simplement ignorés, laissant un vide dans la courbe.

### 2.5 L'installer et l'utiliser hors ligne

L'application web est une *progressive web app* : après une visite elle
fonctionne entièrement hors ligne, et vous pouvez l'installer comme une
application normale.

- **Chrome, Edge ou Android :** cliquez sur l'icône d'installation dans la
  barre d'adresse (ou *Installer l'application* dans le menu du navigateur),
  puis confirmez.
- **iPhone / iPad (Safari) :** touchez **Partager** → **Ajouter à l'écran
  d'accueil**.
- **Autres navigateurs :** cherchez *Installer* ou *Ajouter à l'écran
  d'accueil* dans le menu.

Une fois installée, lancez-la depuis votre écran d'accueil ou votre liste
d'applications — elle s'ouvre instantanément, même sans connexion internet.

### 2.6 Ce que l'application web ne fait pas

L'application web est volontairement simple : elle évalue des expressions et
garde un historique de session. Les commandes **save**, **save script** et
**language** fonctionnent dans les versions bureau, ligne de commande et
terminal (chapitres 3, 4 et 5) — dans l'application web, elles répondent
par une note expliquant que l'enregistrement y est possible. L'historique
n'est pas conservé entre les visites.

## 3. L'application de bureau

L'application de bureau est une fenêtre normale autour de la même
application web. Tout le chapitre 2 s'applique ; seule l'installation et le
lancement diffèrent.

### 3.1 Installation

Téléchargez l'application de bureau pour votre système depuis le site web de
epher :

- **Linux (Debian/Ubuntu) :** le paquet `.deb`

```text
sudo apt install ./epher-desktop-linux-x86_64.deb
```

- **Linux (Fedora/RHEL) :** le paquet `.rpm`

```text
sudo dnf install ./epher-desktop-linux-x86_64.rpm
```

- **Linux (toute distribution) :** l'AppImage — rendez-la exécutable et
  lancez-la :

```text
chmod +x epher-desktop-linux-x86_64.AppImage
./epher-desktop-linux-x86_64.AppImage
```

- **macOS :** ouvrez le `.dmg` et glissez epher dans Applications. Comme la
  compilation n'est pas signée, le premier lancement nécessite un clic droit
  → **Ouvrir**.
- **Windows :** lancez l'installateur. Comme la compilation n'est pas
  signée, choisissez *Plus d'informations* → *Exécuter quand même* au
  premier lancement.

### 3.2 Utilisation

Lancez epher comme n'importe quelle application. Vous obtenez une fenêtre
avec la même interface que l'application web : tapez une expression,
appuyez sur **Entrée** ou cliquez sur **=**, et lisez le résultat. Les
graphiques fonctionnent aussi ici — `graph x ^ 2` dessine dans la fenêtre
(chapitre 2.4). La fenêtre se redimensionne librement.

### 3.3 Stockage : un seul magasin partagé avec la CLI et la TUI

L'application de bureau partage son stockage avec les versions ligne de
commande et terminal. Fonctions, scripts, historique et préférence de
langue vivent au même endroit — `~/.epher` sur votre ordinateur (ou
`epher_STORE_DIR`, chapitre 4.5) — et tout ce qui est enregistré dans une
version est disponible dans les autres :

```text
def area(w, h) = w * h
save area
```

Définissez `area` dans l'application de bureau, `save`ez-la, fermez la
fenêtre — puis ouvrez la CLI et `area(3, 4)` fonctionne. Ça marche aussi
dans l'autre sens : les fonctions et scripts enregistrés dans la CLI ou la
TUI sont déjà là à l'ouverture de la fenêtre, y compris les variables
définies par des scripts enregistrés. Les commandes `save`, `save script`
et `language` du chapitre 4 fonctionnent exactement pareil ici.

> L'application web dans le navigateur est la seule version qui n'utilise
> pas ce stockage : chaque session vit isolée (chapitre 2.6).

## 4. La ligne de commande (CLI)

La CLI est la version texte de epher. Elle a deux modes : un mode à usage
unique pour des résultats rapides, et une session interactive pour un travail
plus long.

### 4.1 Calculs à usage unique

Passez l'expression en argument :

```text
epher "2 + 3 * 4"
```

```text
14
```

Vous pouvez faire tout ce qui est une seule expression dans le chapitre 1 :

```text
epher "if 3 > 2 then 10 else 20"
```

```text
10
```

Si votre expression commence par un signe moins, dites à la CLI où commence
l'expression avec `--` :

```text
epher -- "-2 + 5"
```

```text
3
```

Le mode à usage unique évalue exactement une expression. Les instructions —
variables, fonctions, boucles — nécessitent la session interactive.

### 4.2 La session interactive (REPL)

Lancez la session sans argument :

```text
epher
```

epher affiche son invite et attend :

```text
epher>
```

Tapez maintenant n'importe quoi du chapitre 1, une ligne à la fois. Les
variables gardent leur valeur d'une ligne à l'autre :

```text
epher> x = 5
= 5
epher> x ^ 2
= 25
```

Chaque réponse s'affiche sous la forme `= résultat`. Pour quitter, tapez
`quit` (ou `exit`) :

```text
epher> quit
```

Votre historique est mémorisé : la prochaine fois que vous lancez `epher`,
les lignes de la session précédente sont toujours là.

### 4.3 Enregistrer fonctions et scripts

Définissez une fonction, puis enregistrez-la :

```text
epher> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
epher> save fib
saved fib
```

La commande `save fib` enregistre la fonction sur le disque. La prochaine
fois que vous lancez `epher`, `fib` est déjà définie :

```text
epher> fib(10)
= 55
```

Pour enregistrer un script complet (la dernière ligne tapée) utilisez
`save script` :

```text
epher> x = 0; while x < 5 do x = x + 1; x
= 5
epher> save script count_to_five
saved script count_to_five
```

Les scripts enregistrés s'exécutent automatiquement au démarrage de epher,
donc tout ce qu'ils définissent est prêt pour vous.

### 4.4 Changer la langue de l'interface

La langue de l'interface est choisie parmi les langues configurées sur votre
appareil. Pour la remplacer, tapez `language` suivi de l'un de : `en`,
`zh-CN`, `hi`, `es`, `fr`, `ar` :

```text
epher> language fr
language set to fr
```

Le choix est mémorisé pour la prochaine fois. Notez : la langue que vous
*tapez* — le langage des expressions — est toujours la même, quelle que soit
la langue de l'interface.

### 4.5 Où vivent vos données

Les fonctions, scripts, l'historique et votre choix de langue sont stockés
dans un dossier de votre ordinateur :

```text
~/.epher
```

Supprimez ce dossier pour repartir de zéro. Pour utiliser un autre
emplacement, définissez la variable d'environnement `epher_STORE_DIR` avant
de lancer epher :

```text
epher_STORE_DIR=/tmp/my-epher epher
```

## 5. L'interface de terminal (TUI)

La TUI est une version plein écran de la session interactive, dans votre
terminal. Lancez-la avec :

```text
epher-tui
```

### 5.1 L'écran

L'écran est divisé en panneaux :

- **Expression** — la ligne de saisie (en haut).
- Le **résultat** courant juste en dessous.
- **History** — chaque ligne saisie, avec sa réponse.
- **Graph** — le tracé de la commande `graph` (en bas).
- Une ligne d'aide affiche les raccourcis clavier.

### 5.2 Touches

| Touche | Action |
|---|---|
| Taper | ajouter à l'expression |
| **Entrée** | évaluer |
| **Échap** | effacer la ligne de saisie |
| **Ctrl+C** | quitter |
| **q** | quitter (quand la saisie est vide) |

### 5.3 Les graphiques

Tapez `graph` suivi d'une expression, puis appuyez sur **Entrée** :

```text
graph x ^ 2
```

epher échantillonne la courbe de x = −10 à x = 10 et la dessine sous forme de
graphique ASCII dans le panneau Graph. La légende au-dessus du tracé montre
ce qui est tracé : `y = x ^ 2`.

Vous pouvez tracer n'importe quelle expression, y compris vos propres
fonctions — définissez-en d'abord une, puis tracez-la :

```text
def f(x) = x ^ 3
graph f(x)
```

Les points où l'expression n'a pas de valeur (par exemple la division par
zéro) sont simplement ignorés, laissant un vide dans le tracé.

### 5.4 Enregistrement et persistance

La TUI partage son stockage avec la CLI : tout ce qui est enregistré dans
l'une est disponible dans l'autre. Les fonctions, scripts, historique et la
préférence de langue vivent dans `~/.epher` (chapitre 4.5), et les mêmes
commandes `save`, `save script` et `language` fonctionnent ici.

## 6. Vos données et la vie privée

- La **CLI et la TUI** stockent fonctions, scripts, historique et choix de
  langue localement dans `~/.epher` (ou `epher_STORE_DIR`). Rien ne quitte
  votre ordinateur.
- L'**application web** ne stocke rien sur le disque : l'historique ne dure
  que tant que la page est ouverte. L'application web peut fonctionner hors
  ligne parce que c'est votre navigateur qui stocke la page elle-même.
- L'**application de bureau** enregistre fonctions, scripts, historique et
  choix de langue localement dans `~/.epher` (ou `epher_STORE_DIR`), le même
  magasin que la CLI et la TUI. Rien ne quitte votre ordinateur.

Les quatre versions exécutent le calcul entièrement sur votre appareil —
rien n'est envoyé nulle part.
