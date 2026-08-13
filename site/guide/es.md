# Guía de usuario de Calc

¡Bienvenido! Calc es una calculadora programable y con scripts. Puedes usarla
para un cálculo rápido o para construir tus propias funciones y pequeños
programas, y todo está disponible en seis idiomas.

Esta guía es para principiantes absolutos. Empieza con el cálculo más simple
posible y llega hasta todo el poder del lenguaje. Cada ejemplo muestra lo que
escribes y lo que Calc responde.

Hay cuatro formas de usar Calc — elige la que más te convenga:

| Versión | Qué es | Cuándo conviene |
|---|---|---|
| **Aplicación web** (PWA) | Se ejecuta en tu navegador, se puede instalar y funciona sin conexión | Quieres empezar rápido; sin instalación |
| **Aplicación de escritorio** | Un programa normal con su propia ventana | Quieres una aplicación de escritorio |
| **Línea de comandos** (CLI) | Comandos de texto en una terminal; también una sesión interactiva | Vives en la terminal y te gustan los scripts |
| **Interfaz de terminal** (TUI) | Un programa a pantalla completa dentro de la terminal | Quieres una app de terminal con gráficos e historial |

Las cuatro versiones entienden exactamente el mismo lenguaje. Apréndelo una
vez, úsalo en cualquier parte.

## 1. El lenguaje de Calc

Este capítulo enseña el lenguaje compartido por todas las versiones de Calc.
En la aplicación web o de escritorio, escribe una expresión y pulsa
**Intro** (o haz clic en el botón **=**). En la CLI, escríbela después del
prompt `calc>`. En la TUI, solo escribe y pulsa **Intro**. En la CLI también
puedes escribir `calc "expresión"` para evaluar una expresión directamente.

### 1.1 Tu primer cálculo

Escribe esto:

```text
2 + 3 * 4
```

Calc responde:

```text
14
```

La multiplicación se hace antes que la suma, exactamente como en
matemáticas. Esa regla se llama *precedencia de operadores*.

### 1.2 Orden de las operaciones

El orden completo de precedencia, de más fuerte a más débil:

1. `^` potencia
2. `*` y `/` multiplicación y división
3. `+` y `-` suma y resta

Usa paréntesis para cambiar el orden:

```text
(2 + 3) * 4
```

```text
20
```

El operador `^` calcula potencias y funciona de derecha a izquierda:

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

(`2 ^ 3 ^ 2` significa `2 ^ (3 ^ 2)`, es decir `2 ^ 9` = 512.)

Las potencias pueden ser fraccionarias — `2 ^ 0.5` es la raíz cuadrada de 2:

```text
2 ^ 0.5
```

```text
1.4142135623730951
```

La resta y la división funcionan de izquierda a derecha:

```text
10 - 3 - 2
```

```text
5
```

### 1.3 Los números especiales pi y e

Las dos constantes famosas están integradas:

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

### 1.4 Comparar y lógica

Puedes comparar números. El resultado es `true` (verdadero) o `false`
(falso):

| Comparación | Significado |
|---|---|
| `a > b` | a es mayor que b |
| `a < b` | a es menor que b |
| `a >= b` | a es mayor o igual que b |
| `a <= b` | a es menor o igual que b |
| `a == b` | a es igual a b (nota el doble `=`) |
| `a != b` | a no es igual a b |

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

Combina comparaciones con `and`, `or` y `not`:

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

Dale un nombre a un valor con un solo `=`:

```text
x = 5
```

```text
5
```

Calc te repite el valor. Desde ahora, `x` se puede usar en cualquier parte:

```text
x ^ 2
```

```text
25
```

Puedes cambiar una variable cuando quieras — conserva su valor hasta que la
cambies:

```text
x = x + 1
```

```text
6
```

> Los nombres pueden contener letras y guiones bajos, como `radius` o
> `my_total`. No pueden contener espacios ni empezar por un número.

### 1.6 Decisiones con if

`if` elige entre dos valores:

```text
if 3 > 2 then 10 else 20
```

```text
10
```

La forma es siempre `if condición then valor_si_verdadero
else valor_si_falso`. La parte `else` es obligatoria.

Un ejemplo más útil con una variable:

```text
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> Calc no tiene valores de texto — ambas ramas de un `if` deben ser números
> (o resultados de comparaciones).

### 1.7 Bucles con while

`while` repite una instrucción mientras se cumpla una condición:

```text
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Lee ese script así: *empieza x en 0; mientras x sea menor que 5, suma 1 a x;
luego muestra x.* El resultado es 5 porque el bucle se ejecutó cinco veces.

> **Red de seguridad:** Calc detiene cualquier bucle después de 100 000
> pasos y muestra `error: step limit exceeded`. Eso te protege de bucles que
> nunca terminarían. Si lo ves, tu condición probablemente nunca se volvió
> falsa.

### 1.8 Tus propias funciones con def

Una función es un cálculo con nombre y parámetros:

```text
def f(x) = x ^ 2
```

Luego úsala:

```text
f(7)
```

```text
49
```

Las funciones pueden tener varios parámetros:

```text
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

También puedes definir una función sin parámetros:

```text
def answer() = 42
answer()
```

```text
42
```

### 1.9 Recursión: una función que se llama a sí misma

El ejemplo más famoso — los números de Fibonacci:

```text
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```text
fib(10)
```

```text
55
```

`fib(10)` es el décimo número de Fibonacci. La función se llama a sí misma
con argumentos más pequeños hasta llegar a `n <= 1`. Esto funciona porque la
forma `if ... then ... else ...` solo calcula la rama que necesita.

> El cuerpo de una función es una sola expresión — una línea. Combina
> varios cálculos con `;` en un script en su lugar (sección siguiente).

### 1.10 Scripts: varias instrucciones a la vez

Un *script* es varias instrucciones unidas con `;`, ejecutadas una tras
otra:

```text
x = 10; y = x + 5; x + y
```

```text
25
```

Los scripts son la forma de construir pequeños programas: prepara
variables, haz bucles y muestra un resultado final.

### 1.11 Resultados exactos: frac, dec y big

Normalmente Calc calcula con números decimales como una calculadora de
bolsillo. Algunos números se ven mejor exactos.

**frac(n, d)** crea una fracción exacta:

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

Las fracciones se mantienen exactas a través de los cálculos:

```text
frac(1, 3) * 3
```

```text
1
```

**dec(x)** crea un decimal exacto. Compara estos dos:

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

El primer resultado es el pequeño error de redondeo que toda computadora
comete con los decimales. `dec()` lo elimina.

**big(x)** crea un número entero exacto, para valores demasiado grandes para
una calculadora de bolsillo:

```text
big(10 ^ 20)
```

```text
100000000000000000000
```

### 1.12 Funciones integradas

Calc tiene un pequeño conjunto de funciones integradas:

| Función | Significado | Ejemplo | Resultado |
|---|---|---|---|
| `sqrt(x)` | raíz cuadrada | `sqrt(16)` | `4` |
| `min(a, b)` | el menor de dos | `min(3, 7)` | `3` |
| `frac(n, d)` | fracción exacta | `frac(1, 3)` | `1/3` |
| `dec(x)` | decimal exacto | `dec(0.1)` | `0.1` |
| `big(x)` | número entero exacto | `big(10 ^ 20)` | `100000000000000000000` |

Se combinan como todo lo demás:

```text
min(sqrt(16), 5)
```

```text
4
```

### 1.13 Leer los errores

Cuando algo sale mal, Calc te lo dice en lugar de adivinar:

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
sin(1)
```

```text
error: unknown name: sin
```

El último ejemplo es importante: `sin` **no** está integrado — solo las
funciones de la sección 1.12. El mensaje de error te dice exactamente qué
Calc no conoce, para que puedas arreglar tu expresión.

### 1.14 Referencia rápida

| Qué | Sintaxis | Ejemplo |
|---|---|---|
| Sumar, restar, multiplicar, dividir | `+ - * /` | `7 / 2` |
| Potencia | `^` (de derecha a izquierda) | `2 ^ 10` |
| Paréntesis | `( )` | `(2 + 3) * 4` |
| Constantes | `pi`, `e` | `2 * pi` |
| Comparar | `> < >= <= == !=` | `3 >= 2` |
| Lógica | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Decisión | `if c then a else b` | `if x > 0 then 1 else -1` |
| Bucle | `while c do statement` | `while x < 5 do x = x + 1` |
| Función | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | instrucciones unidas con `;` | `x = 1; x + 1` |
| Fracción exacta | `frac(n, d)` | `frac(1, 3)` |
| Decimal exacto | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Número entero exacto | `big(x)` | `big(10 ^ 20)` |

## 2. La aplicación web (PWA)

### 2.1 Cómo abrirla

La aplicación web está en:

```text
https://upyesp.github.io/calc/pwa/
```

No necesita instalación — funciona en cualquier navegador moderno, en
ordenador, móvil o tableta.

### 2.2 Tu primer cálculo

1. Haz clic en el campo de texto (ya está enfocado cuando la página carga).
2. Escribe una expresión, por ejemplo `2 + 3 * 4`.
3. Pulsa **Intro** o haz clic en el botón **=**.

El resultado aparece en texto grande debajo del campo. Todo lo del capítulo
1 funciona aquí, incluidas variables, funciones y scripts.

### 2.3 Historial

Cada cálculo se añade a la lista de historial debajo del resultado, para que
puedas desplazarte hacia atrás y ver lo que hiciste. El historial se
conserva mientras la página está abierta.

### 2.4 Instalarla y usarla sin conexión

La aplicación web es una *progressive web app*: después de una visita
funciona completamente sin conexión y puedes instalarla como una app normal.

- **Chrome, Edge o Android:** haz clic en el icono de instalar de la barra
  de direcciones (o *Instalar aplicación* en el menú del navegador) y
  confirma.
- **iPhone / iPad (Safari):** toca **Compartir** → **Añadir a pantalla de
  inicio**.
- **Otros navegadores:** busca *Instalar* o *Añadir a pantalla de inicio*
  en el menú.

Una vez instalada, ábrela desde tu pantalla de inicio o lista de apps — se
abre al instante, incluso sin conexión a internet.

### 2.5 Lo que la aplicación web no hace

La aplicación web es intencionadamente simple: evalúa expresiones y guarda
un historial de sesión. Los comandos **save**, **save script** y
**language** existen en la línea de comandos y en la versión de terminal
(capítulos 4 y 5). El historial no se guarda entre visitas.

## 3. La aplicación de escritorio

La aplicación de escritorio es una ventana normal alrededor de la misma
aplicación web. Todo lo del capítulo 2 se aplica; la diferencia está solo en
cómo la instalas y la abres.

### 3.1 Instalación

Descarga la aplicación de escritorio para tu sistema desde el sitio web de
Calc:

- **Linux (Debian/Ubuntu):** el paquete `.deb`

```text
sudo apt install ./calc-desktop-linux-x86_64.deb
```

- **Linux (Fedora/RHEL):** el paquete `.rpm`

```text
sudo dnf install ./calc-desktop-linux-x86_64.rpm
```

- **Linux (cualquier distro):** el AppImage — hazlo ejecutable y ejecútalo:

```text
chmod +x calc-desktop-linux-x86_64.AppImage
./calc-desktop-linux-x86_64.AppImage
```

- **macOS:** abre el `.dmg` y arrastra Calc a Aplicaciones. Como la compilación
  no está firmada, el primer arranque necesita clic derecho → **Abrir**.
- **Windows:** ejecuta el instalador. Como la compilación no está firmada,
  elige *Más información* → *Ejecutar de todas formas* en el primer arranque.

### 3.2 Uso

Inicia Calc como cualquier otra aplicación. Obtienes una ventana con la misma
interfaz que la aplicación web: escribe una expresión, pulsa **Intro** o
haz clic en **=**, y lee el resultado. La ventana se puede redimensionar
libremente.

## 4. La línea de comandos (CLI)

La CLI es la versión de texto de Calc. Tiene dos modos: un modo de un solo
uso para resultados rápidos y una sesión interactiva para trabajos más
largos.

### 4.1 Cálculos de un solo uso

Pasa la expresión como argumento:

```text
calc "2 + 3 * 4"
```

```text
14
```

Puedes hacer cualquier cosa del capítulo 1 que sea una sola expresión:

```text
calc "if 3 > 2 then 10 else 20"
```

```text
10
```

Si tu expresión empieza con un signo menos, dile a la CLI dónde empieza la
expresión con `--`:

```text
calc -- "-2 + 5"
```

```text
3
```

El modo de un solo uso evalúa exactamente una expresión. Las instrucciones —
variables, funciones, bucles — necesitan la sesión interactiva.

### 4.2 La sesión interactiva (REPL)

Inicia la sesión sin argumentos:

```text
calc
```

Calc muestra su prompt y espera:

```text
calc>
```

Ahora escribe cualquier cosa del capítulo 1, una línea cada vez. Las
variables conservan sus valores entre líneas:

```text
calc> x = 5
= 5
calc> x ^ 2
= 25
```

Cada respuesta se muestra como `= resultado`. Para salir, escribe `quit` (o
`exit`):

```text
calc> quit
```

Tu historial se recuerda: la próxima vez que inicies `calc`, las líneas de
la sesión anterior siguen ahí.

### 4.3 Guardar funciones y scripts

Define una función y luego guárdala:

```text
calc> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
calc> save fib
saved fib
```

El comando `save fib` guarda la función en el disco. La próxima vez que
inicies `calc`, `fib` ya está definida:

```text
calc> fib(10)
= 55
```

Para guardar un script completo (la última línea que escribiste) usa
`save script`:

```text
calc> x = 0; while x < 5 do x = x + 1; x
= 5
calc> save script count_to_five
saved script count_to_five
```

Los scripts guardados se ejecutan automáticamente cuando Calc arranca, así
que todo lo que definen está listo para ti.

### 4.4 Cambiar el idioma de la interfaz

El idioma de la interfaz se elige entre los idiomas configurados en tu
dispositivo. Para cambiarlo, escribe `language` seguido de uno de: `en`,
`zh-CN`, `hi`, `es`, `fr`, `ar`:

```text
calc> language fr
language set to fr
```

La elección se recuerda para la próxima vez. Nota: el idioma que *escribes*
— el lenguaje de las expresiones — es siempre el mismo, en cualquier idioma
de la interfaz.

### 4.5 Dónde viven tus datos

Las funciones, scripts, historial y tu elección de idioma se guardan en una
carpeta de tu ordenador:

```text
~/.calc
```

Borra esa carpeta para empezar completamente de cero. Para usar otra
ubicación, define la variable de entorno `CALC_STORE_DIR` antes de iniciar
Calc:

```text
CALC_STORE_DIR=/tmp/my-calc calc
```

## 5. La interfaz de terminal (TUI)

La TUI es una versión a pantalla completa de la sesión interactiva, dentro
de tu terminal. Iníciala con:

```text
calc-tui
```

### 5.1 La pantalla

La pantalla está dividida en paneles:

- **Expression** — la línea de entrada (arriba).
- El **resultado** actual justo debajo.
- **History** — cada línea que escribiste, con su respuesta.
- **Graph** — la gráfica del comando `graph` (abajo).
- Una línea de pistas muestra los atajos de teclado.

### 5.2 Teclas

| Tecla | Acción |
|---|---|
| Escribir | añadir a la expresión |
| **Intro** | evaluar |
| **Esc** | borrar la línea de entrada |
| **Ctrl+C** | salir |
| **q** | salir (cuando la entrada está vacía) |

### 5.3 Gráficas

Escribe `graph` seguido de una expresión y pulsa **Intro**:

```text
graph x ^ 2
```

Calc muestrea la curva de x = −10 a x = 10 y la dibuja como una gráfica
ASCII en el panel Graph. El título sobre la gráfica muestra lo que se
representa: `y = x ^ 2`.

Puedes graficar cualquier expresión, incluidas tus propias funciones —
primero define una y luego grafícala:

```text
def f(x) = x ^ 3
graph f(x)
```

Los puntos donde la expresión no tiene valor (por ejemplo división entre
cero) simplemente se omiten, dejando un hueco en la gráfica.

### 5.4 Guardar y persistencia

La TUI comparte su almacenamiento con la CLI: todo lo guardado en una está
disponible en la otra. Las funciones, scripts, historial y preferencia de
idioma viven en `~/.calc` (capítulo 4.5), y los mismos comandos `save`,
`save script` y `language` funcionan aquí.

## 6. Tus datos y privacidad

- La **CLI y la TUI** guardan funciones, scripts, historial y la elección de
  idioma localmente en `~/.calc` (o `CALC_STORE_DIR`). Nada sale de tu
  ordenador.
- La **aplicación web** no guarda nada en disco: el historial dura solo
  mientras la página está abierta. La aplicación web puede funcionar sin
  conexión porque la propia página la guarda tu navegador.
- La **aplicación de escritorio** es la aplicación web en una ventana;
  tampoco guarda datos.

Las cuatro versiones ejecutan el cálculo íntegramente en tu dispositivo —
nada se envía a ningún sitio.
