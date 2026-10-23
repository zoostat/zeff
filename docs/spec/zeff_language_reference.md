# Zeff Language Reference

Version of this document: 17 October 2026 (audited; revisions so far: storage formats and the Parquet profile, a 1.0 limited to linear and binary models, `fe:`, `re:` and `instruments:` arguments, failure handling, file schemas, shared memory, data roots, accuracy classes, names and case rules, engine architecture, language at a glance, definitions with `=`, causal graphs, model-matrix rules, contrasts, random-number generator, interoperability, package bundle and release plan, lexical rules and limits, the command line and installation, column types and metadata, reserved types and figure rendering) · Engine today: none; the code starts afresh at 0.1.0 (section 14 of the Engine and Project Specification)

Zeff is a statistics language for empirical and causal research. The Zeff Specification is published in two volumes with section numbers shared between them; each volume is edited directly, and the text both carry (the status paragraph and the headers) is kept identical by hand. This volume, the **Language Reference**, holds what an implementation of the language must do and what a script may rely on: the language at a glance (1.5), syntax (3), types and values (4), control flow, functions and errors (5), frames and their verbs (6), the declarative forms: formulas, model matrices, causal graphs, model blocks, recipes and plots (7), a complete analysis with its printed output (10), diagnostics (11) and the glossary (16). The companion volume, the **Engine and Project Specification**, holds the purpose, principles and constraints (1.1 to 1.4), projects and the command line (2), packages, tiers and the engine (8), the function catalogue and the help contract (9), editor support (12), accuracy and validation (13), the build order and release plan (14) and the open decisions (15). A reference to a section of the other volume says so.

**Status of this document.** The rules for the language, types, frames, formulas, results, diagnostics and projects that apply to 1.0 are normative: an implementation of Zeff 1.0 must follow them. Anything marked with a later release ("from 1.1", "after 1.0"), "proposed" or "roadmap" is a plan, not yet a requirement, and the open decisions of section 15 of the Engine and Project Specification are unsettled. Code examples and printed outputs are illustrative: their numbers are invented, but their form is normative.

**Conformance.** Zeff has no implementation-defined, unspecified or undefined behaviour: every operation's result is fixed by this document or by the function's entry in zeff_functions.csv, and every failure is a named diagnostic (section 11). The grammar of section 3.4 is normative for syntax and the prose for meaning; where two readings of this document conflict, the conformance suite (section 13 of the Engine and Project Specification) decides and the document is corrected in the next release.

---

## 1. What Zeff is

Sections 1.1 to 1.4, the users, principles, non-goals, licence and technical constraints, are in the Engine and Project Specification; this volume begins with the language at a glance.

### 1.5 Language at a glance

Zeff is a statically typed, expression-oriented, mostly functional language for statistics, with method-call syntax. It reads like an object-oriented language (`wages.reg(f)`) but has no classes, inheritance or objects that change state: a method call is another way to write a function call, and the function is chosen by the type of its first argument.

| Characteristic | Zeff | Compared with R and Stata |
| --- | --- | --- |
| Paradigm | Mostly functional: functions return new values and never change anything outside themselves | R mixes functional code with three object systems (S3, S4, R5); Stata is a command language |
| Purpose | Domain-specific: statistics only (section 1.3) | R is general-purpose |
| Data | Immutable: `let` bindings never change; a later `let` shadows the earlier one | Both change data in place |
| Typing | Static: names, arguments, types and columns are checked before anything runs, against cached file schemas; types are inferred except in function signatures | R is dynamic and finds errors mid-run |
| Type strength | Strong: no silent conversions; failed conversions become `NA` with a warning; text never becomes a category on its own | Both coerce quietly |
| Polymorphism | One function per name per type, chosen by the type of the first argument and fixed when the script is checked (section 8.1 of the Engine and Project Specification, rule 6) | R's S3 dispatch is similar but can change at run time |
| Objects | No classes or inheritance. Built-in types implement shared interfaces in Rust (the result and estimator interfaces of `results`, backed by the estimator trait); user-defined types come after 1.0 (section 15 of the Engine and Project Specification) | R has classes; Stata has none |
| Expressions | `if` and `case` return values, as in Rust | |
| Vectorised | Operators on columns work row by row: `wage > 20` is a Bool column | As R |
| Declarative parts | Formulas, `model { }` blocks and `dag { }` causal graphs describe a model; the engine decides how to fit it | R takes lavaan models as strings |
| Missing values | One `NA`, three-valued logic, every dropped row counted (section 4.1) | R has `NA` and `NaN`; Stata's missing counts as larger than any number |
| Definitions and options | `=` creates a name (`add(lwage = log(wage))`), `:` sets an option (`se: robust`) | R uses `=` for both; Stata uses `generate` and commas |
| Errors | No exceptions: an error stops the script; `attempt` turns one into a `Failed` value, like Rust's `Result` | R has `tryCatch`, Stata `capture` |
| Termination | No `while` and no recursion: every loop has a known length and `iterate` needs a finite `max:`, so a script always finishes | |
| Functions | First-class but named; closures after 1.0 | R has full closures |
| Reproducibility | Seeds always recorded; bit-identical results on every platform (section 8.3 of the Engine and Project Specification) | Neither guarantees this |
| Execution | Interpreted from a typed intermediate representation; every built-in method is compiled Rust | R is interpreted over C; Stata is closed-source C |
| Indexing and case | Positions start at 1; names are case-sensitive (section 3.6) | As R |

---

---

## 3. Syntax

Zeff borrows Rust's surface (`let`, `.method()` chains, `name: value` arguments, `//` comments) and keeps R's `~` formulas inside calls. There are no semicolons, no `mut`, no type annotations outside function signatures and no `?` operator.

```
let wages = import("data/wages.csv")
    .where(year >= 2010)
    .add(lwage = log(wage), exper2 = exper^2)

wages.summary([lwage, educ, exper])                     // printed: not a `let`
let m = wages.reg(lwage ~ educ + exper, fe: firm, cluster: firm)
report([m]).export("output/table1.docx")
```

### 3.1 Rules

| Element | Rule |
| --- | --- |
| Statements | The lexer emits a NEWLINE token at every line break, collapsing blank and comment-only lines and supplying one at the end of the file. The innermost open bracket decides what a NEWLINE means: inside `( )`, `[ ]`, a record literal or `case { }` it means nothing and items are separated by commas; inside a block, `model { }` or `dag { }` it ends a statement or line, even when an outer `[` is open (`[for ... { a NEWLINE b }]`). At the top level and in a block, a NEWLINE ends a statement unless the line ends with a comma, a binary operator (`~`, `=~`, `~~`, `:`, `as` and `..` included), `=`, `=>`, `->` or `::`, or the next line that is not blank or a comment begins with `.` followed by a name (a line beginning with `..` is a spread, not a continuation). A line may not end with `.` (Z0312). An `else` belongs to an `if` only when it follows the `}` on the same line. A statement may not be a bare unary-minus expression or begin with `+`, `*`, `/`, `~`, `&&` or `\|\|`: `zeff check` reports Z0305 ("this line begins with an operator; did you mean to continue the previous line? End that line with the operator"), which catches a wrapped formula whose `- female` would otherwise become a separate statement. `zeff fmt` breaks long expressions and formulas after an operator, never before one |
| Value of a block | A block's value is the value of its last statement when that statement is an expression; when it is a `let`, a `for` without brackets, `break`, `continue` or `return`, or the block is empty, the value is `Nothing` (section 3.7) |
| Bindings | `let name = expr` binds a value; bindings never change. A later `let` with the same name shadows the earlier one |
| Printing | A statement that is not a `let` prints its value; a frame prints its `describe` header and first ten rows |
| Calls | Arguments are positional, in parameter order, or `name: value`; once one is named, the rest must be. An unknown name or a repeated argument is an error. Argument names and record field names may be any of the 26 reserved words, the eight reserved for later included (`model:`, `case:`, `type:`), since they always sit before a colon |
| Definitions | `name = expr` creates a name; `name: value` sets an option. In `add`, `update`, `aggregate`, `rename`, `derive` and `frame`, each `name = expr` defines a column, a new variable name or a derived quantity, while `by:` and `where:` set options: `wages.add(dev = lwage - mean(lwage), by: firm)`, `wages.rename(gross_wage = wage)`. Option records (`{se: robust}`, `types: {hours: Str}`) keep `:`, and model blocks keep lavaan's `:=` |
| Methods | `x.f(a)` is the same call as `f(x, a)`, for every function, built in or user-written. `dist.`, `power.` and `spec.` are namespaces, not values: `dist.normal()` names a function |
| Qualified names | `package::name` picks a function when two share a name: `functions::summary`, `data::summary` |
| Bare words | The parameter decides: `cluster: firm` expects a variable, `se: robust` a value from a fixed list, `fe: firm + year` formula terms, `re: [school => 1 + x]` a mapping from groups to terms; all are checked before anything runs |
| Names | Case-sensitive; ASCII letters, digits and `_`; file names that are not valid names in backticks (section 3.6) |
| Comments | `//` to the end of the line; `///` directly before a function becomes its help text (section 5.3); `// zeff:` begins a pragma (section 11.1). There are no block comments |
| Indexing | Positions start at 1: `fits[1]`, and `slice(s, 1, 3)` for text; a frame's column by name: `wages["lwage"]`; results by term name, never by position: `m.coef(educ)`, `m.std_errors(educ)`, `t.p_values()`; a record's field by name: `state.loglik` |
| Reserved words | `let`, `fn`, `if`, `else`, `case`, `for`, `in`, `break`, `continue`, `return`, `use`, `model`, `dag`, `as`, `true`, `false`, `NA`, `Inf`; and, reserved for later use so that no edition is needed to add them, `type`, `while`, `match`, `mut`, `pub`, `loop`, `trait`, `impl` (section 3.6) |

### 3.2 Literals

`42` (Int), `0.05` and `1e-6` (Float), `"text"` (Str, with `\n`, `\t`, `\"`, `\\` and `\u{XXXX}` escapes), `true` and `false`, `NA`, `Inf`, `[a, b, c]` (List), `{ se: robust, cluster: firm }` (Record), `[0 => "male", 1 => "female"]` (Mapping), `2..5` (Range of numbers) and `x1..x9` (Vars: the variables from `x1` to `x9` by position in the frame, section 3.7). Both ends of a range are included.

| Rule | Detail |
| --- | --- |
| Int or Float | A number with a decimal point or an exponent is a Float, so `1e6` and `1e+6` are Floats and `1000000` an Int; Int and Float are both 64-bit (section 4). A Float literal denotes the nearest IEEE binary64 value (ties to even), so `0.1` is the same number on every platform |
| Malformed numbers | `.5`, `5.` and a leading zero before another digit (`007`) are errors that show the accepted form (Z0310); an integer literal beyond 64 bits is a check-time error (Z0310), while `-9223372036854775808` is accepted as the smallest Int; `_` may stand only between two digits (`1_000_000`, `0.000_001`) |
| Strings | Double quotes only; the escapes are `\n`, `\t`, `\r`, `\"`, `\\` and `\u{XXXX}` (one to six hex digits) for one Unicode scalar value. Any other backslash is an error (Z0311) whose help names the raw form: a raw string `r"\d+"` has no escapes and ends at the first `"`, and is the form for regular expressions and Windows paths (`r"D:\surveys"`; forward slashes work everywhere). A string does not span lines (Z0311); there is no interpolation, `format` and `join` build text |
| Ranges | `2..5` is an Int range with step 1, both ends included; a range whose end is before its start, `5..2`, is empty; `x1..x9` is a variable range (section 3.7) |

### 3.3 Operators

From tightest to loosest binding:

| Level | Operators | Notes |
| --- | --- | --- |
| 1 | `x.f()`, `f()`, `x[i]` | Calls and indexing |
| 2 | `^` | Power, right-associative and tighter than unary minus: `2^3^2` is 512 and `-2^2` is -4 |
| 3 | `-x`, `!x` | Negation, logical not |
| 3.5 | `:` | Interaction; allowed only inside a formula or a `Terms` argument, otherwise error Z0306 |
| 4 | `*`, `/` | `/` always returns a Float; integer division is `div(a, b)`. Inside a formula, `*` is expansion (`a * b` is `a + b + a:b`) |
| 4.5 | `as` | Label, in model blocks only: `dem60 ~ ind60 as a`; elsewhere error Z0308 |
| 5 | `+`, `-` | |
| 6 | `..` | Range |
| 7 | `==`, `!=`, `<`, `<=`, `>`, `>=` | Do not chain: `a < b < c` is an error (Z0314), as is `1..2..3` or `y ~ x ~ z` |
| 8 | `&&` | Three-valued with NA: `NA && false` is `false` |
| 9 | `\|\|` | `NA \|\| true` is `true` |
| 10 | `~`, `=~`, `~~` | Formula and model operators, loosest of all; `~` builds a `Formula` value wherever it appears, while `=~`, `~~` and `:=` belong inside `model { }` (Z0308 elsewhere) |

When the `Matrix` type arrives (section 4.3), `*` between two matrices is the matrix product and elementwise operations are methods, so no operator changes meaning. Every binary operator is left-associative except `^`, which is right-associative; `-x^2` is `-(x^2)` and `!a && b` is `(!a) && b`. Tokens from other languages are syntax errors with a help line naming the Zeff form (Z0315): `;`, `&`, `|`, `**`, `%`, `<-`, `TRUE`, `FALSE`, `NULL`, `None`, and `=` where a comparison was meant. On columns every operator works row by row: `wage > 20` is a Bool column. A leading `..` spreads a record into a call: `wages.reg(f, ..opts)`, where an argument written out overrides the spread one. The parser builds one expression tree for every argument, with the operators above; the typed IR gives the tree its meaning from the parameter's type (section 7). In a `Formula` or `Terms`, `*` is expansion, `:` interaction, `^` and arithmetic functions compute columns before fitting, and `-` removes a term; in any other parameter the same tree is arithmetic and `:` is an error. The grammar therefore needs no knowledge of function signatures, and a misspelt function name gives one error, not a cascade.

### 3.4 Grammar

Notation: `{ x }` is zero or more, `[ x ]` optional, `|` a choice, quoted text a literal token and an upper-case word a token class (`NAME`, `NUMBER`, `STRING`, `TEXT`, `NEWLINE`). Blank lines and `//` comments are removed by the lexer before the grammar applies; `///` lines attach to the `fn` that follows. The grammar is normative for syntax (section 1).

```
script      = { NEWLINE } { ( statement | fn_def | "use" qname ) NEWLINE { NEWLINE } }
            // fn and use only at the top level of a file (Z0309); use before the first statement
statement   = "let" NAME "=" expr | for_loop
            | "return" [ expr ] | "break" | "continue" | expr
            // return only in a function, break and continue only in a loop (Z0313)
fn_def      = { DOC } "fn" NAME "(" [ params ] ")" "->" type block
params      = param { "," param }
param       = NAME ":" type [ "=" expr ]
type        = simple [ "?" ] | words | fn_type | "Nothing"
simple      = qname [ "<" type { "," type } ">" ]
            | "{" NAME ":" type { "," NAME ":" type } "}"
            // a record type: fields and their types
words       = NAME "|" NAME { "|" NAME }                               // a fixed list of bare words
fn_type     = "Fn" "(" [ type { "," type } [ "," ".." ] ] ")" "->" type
            // a trailing .. accepts further parameters, checked at each call against the
            // record that supplies them, as iterate's args: does (section 5.5)
for_loop    = "for" NAME "in" expr block
block       = "{" { NEWLINE } { statement NEWLINE { NEWLINE } } "}"    // value: section 3.1
expr        = formula
formula     = or_expr [ ( "~" | "=~" | "~~" ) or_expr ] | "~" or_expr
            // level 10; no chaining (Z0314); the one-sided form, `~ z`, only where a
            // parameter takes a one-sided Formula, such as `zero:` (Z0322 elsewhere)
or_expr     = and_expr { "||" and_expr }                               // left
and_expr    = cmp_expr { "&&" cmp_expr }                               // left
cmp_expr    = range_expr [ CMP_OP range_expr ]                         // no chaining (Z0314)
range_expr  = add_expr [ ".." add_expr ]                               // no chaining
add_expr    = as_expr { ( "+" | "-" ) as_expr }                        // left
as_expr     = mul_expr [ "as" NAME ]
            // one label, model blocks only (Z0308)
mul_expr    = inter_expr { ( "*" | "/" ) inter_expr }                  // left
inter_expr  = unary { ":" unary }
            // left; formulas and Terms only (Z0306)
unary       = ( "-" | "!" ) unary | power
power       = postfix [ "^" unary ]                                    // right
postfix     = primary { "." qname "(" [ args ] ")" | "." NAME | "[" expr "]" }
            // a qualified method, data.survival::coxph(...), names a package's function (section 8.4 of the Engine and Project Specification)
            // "." NAME without "(" reads a record field; on any other value error Z0213
            // dist.normal(...), power.t_test(...) and spec.lasso(...) have this shape; the checker knows them
            // from the function registry (section 3.7)
primary     = literal | qname | qname "(" [ args ] ")" | "(" expr ")"
            | if_expr | case_expr | model_block | dag_block | "[" for_loop "]" | "."
            // "." alone: every other column, only as a term on the right of ~ in the
            // formulas of learn and tune (section 7.5); elsewhere Z0321
if_expr     = "if" expr block [ "else" ( if_expr | block ) ]
            // else on the line of the closing brace
case_expr   = "case" "{" arm { "," arm } [ "," ] "}"
            // line breaks inside mean nothing
arm         = ( expr | "_" ) "=>" ( expr | block )                     // "_" once, last (Z0316)
qname       = NAME { "::" NAME }
literal     = NUMBER | STRING | RAW_STRING | "true" | "false" | "NA" | "Inf"
            | "[" [ expr { "," expr } [ "," ] ] "]"
            | "[" pair { "," pair } [ "," ] "]"
            | "{" [ field { "," field } [ "," ] ] "}"
pair        = expr "=>" expr
            // keys and values may be terms, since `:` is an operator
field       = ( NAME | KEYWORD ) ":" expr | ".." expr
args        = arg { "," arg } [ "," ]
arg         = ( NAME | KEYWORD ) ":" expr | NAME "=" expr | ".." expr | expr
            // NAME "=" expr: a definition (add, update, aggregate, rename, derive, frame)
            // every argument is one expression tree; the parameter's type gives it formula,
            // Terms or arithmetic meaning (section 3.3); `|` never appears
            // NAME ":" at the start of an argument is always an option, so an interaction
            // passed positionally is written in brackets: (firm:year)
model_block = "model" "{" { NEWLINE } { model_line NEWLINE { NEWLINE } } "}"
model_line  = formula | NAME ":=" add_expr
            // a formula whose operator is =~, ~ or ~~
            | "efa" "(" NAME { "," NAME } ")" "=~" or_expr
dag_block   = "dag" "{" { NEWLINE } { dag_line NEWLINE { NEWLINE } } "}"   // section 7.3
dag_line    = node { ( "->" | "<->" | "--" ) node } | ( "latent" | "selected" ) NAME { "," NAME }
node        = NAME | "[" NAME { "," NAME } "]"
CMP_OP      = "==" | "!=" | "<" | "<=" | ">" | ">="
KEYWORD     = one of the 26 reserved words of section 3.1: the 18 in use and the 8 reserved for later
NAME        = ( LETTER | "_" ) { LETTER | DIGIT | "_" }
            | "`" { any character but "`" and a line break } "`"
            // "_" alone is not a NAME: it is the case catch-all; section 3.6
LETTER      = "a".."z" | "A".."Z";   DIGIT = "0".."9"
NUMBER      = INT | FLOAT                                              // section 3.2
INT         = DIGIT { DIGIT | "_" DIGIT }
FLOAT       = INT "." INT [ EXP ] | INT EXP
EXP         = ( "e" | "E" ) [ "+" | "-" ] INT
STRING      = '"' { CHAR | ESCAPE } '"'
            // one line; escapes in section 3.2
RAW_STRING  = 'r"' { any character but '"' and a line break } '"'
DOC         = "///" to the end of the line, directly above the fn it describes (section 5.3)
```

**Tokens.** Besides names, numbers, strings and the operators of section 3.3:

| Token | Meaning | Where |
| --- | --- | --- |
| `=` | Binding; definition | `let`; `add`, `update`, `aggregate`, `rename`, `derive`, `frame` |
| `:` | Option; interaction | Arguments; formulas and `Terms` |
| `::` | Qualified name | `functions::summary`, `data::summary` |
| `.` | Method call, record field, namespace | `x.f()`, `r.name`, `dist.normal` |
| `..` | Range; spread | `2..5`, `x1..x9`; `..opts` in a call |
| `=>` | Arm; mapping entry | `case`; `[0 => "male"]` |
| `:=` | Defined parameter | Model blocks |
| `->` | Return type; directed edge | `fn ... -> Float`; `dag { }` |
| `<->`, `--` | Bidirected and undirected edges | `dag { }` |
| `~`, `=~`, `~~` | Formula, measurement, covariance | Formulas; model blocks |
| `` ` `` | Quoted name | Column names that are not valid names (section 3.6) |
| `( )`, `[ ]`, `{ }` | Grouping and calls; lists, indexing, mappings and `[for ...]`; records and blocks | |
| `,` | Separator | Arguments, list items, arms |
| `//`, `///`, `// zeff:` | Comment; help text; pragma | Anywhere; directly before `fn`; above or at the end of a statement (section 11.1) |
| `r"..."` | Raw string | Regular expressions and Windows paths (section 3.2) |

### 3.5 Style

One style, enforced by `zeff fmt`, which has no settings; Zemp runs it on save and on all code it writes.

| Rule | Zeff style | Example |
| --- | --- | --- |
| Names | snake_case for bindings, variables and functions; PascalCase for types; built-in constants in capitals | `avg_sales`, `RegFit`, `PI` |
| Indentation | 4 spaces, never tabs | |
| Line length | At most 100 characters | |
| Chains | A chain that does not fit on one line puts one `.method()` per line, the dot starting the line | see above |
| Arguments | `name: value`, a space after the colon and none before; long calls break after a comma | `se: robust` |
| Spacing | Spaces around binary operators, `~`, `=~`, `~~` and `=>`; none around `^`, `:` and `..`, and none just inside brackets | `y ~ x + exper^2` |
| Model blocks | One formula per line, operators aligned | section 7 |
| Comments | Explain why, not what | `// cluster at firm: treatment assigned by firm` |

Before and after `zeff fmt`:

```
let m=wages.where(year>=2010).reg(lwage~educ+exper,fe:firm,cluster:firm,se:robust)
```

```
let m = wages
    .where(year >= 2010)
    .reg(lwage ~ educ + exper, fe: firm, cluster: firm, se: robust)
```

### 3.6 Names, case and source files

**Case sensitivity.** Zeff is case-sensitive everywhere: keywords, bindings, functions, argument names, types and variables. `Wage` and `wage` are different names, and `TRUE`, `True` and `NULL` are not Zeff words. Because survey and Stata files often differ from the code only in case, a name that matches only when case is ignored is an error with the exact spelling suggested, never a silent match.

```
error[Z0102]: no variable `Wage` in `wages`; names are case-sensitive
 --> scripts/main.zeff:6:12
  |
6 | wages.mean(Wage)
  |            ^^^^ differs only in case from `wage`
  |
  = help: write `wage`
```

**What a name may contain.**

| Rule | Detail |
| --- | --- |
| Characters | A name starts with a letter or `_` and continues with letters, digits and `_`; ASCII only, so code reads and types the same on every keyboard |
| Length | No limit in Zeff; exporting to a Stata file shortens names over 32 characters and reports each change |
| `_` alone | The catch-all in `case`; it cannot be bound |
| Leading `_` | A function in `functions/` whose name starts with `_` is private to its file and not brought in by `use` |
| Reserved words | The 18 words of section 3.1 cannot be names, except as argument and record field names before `:` (section 3.1); section 3.7 sets out the four kinds of word and how a column with a reserved name is written |
| Reserved for later use | `type`, `while`, `match`, `mut`, `pub`, `loop`, `trait` and `impl` cannot be names either (argument and record field names excepted, as for the other reserved words), so a later release can give them meaning (the `type` declaration of section 15 of the Engine and Project Specification, for one) without a new edition |
| Built-in names | A `let` cannot reuse a name from `core`, which is frozen at 1.0 (`let log = ...` is an error). A binding may shadow a name from any other package, with a warning, so adding functions in later releases never breaks an existing script (`let total = ...` warns that it hides `survey::total`). Project functions reuse a built-in name only when written in full, `functions::summary`: a bare call to a name that both a built-in and a project function define is an error (Z0109) naming the full form, never a silent choice. Third-party functions (after 1.0) are always written qualified, `pkg::fn`, unless `use pkg` brings them in, and `use` refuses a name that collides with a built-in or with another imported package (Z0107), so no third-party function ever shadows a built-in (section 8.1 of the Engine and Project Specification, rule 9). A later engine therefore never changes what a bare name calls: when a newer engine adds a built-in with the name of a project function or of an imported name, `zeff engine use` lists every bare call it would capture and, with `--write`, rewrites them to the full form before the pin moves (section 2 of the Engine and Project Specification) |
| Two names differing only in case | Allowed for bindings but warned once by `zeff check`, since the code is hard to read aloud |

**Variable names from files.** Data files bring names Zeff did not choose. `import` keeps them as they are by default (`names: keep`); `names: snake` turns each into a valid snake_case name (`Gross Sales (total)` becomes `gross_sales_total`) and prints the mapping. A kept name that is not a valid Zeff name is written in backticks, `` wages.mean(`gross sales (total)`) ``, and the editor offers to rename it. Two columns whose names differ only in case, or become equal after `names: snake`, are an error at import (Z0605) that lists them, so no column is ever dropped or merged silently. Variable labels from Stata files (and SPSS files, after 1.0) are kept as labels and shown in tables; code always uses the name.

**Naming conventions.** Built-in names follow these patterns, and project functions should too; `zeff check` warns on names that break the case rules, not on the patterns.

| Kind | Convention | Examples |
| --- | --- | --- |
| Bindings, variables, functions, arguments | snake_case | `avg_sales`, `share_missing`, `weight_type:` |
| Types | PascalCase; model results end in `Fit` | `Frame`, `RegFit`, `DidFit` |
| Constants | Capitals | `PI`, `E` |
| Packages and namespaces | One lowercase word; never the name of a shared argument or a frame verb | `regress`, `multilevel`, `identify`; the prefixes `dist.`, `power.` and `spec.` group a family of functions |
| Predicates | Start with `is_`, return Bool | `is_missing`, `is_finite`, `is_in` |
| Counts | Start with `n_` | `n_rows`, `n_missing`, `n_obs` |
| Conversions and reshapes | Start with `to_` | `to`, `to_long`, `to_wide`, `to_timezone` |
| Tests | End in `_test`, return a `TestResult` | `t_test`, `chi2_test`, `egger_test`; long-established names keep their own form (`anova`, `hausman`, `kmo`), and design checks are named for what they check (`pre_trends`) |
| Plots | Plain chart nouns for data plots, `_plot` for derived ones | `histogram`, `scatter`, `qq_plot`, `coef_plot` |
| Models | Short method names, written out where an abbreviation is unclear | `reg`, `ivreg`, `logistic`, `causal_forest`, not `lr` or `cf` |
| Result methods | Nouns for what they return | `coef`, `residuals`, `effects`, `loadings` |
| Bool arguments | Named for the true case, default false where possible | `paired: true`, `ordered: true`, `first: true` |
| Script and function files | snake_case with `.zeff` | `main.zeff`, `clean_wages.zeff` |
| Projects | Lowercase, words joined by `_` or `-` | `schooling`, `cold-storage` |

**Source files and literals.**

| Rule | Detail |
| --- | --- |
| Encoding | Scripts are UTF-8; strings and comments may use any script, so labels in any language work as text |
| Line endings | LF; `zeff fmt` converts CRLF and removes trailing spaces |
| Digit separators | `_` may group digits: `1_000_000`, `0.000_001` |
| Number forms | Decimal and scientific only (`1e-6`); no hexadecimal, octal or `L` suffixes |
| Strings | Double quotes only; single quotes are an error that suggests double |
| Whitespace | Only line breaks matter (section 3.1); indentation never changes meaning |

### 3.7 Core semantics

This section fixes the rules an implementation needs beyond the grammar.

**Kinds of word.**

| Kind | Words | Rule |
| --- | --- | --- |
| Reserved | The 18 of section 3.1, and the 8 reserved for later use | Never a name, except as an argument or record field name before `:` (section 3.1). A column with a reserved name, such as `case` or `model`, is written in backticks: `` wages.mean(`case`) `` |
| Contextual | `latent` and `selected` inside `dag { }`; `efa` inside `model { }`; `_` in `case` | Special only in their context; ordinary names elsewhere |
| Namespaces | `dist`, `power` and `spec` | Only in `dist.normal(...)`, `power.t_test(...)` and `spec.lasso(...)`: the call has the shape of a method call, and the checker recognises the namespaced function from the function registry, so a column named `dist` stays an ordinary name |
| Built-in names | Functions and constants of the packages | `core` names cannot be rebound; others may be shadowed by a `let`, with a warning (section 3.6) |
| Argument names | `by`, `where`, `se`, `fe` and every other parameter name | Never reserved: they have meaning only before a `:` in a call |

**Types in expressions.**

| Rule | Detail |
| --- | --- |
| Numbers | Int with Int gives Int for `+`, `-` and `*`; `/` and `^` always give Float (so `exper^2` is Float); any Float operand gives Float, and an Int is accepted wherever a Float is expected (`level: Float = 0.95` takes `1`; `Column<Int>` where `Column<Float>` is expected), exactly up to 2^53. `div(a, b)` is floored division and `mod(a, b)` its remainder with the sign of `b`, as in Python and R, on Int or Float; `div(a, 0)` and `mod(a, 0)` with Int are errors (Z0212), with Float they follow `/`. `sum`, `min`, `max`, `abs`, `running_sum`, `div` and unary `-` on Int stay Int, and an overflow in any of them is an error naming the row (Z0211); `mean`, `sd`, `sqrt`, `exp`, `log`, `factorial`, `choose`, `gamma`, `round`, `floor`, `ceil` and `truncate` give Float, so `factorial(25)` is finite and `floor(x)` has R's type |
| No implicit conversion | Text never becomes a number or a date, and Bool is not a number: `female + 1` with a Bool `female` is an error that suggests `to(female, Int)`. The reducers `mean`, `sum` and `n` are the exception: on a Bool column they give the share, the count of true values and the count of rows, since the share of a treatment indicator is the commonest summary of all |
| Comparison | Only between values of one type, with Int and Float compared as numbers. A `Category` compares with a `Str` by label, `industry == "Retail"`, and an ordered category also with `<`, `<=`, `>` and `>=` by level order; a label that is not a level is a check-time error listing the levels (Z0216). An Int column with value labels compares as Int: `female == "female"` is an error (Z0216) suggesting `female == 1`. Ordered categories compare with each other by level order; unordered categories support only `==` and `!=`; text compares by Unicode scalar value (section 3.8). A `Date` never compares with text: `sold > "2020-01-01"` is an error (Z0210) that suggests `date("2020-01-01")` |
| Compound values | `==` and `!=` are defined for scalars, dates, categories and `NA`; on lists, records, mappings, frames and results they are errors (Z0208) |
| Identity | Two relations exist, as in Julia. Comparison (`==` and the rest) is three-valued, as above. Identity is what `sort`, `distinct`, `duplicates`, `unique`, `is_in`, `n_distinct`, grouping with `by:`, `tabulate` and `recode` keys use: `NA` is identical to `NA` and `-0.0` to `0.0`, so `is_in(NA, [NA])` is `true`, `unique([NA, NA])` is `[NA]`, and two rows with `NA` keys form one group. Merge keys are the exception and never match on `NA` (section 4.1). Order, for `<` and for `sort`: `false < true`; numbers numerically, with `NA` last in ascending and descending sorts; text by code point; dates chronologically; ordered categories by level order, and unordered categories sort by level order while allowing only `==` and `!=` |
| `Nothing` | `print`, `warn`, `assert` and an `if` without `else` have the type `Nothing`, which cannot be bound or passed (Z0206) and prints nothing; an `if` used as a value must have an `else` |
| Records | A record literal has a structural type, its field names and their types, inferred from the literal; `r.name` on a record without that field is a check-time error (Z0209). `Record` alone in a signature accepts any record, and the fields the function reads are checked at each call against the argument's type, so `iterate`'s state is checked where `iterate` is called |
| Mappings | A `Mapping<K, V>` has one key type and one value type: `[0 => "male"]` is `Mapping<Int, Str>`, and the mappings of `re:` have `Var` keys and `Terms` values. Keys are unique and keep insertion order; `m[k]` for an absent key is an error (Z0204), and `is_in(k, keys(m))` tests first |
| Functions as values | A function type is written `Fn(T1, T2) -> R`. A parameter of that type accepts a named function whose parameter types and return type match exactly, defaults ignored; any other function is an error listing both signatures (Z0205) |
| Indexing | `xs[i]` outside `1..len(xs)` is an error naming the index (Z0207); indices are never negative; `xs[2..4]` is the sub-list from 2 to 4; a frame's `wages["lwage"]` names a column (Z0101 when absent) |
| Conversion | `to(x, T)` follows the table below; a value that cannot convert becomes `NA` with a warning giving the count and three failing values (W0203), never silently |
| Dates | Date minus Date gives Int days; Date plus Int is not allowed: use `add_days`; DateTime likewise with `days_between`. This stays so when `Duration` arrives (section 4.3): `DateTime − DateTime`, an error in 1.0, then gives a Duration, and no existing script changes type |
| Inference | A binding's type is the type of its expression, fixed when the script is checked; a function's result must match its declared return type |
| `case` | Every arm must give the same type, with Int and Float joining to Float; `NA` fits any type |
| Lists | One element type after joining (next row), plus `Failed` (section 4); `[1, 2.5]` is a `List<Float>` and `[m1, iv]` a `List<Result>` |
| Joining types | Several constructs need one type for several expressions: list literals, `case` arms, `if`/`else` branches, `coalesce`, `min` and `max`. The join of two types is: a type with itself gives itself; `Int` with `Float` gives `Float`; any type with `NA` gives that type; two result types (`RegFit`, `IvFit`, `GlmFit`, `TestResult`, …) give `Result`, which offers the result and estimator interfaces (section 8.1 of the Engine and Project Specification, rule 5), while a method of one type, such as `first_stage`, needs an element whose type is known (`fits[2].is(IvFit)`); anything else is an error naming both types (Z0214). `Result` and `Column` may be written in signatures |
| Bare `NA`, `[]`, `[NA]` | Take their type from the parameter, `case`, `if` or list they appear in; where nothing gives one (`let x = NA`, `let xs = []`) it is an error (Z0219) whose help gives the type at the use, `to(NA, Int)` |
| Columns and scalars | A column is a value: `wages["lwage"]` is a `Column<Float>` of the frame's row count, usable with `len`, `frame(x = ...)` and every column function. Beside a column, a scalar stands for its value in every row. Two columns combine only when they belong to the same frame or have the same length; otherwise it is an error (Z0217) suggesting `merge`. Every built-in scalar function of `core`, `text` and `time` applies row by row when given a column and returns a column. A project function is not lifted: its parameter types are taken literally, so `fn double(x: Float)` refuses a column (Z0218, "declare `x: Column<Float>` or write the expression inside `add`"). In `case`, arms may mix scalars and columns, and the result is a column when any condition or value is one |
| Text | No operator joins or repeats text: `"a" + "b"` is an error (Z0215) suggesting `join` or `format`. `==`, `!=`, `<`, `<=`, `>` and `>=` compare code points and are case-sensitive (section 3.8) |
| Words and types | A bare word such as `robust` exists only as an argument whose parameter lists its allowed values; elsewhere it is an unknown name (Z0101), and a `Str` is not accepted in its place (Z0403, listing the words), so Zemp writes the word, never a string. A type name (`Str`, `Float`, `Failed`, `Date("%d/%m/%Y")`) is a value only for `to`, `is`, `type_of` comparisons and `types:`; `Date(format)` and `DateTime(format)` are the only type constructors; type names are `core` names and cannot be rebound |

**Missing values: full truth table.** With T for true, F for false and N for `NA` (both operators are symmetric, so swapping `a` and `b` gives the same result):

| `a` | `b` | `a && b` | `a \|\| b` | `!a` |
| --- | --- | --- | --- | --- |
| T | T | T | T | F |
| T | F | F | T | F |
| T | N | N | T | F |
| F | F | F | F | T |
| F | N | F | N | T |
| N | N | N | N | N |

**Conversions.** `to(x, T)` and the implicit Int-to-Float rule:

| From → to | Rule |
| --- | --- |
| Int → Float | Implicit everywhere: arithmetic, comparisons, arguments, list and `case` joins, `Column<Int>` where `Column<Float>` is expected; exact up to 2^53 |
| Float → Int | Only by `to`; a whole-number value converts, any other (2.5, `Inf`) becomes `NA` with warning W0202 whose help names `round`, `floor` or `truncate` |
| Str → Int, Float | Leading and trailing spaces ignored; the text must be a whole decimal literal (`"42"`, `"-7"`) or a Float literal (`"4.2e1"`); anything else, `"1_000"` and `"1,000"` included, is `NA` with a warning; grouped digits need `thousands:` at import |
| Str → Bool | `true` or `false` in any case; everything else `NA` with a warning |
| Int → Bool | 0 and 1 only; other values `NA` with a warning. Bool → Int or Float gives 0 and 1 |
| Anything → Str | The printed form of section 3.8 for scalars, with a Float as the shortest text that reads back to the same number |
| Str → Category | The distinct values in code-point order as levels, unless `levels:` is given (`category()`); Category → Str gives the label and Category → Int the level position, 1-based |
| Str → Date, DateTime | ISO 8601 only, or the format given; otherwise `NA` with a warning (section 4.1); an impossible date such as `2026-02-30` likewise |
| Date → DateTime | Midnight; DateTime → Date drops the time |

Every other operator and function gives `NA` when an input is `NA`, unless its entry in zeff_functions.csv says otherwise (reducers skip `NA` with a warning, as section 4.1 sets out).

**Evaluation order and scope.**

| Rule | Detail |
| --- | --- |
| Eager | In 1.0 every statement and chain step runs when reached, in order (lazy chains from 1.1 give the same results) |
| Definitions in one call | `add`, `update` and `aggregate` evaluate their definitions left to right, and later ones may use earlier ones: `.add(lwage = log(wage), dev = lwage - mean(lwage))` |
| Arguments | The receiver first, then the arguments left to right as written, positional then named, with a spread `..opts` evaluated where it stands (an argument written out overrides it); then the defaults of omitted parameters in parameter order. Random draws within one statement are numbered in this order (section 8.3 of the Engine and Project Specification) |
| Defaults | A parameter's default is evaluated at each call, after the parameters before it, which it may name; a `Var` default names a column of the frame argument and is checked against that frame at the call |
| Short-circuit | `&&` and `\|\|` on single values skip the right side when the left decides the result; on columns both sides are computed row by row |
| Block scope | A `let` inside `{ }` (an `if`, `case` arm, `for` body or function) is visible only to the end of that block; a loop variable is visible only inside its loop, a `let` in the body is fresh on every pass, and a `let` that shadows a loop variable or a parameter warns (W0302) |
| Narrowing | After `if x.is(Failed) { ... }` whose block ends in `continue`, `break`, `return` or `error`, `x` has its non-failed type for the rest of the enclosing block, and inside `if !x.is(Failed) { ... }` likewise; no other expression narrows a type |
| Name lookup | In a frame method, in this order: (1) a name in call position is a function; (2) a loop variable or function parameter in scope, and if the frame also has a column of that name it is an error (Z0106) naming both, since one of them should be renamed; (3) a column; (4) a script binding, with a warning when a column of the same name exists. Outside a frame method: a loop variable or parameter, then a script binding, then a function. The parameter's type decides which of these applies, as the next table sets out |
| Functions | See only their parameters and other functions (section 5.3); `fn` and `use` appear only at the top level of a file (Z0309), and `return` outside a function or `break` and `continue` outside a loop are errors (Z0313) |
| Recursion | A function may not call itself directly or through other functions (Z0108, with the cycle shown); iteration is `for` and `iterate`, so a script always finishes and the engine needs no call-depth limit beyond the nesting limit of section 3.8 |
| Effects | A function may call `print`, `warn`, `error` and `assert`; `import`, `save`, `export`, `seed`, `today`, `now` and `timer` are allowed only in scripts (Z0110), so a function's result depends on its arguments alone and may be cached |

**Resolution by parameter type.** A bare name in an argument means what the parameter's type says:

| Parameter type | A bare name means | Looked up in |
| --- | --- | --- |
| `Var`, `Vars`, `Terms`, `Formula` | A column, or a loop variable or parameter holding a `Var` | The receiver frame; in `merge`, `on:` names must exist in both frames with the same type (rename one first when the names differ); a `Var` passed to a project function is checked against the frame the function uses it with |
| A fixed list of words | A word | The parameter's list (Z0402 when absent) |
| A term name (`coef`, `test`, `derive` and the other result methods) | A term of section 7.2 | The result's model matrix |
| Any other type | A loop variable or parameter, then a script binding, then a function | The enclosing scopes, in that order |
| Call position, `f(...)` or `.f(...)` | A function, always | Built-in functions, then the project's functions brought in by `use`; a bare name that both define is an error (Z0109) naming the full form (section 3.6) |

**Variable ranges.** `x1..x9` selects every column from `x1` to `x9` by position in the frame, both ends included, whatever their names. Both ends must exist and `x9` must not come before `x1`; otherwise the range is an error that shows the frame's order. `vars()` lists outside a frame method keep ranges unexpanded until they meet a frame.

**Failed values.** A `Failed` value can sit in a list (section 5.4). Using one anywhere else, as an argument, in arithmetic, in a frame or as a model input, is an error that reports the original failure's code and line. `report`, `len` and other collecting functions skip `Failed` values and count them; `x.is(Failed)` tests for one. A method called on a binding that may hold a `Failed` value, such as `let m = attempt(...)` followed by `m.coef(x)`, is a check-time warning unless the call is guarded by `m.is(Failed)`.

**Stability.** Error and warning codes, term names (section 7.2) and printed result layouts are stable within an edition; changing them needs a new edition.

### 3.8 Text, time, files and limits

The rules of this section settle what language specifications usually settle and statistics packages usually leave to the platform; every one of them is fixed so that a script gives the same result on every machine.

**Text.**

| Rule | Detail |
| --- | --- |
| Unit | Text functions count Unicode scalar values (code points), never bytes or grapheme clusters: `len("café")` is 4 and `len("e\u{301}")` is 2, since a combining accent is its own scalar value; `slice`, `pad`, `extract` and `format` use the same unit. Grapheme-aware functions are an open decision (section 15 of the Engine and Project Specification) |
| Comparison and order | Text compares and sorts by code point, never by locale, so order is identical everywhere; `sort` on a text column follows the same rule. Locale-aware collation is after 1.0 |
| Case | `upper` and `lower` apply Unicode simple case mapping, one scalar value to one, so a string's length never changes (`"ß"` stays `"ß"`); `ignore_case:` in `contains` uses simple case folding; `trim` removes Unicode white space |
| Normalisation | None is applied: two strings that differ only in normalisation form are different values, and `import` notes a text column whose values differ only in that way (W0601) |
| Regular expressions | `regex: true` in `contains` and `replace`, and the patterns of `extract` and `split`, follow the syntax of Rust's `regex` crate: Unicode classes, no back-references or look-around, matching in linear time, identical on every platform |
| Encoding | Scripts and data files are UTF-8; a data file in another encoding is an error (Z0608) that suggests `encoding:` on `import`; exports are UTF-8 without a byte-order mark unless `save` is given `bom: true` for Excel |

**Time.**

| Rule | Detail |
| --- | --- |
| Calendar | Proleptic Gregorian, years 1 to 9999; no leap seconds |
| `Date` | A calendar day without a time zone |
| `DateTime` | A calendar date and wall-clock time to the microsecond, with no time zone, in 1.0; ISO 8601 text with an offset is converted to UTC and noted (W0702); `DateTime − DateTime` is not allowed, `days_between` is; time zones and `to_timezone` come after 1.0 |
| Month arithmetic | `add_months` clamps to the last day of the target month: 31 January plus one month is 28 or 29 February, as Stata and SQL do (R's `seq` rolls over into March); `months_between` counts whole months with the same rule |
| Differences | `Date − Date` is Int days; `days_between` on DateTimes counts whole days between the two instants, truncating towards zero |
| `today()` and `now()` | The only functions whose value depends on when a script runs: `now()` gives the machine's local wall-clock time and `today()` its date. Each use is noted (W0703) and its value is recorded in the run record at first use; `zeff run --replay` reuses the recorded values, `--deny-warnings` refuses them, and a statement that depends on either is never served from the result cache (section 8.3 of the Engine and Project Specification), so a replication package never depends on the clock |

**Files.**

| Rule | Detail |
| --- | --- |
| CSV reading | RFC 4180: `,` delimiter unless `delim:` is given, with `;` or tab detected from the first line and noted; `"` quotes with doubled quotes inside; line breaks allowed inside quotes; CRLF and LF accepted; a last line without a line break accepted; a row with the wrong number of fields is an error naming the row (Z0607). The first row is the header unless `header: false`, in which case columns are `v1`, `v2`, …; a repeated or empty header name is an error (Z0605), and `names: snake` fixes awkward headers (section 3.6). `true` and `false` in any case make a Bool column; `0` and `1` stay Int. The file must be UTF-8, with or without a byte-order mark, unless `encoding:` names another encoding from the WHATWG list (`"windows-1252"`); an undecodable byte is an error naming the row (Z0608) |
| CSV writing | UTF-8 without a byte-order mark unless `bom: true`; LF line endings; fields quoted only when needed; `NA` as an empty cell; `Inf` as `Inf`; Floats as the shortest text that reads back to the same number; dates in ISO 8601; categories as labels unless `codes: true`; variable labels and the panel declaration are not written, so Parquet is the format that keeps them |
| Paths | Written with `/` on every system; `\` is neither a separator nor a valid escape (section 3.2); paths are checked for case (Z0702) and confined to the project and its data roots (Z0701) |
| Exports | CSV writes an empty cell for `NA` and ISO 8601 dates; Stata files write `.`; Parquet and Arrow keep labels, categories, contrasts and panel declarations in schema metadata under the Zeff Parquet profile, and a Parquet file `save` writes is the same bytes on every platform (section 8.7 of the Engine and Project Specification) |

| Zeff type | Arrow type | Note |
| --- | --- | --- |
| `Bool` | Boolean | The validity bitmap carries `NA` for every type |
| `Int` | Int64 | |
| `Float` | Float64 | `NaN` is never stored (section 4.1) |
| `Str` | LargeUtf8 | 64-bit offsets, so one column may exceed 2 GB |
| `Date` | Date32 | Days since 1970-01-01 |
| `DateTime` | Timestamp(microsecond, UTC) | |
| `Category` | Dictionary(Int32 → Utf8) | Levels in order; the ordered flag and the contrast in field metadata |

**Limits.**

| Limit | Value |
| --- | --- |
| Rows | Counted in Int64; a frame may exceed 2^31 rows natively, and the browser is bounded only by the memory budget (section 8.3 of the Engine and Project Specification, point 7) |
| Columns | At most 2^31 − 1 |
| One text value | At most 2 GB |
| Expression nesting | 256 levels, and 1,024 steps in one chain; deeper is a parse error (Z0307), so the parser's stack is bounded in WebAssembly |
| Script size | 16 MB per file |
| Category levels | A category with more than 10,000 levels is a warning (W0602), since an identifier column was probably categorised by mistake; the dictionary allows 2^31 − 1 |
| Names | No length limit (section 3.6) |

**Diagnostic positions.** A position is a 1-based line and a 1-based column counted in Unicode scalar values; the language server converts columns to the UTF-16 units the protocol uses, and byte offsets never appear in a message.

**Printing.** A statement that is not a `let` prints its value through the display interface (section 8.1 of the Engine and Project Specification, rule 5):

| Value | Printed as |
| --- | --- |
| `Int`, `Float` | An Int in full; a Float to six significant digits with trailing zeros dropped, in exponent form below 1e-4 and from 1e15 (`0.975002`, `4.90671e-198`), as R and Stata print; `to(x, Str)` and CSV writing keep the shortest text that reads back to the same value; `-0.0` prints as `0`; `NA`, `Inf` and `-Inf` as written; no locale, no thousands separator unless `format(x, big_mark:)` asks |
| `Str`, `Bool`, `Date`, `DateTime` | The value itself; dates in ISO 8601 |
| Frame | Its `describe` header and first ten rows (section 3.1) |
| `Column<T>` | Its type and length on one line (`Column<Float> · 5 values`), then its values as a list prints them: one per line, up to 20, then the count of the rest; the values of an opaque column print as the opaque row below says |
| List, Record, Mapping | One line per item or field, up to 20, then the count of the rest |
| Result | Its `summary`; tables round half to even: estimates, standard errors and confidence limits to 4 decimals, test statistics to 2, descriptive statistics to 3, p-values to 3 with `<0.001` below, and counts with thousands separators, as section 10.2 shows; a `report` table, which sets models side by side, rounds estimates and standard errors to 3 decimals; `report(digits:)` changes a table's precision |
| Table, Plot | Shown in the viewer or the console; `.export()` writes them |
| `Fn` | Its signature |
| `Dag` | Its text form, one edge per line as a `dag { }` block writes it; `g.plot()` draws it |
| Opaque column value | A summary in angle brackets: `<list 3>`, `<record 4>`, `<bytes 2.1 kB>`, `<file scans/0142.jpg>`, `<polygon>` (section 4.3) |
| `Nothing` | Nothing (section 3.7) |

**Exit codes.** `zeff run`, `zeff check` and `zeff test` exit with 0 on success; 1 on a script error (Z01xx to Z08xx) or a failed `assert`; 2 on an internal error (Z9xxx); 3 when `--deny-warnings` turns a warning into an error; 4 when the engine, edition or `zeff.lock` does not match the project, including a pinned engine that is not installed and cannot be installed (Z0710, section 2.1 of the Engine and Project Specification).

---

---

## 4. Types and values

Every value has one of the types below. Types are inferred everywhere and written only in function signatures; `type_of(x)` names a value's type, `x.is(Type)` tests it and `x.to(Type)` converts it, turning failures into NA with a warning.

| Family | Types | Example | Notes |
| --- | --- | --- | --- |
| Scalar | `Bool`, `Int`, `Float`, `Str` | `true`, `42`, `0.05`, `"text"` | Int and Float are 64-bit; Int overflow is an error; Str is UTF-8 |
| Time | `Date`, `DateTime` | `date("2026-10-15")` | Durations and time zones after 1.0 |
| Column | `Column<T>` of Bool, Int, Float, Str, Date, DateTime or `Category`; opaque column types carried but not computed on (section 4.3) | a variable in a frame | One type per column, plus NA; text never becomes a Category silently |
| Collection | `List<T>`, `Record`, `Mapping<K, V>`, `Range` | `[m1, m2]`, `{ se: robust }`, `[0 => "male"]`, `2..5` | A list holds one type, and any list may also hold `Failed` values (section 5.4); a record has the fields of its literal (a structural type); a mapping has one key and one value type and keeps insertion order (section 3.7) |
| Variables | `Var`, `Vars`, `Term` | `firm`; `[age, x1..x50]` in a frame method, `vars(age, x1..x50)` outside one; `educ` in `m.coef(educ)` | Variable names, checked against the frame where they are used; a `Term` names a column of a model matrix (section 7.2) and is checked against the result |
| Data | `Frame`, `SurveyDesign` (1.9) | `import("data/wages.csv")`, `data.survey(...)` | A dataset; `panel()` adds unit and time to it; a survey design adds weights, strata and clusters |
| Specification | `Formula`, `Terms`, `Model`, `Dag`, `Distribution` | `lwage ~ educ`, `firm + year`, `model { }`, `dag { }`, `dist.t(df: 24)` | Values like any other: `let f = lwage ~ educ + exper`. `Terms` is the right side of a formula without `~`, as `fe:` takes it. A distribution parameter outside its domain (`sd:` not positive, `df:` not positive) or a probability outside [0, 1] given to `quantile` is an error (Z0413) |
| Result | `TestResult`, `RegFit`, `IvFit`, `GlmFit` in 1.0; `DidFit`, `SemFit`, `MixedFit`, `LassoFit`, `ForestFit`, `LcaFit`, `LcaFits` and more as packages arrive | `wages.reg(f)` | `Result` is the interface every one of them implements: the result and estimator interfaces of `results` (section 8.1 of the Engine and Project Specification, rule 5) |
| Failure | `Failed` | `attempt(expr)` when `expr` errors | Holds the error code, message and line; may stand in any `List<T>`, so `[for ...]` with `attempt` gives `List<RegFit>` with some entries failed; `report` and `len` skip it with a count (section 5.4) |
| Output | `Table`, `Plot` | `report([m1, m2])`, `fit.plot()` | Matrix results (correlation, covariance, vcov) are Tables; exported with `.export(path)` |
| Function | `Fn(T1, …) -> R` | a named `fn` | Functions can be passed as arguments to a parameter of matching type (section 3.7) |
| Word | a fixed list in a signature, `se: classical \| robust` | `robust`, `many_to_one`, `holm` | A bare name where a parameter lists its allowed values; it exists only as such an argument (section 3.7) |
| Type | `Type` | `Str`, `Float`, `Failed`, `Date("%d/%m/%Y")` | A type name used as a value, accepted only by `to`, `is`, `type_of` comparisons and `types:` (section 3.7) |
| Nothing | `Nothing` | `warn("...")`, `if c { ... }` | The type of a call that returns no value; cannot be bound or passed, prints nothing (section 3.7) |

### 4.1 Missing values, NaN and infinity

Zeff has one missing value, `NA`. Invalid results such as `0/0` become `NA` with a warning naming the cause, so a column holds `NA` and `Inf` but never `NaN`. Kernels set the validity bit as they compute, rather than scanning their output afterwards, and report the count once.

| Situation | Zeff behaviour | Compared with R and Stata |
| --- | --- | --- |
| Arithmetic | `NA + 1` is `NA` | As R |
| Scalars | The same rules as columns: `Inf - Inf`, `Inf * 0`, `0/0`, `sqrt(-1)` and `log(-1)` give `NA` with a warning; `x/0` gives `Inf` or `-Inf` with a warning; `Inf` is a Float and turns an Int expression into a Float one; `-0.0` is kept as IEEE requires, equals `0.0`, groups with it (section 3.7, Identity) and prints as `0` | R keeps `NaN` |
| `0/0`, `sqrt(-1)`, `log(-1)` | `NA`, with a warning giving the cause and the number of rows | R keeps `NaN` |
| `log(0)`, `1/0` | `-Inf`, `Inf`, with a warning and a suggestion (`log1p`, or filter first) | R is silent; Stata turns them into missing silently |
| Comparison | `income > 100` is `NA` where income is `NA`; `NA == NA` is `NA` | Stata treats missing as larger than any number |
| Filtering | `where(cond)` drops rows where `cond` is `NA` and says how many; `where(age > 30)` and `where(!(age > 30))` therefore both drop rows with `NA` age, and the note of each says so, so the two never silently partition a frame | Stata's `keep if x > 5` keeps missing x |
| Statistics | `NA` skipped with a warning giving the count; any `Inf` gives `Inf`; `Inf` and `-Inf` together give `NA` with a warning; a sum of all-NA input is `NA` | R needs `na.rm = TRUE`; R and Stata give 0 for an all-missing sum |
| Weights | A row with `NA` weight is dropped and counted; a negative weight is an error listing the rows; a zero weight leaves the row out of the estimate and the observation count, with a note giving how many (as R's `lm` leaves such rows out of the degrees of freedom) |
| Models | Rows with `NA` in model variables dropped and counted; `Inf` in a model variable is an error listing the rows; `missing: fiml` in SEM | Stata drops silently |
| Groups | `NA` forms its own group, shown as `NA` | pandas drops it |
| Merge keys | `NA` never matches; such rows are reported as unmatched | dplyr matches NA keys |
| Sorting | `NA` last; `Inf` and `-Inf` sort as numbers | |
| Testing | `is_missing(x)`, `is_finite(x)`, `n_missing(x)`, `fill_missing(x, 0)`, `coalesce(a, b)` | `x == NA` is always `NA`, so it is flagged as a mistake |
| Import | `types: {col: Str}` overrides a guessed column type; empty cells, `NA` and `.` are missing; `Inf` and `-Inf` are infinity; `NaN` becomes `NA` with a note; `na_values: [-99, -98]` adds codes. Dates are read only when the form is unambiguous (ISO 8601, `2026-10-16`) or when `types:` names the format, `types: {sold: Date("%d/%m/%Y")}`; a column that looks like dates in any other form (`04/05/2026`) is read as text, with a note that names the column, shows three values and proposes the `types:` entry. A format is never guessed. A UTF-8 byte-order mark at the start of a file is removed with a note; `thousands: ","` reads grouped digits such as `1,234,567`; the CSV dialect, other encodings and Bool spellings are in section 3.8 |
| Export | Empty cell in CSV, `.` in Stata files; the full CSV writing rules are in section 3.8 | |

`Inf` stays a value because it has honest uses, such as open ends in `bin(x, breaks: [-Inf, 0, 10, Inf])`. After 1.0, `na_values: [-99 => "refused", -98 => "dont_know"]` will keep the reason while the value still counts as `NA`.

### 4.2 Categories

```
let wages = wages
    .add(industry = category(industry),
         edu = category(edu, levels: ["school", "college", "postgrad"], ordered: true))
    .label(female, [0 => "male", 1 => "female"])
    .recode(industry, ["Mining" => "Other", "Utilities" => "Other"])

let m = wages.reg(lwage ~ educ + base(industry, "Retail") + female, se: robust)
```

| Task | Zeff | R / Stata |
| --- | --- | --- |
| Create | `category(var, levels:, ordered:)` | `factor()`; `encode` |
| Use in a formula | Expands to indicator variables; the first level is the base | Stata needs `i.var` |
| Change the base | `base(var, "Retail")` inside the formula | `relevel()`; `ib3.var` |
| Value labels | `label(var, [code => "text"])` keeps codes and shows labels | `label define`, `label values` |
| Merge levels | `recode(var, [old => new])` | `forcats::fct_collapse()`; `recode` |
| Inspect | `levels(var)` | `levels()` |
| Contrasts | `category(x, contrast: treatment \| sum \| helmert \| poly = treatment)` sets how the indicators are coded; `sum` gives the effects type-3 ANOVA needs, `poly` orthogonal polynomial trends for ordered levels | R `contrasts()`, `contr.sum`; Stata `anova` and `contrast` |
| Ordinal items | `ordered: true` makes CFA and SEM treat the item as ordinal once WLSMV arrives (1.6). Before then, an ordered item in `cfa` or `sem` is an error whose help gives the usual interim choice: fit with `estimator: mlr` and `ordered: false`, which treats five- or seven-point items as continuous with robust standard errors | lavaan `ordered =` |


### 4.3 Column types, metadata and the data to come

Zeff 1.0 computes on seven column types. Spatial, nested, binary, file-backed and network data arrive later, and four rules fixed now let them arrive without a change to the language.

**1. A column type is declared through one interface.** In the kernel, every column type declares its Arrow storage; its Arrow extension name and version, written into the field metadata so that Parquet and Arrow files carry the type; how a value prints and its type-row abbreviation; its NA rule; the relations it supports (comparison, identity or neither, section 3.7); and its conversions under `to`. The seven types of 1.0 are the first declarations. New column types are added by the engine with a minor release, never by a script or a third-party package (the sandbox of section 8.3 of the Engine and Project Specification, point 6 has no place for a type's printing or sorting), so the set of types an engine knows is closed, and every engine prints and sorts a type the same way.

**2. Computable and opaque.** A column type is either *computable*, accepted by every verb, reducer, model and plot where its type fits (the seven types of 1.0), or *opaque*: the engine stores it, hashes it, prints it as a summary, keeps it through the row verbs (`where`, `sort`, `sample`, `select`, `drop`, `rename`, `merge`, `append`, `head`, `tail`, and `distinct` by identity), round-trips it through Parquet and Arrow with its extension metadata, and refuses it everywhere else with an ordinary check-time type error (a `Geometry` column where a Float is expected is a Z04xx error like any other). `save` to CSV, Excel, Stata or SPSS cannot hold an opaque column and is an error naming it and suggesting `drop`. An opaque type becomes computable when its package arrives, function by function, and nothing an earlier script did changes.

| Opaque type in 1.0 | Storage | Arrives from | Prints as | Computable from |
| --- | --- | --- | --- | --- |
| `Column<List<T>>` | Arrow list, or a fixed-size list for vectors of one length such as embeddings | Parquet and Arrow, a JSON array inside a record, an R list column | `<list 3>`; type row `<lst>` | 1.1: `unnest` (one row per element) |
| `Column<Record>` | Arrow struct | Parquet and Arrow, a nested JSON object, an API response | `<record 4>`; `<rec>` | 1.1: `flatten` (dotted columns), `r.field` in column expressions |
| `Bytes` | Arrow binary | Parquet and Arrow | `<bytes 2.1 kB>`; `<bin>` | With the first package that needs it |
| `FileRef` | A path relative to the project or a data root, with the file's content hash | Readers that list a folder of files (1.x) | `<file scans/0142.jpg>`; `<file>` | With the first package that reads media; `external()` from 1.1 |
| `Geometry` | GeoArrow (WKB) with `crs` in the metadata | GeoParquet and GeoJSON geometry, later shapefile and GeoPackage | `<polygon>`, `<point>`; `<geom>` | The `spatial` package (section 14.1 of the Engine and Project Specification) |
| `Duration` | Arrow duration | Time arithmetic after 1.0 | `3d 04:00`; `<dur>` | With time zones (section 15 of the Engine and Project Specification); `DateTime − DateTime`, an error in 1.0, then gives a Duration, while `Date − Date` stays Int days (section 3.8) |
| `Period` | Int64 with the frequency in the metadata: year, quarter, month, week or day, and the first month of a fiscal year | Stata `%ty`, `%th`, `%tq`, `%tm` and `%tw` formats, Excel, later `to_long(names_type:)` | `2019Q4`, `2019-03`, `2019w05`, `FY2019-20`; `<per>` | 1.x: `panel(time:)` takes its frequency for gaps and lags, and the `series` package |

The names `Geometry`, `Bytes`, `FileRef`, `Duration`, `Period`, `Matrix`, `Vector` and `Network` (the last three are value types, not column types) are `core` names from 0.2.0, so no script or package can take them (section 3.6); `type_of` returns them and `is` tests them. A `FileRef` names a file without reading it: its content hash is computed when the frame is first read, cached in `.zeff/schemas/` under the path, size and modification time, and recorded in the run record like any input (section 2 of the Engine and Project Specification); the content is read only by a function that asks for it, so a frame may hold ten thousand scans or recordings within the memory budget.

**3. Metadata on columns and frames.** A label is one key of a key–value store that every column carries; a frame has one too, for its panel declaration and its source. Readers fill the store, `label` sets `label` and `value_labels`, a verb keeps a column's metadata whenever it keeps the column, `rename` keeps all of it, a derived column starts empty (so `log(wage)` carries no unit, which is right), `aggregate` keeps the metadata of its `by:` columns only, and Parquet and Arrow carry the whole store in the field metadata. The keys are fixed now so that every reader fills the same ones; Zemp's variable view shows them all.

| Key | Meaning | Filled by |
| --- | --- | --- |
| `label` | Variable label | Stata, SPSS, SAS, R `labelled` columns; `label` |
| `value_labels` | Codes to text | Stata, SPSS; `label(var, [code => "text"])` |
| `missing_codes` | Values read as NA, kept for round trips | Stata extended missing values, SPSS user-missing values, `na_values:` |
| `measure` | `nominal`, `ordinal` or `scale` | SPSS; `category(ordered: true)` sets ordinal |
| `unit` | Unit of measurement, as text (`"km"`, `"kg"`, `"tonnes"`) | Readers that carry units; a `unit:` option of `label` in 1.x; read by spatial cutoffs and survey quantities |
| `question` | Survey question text | Survey-platform readers (1.x) |
| `source` | The file, sheet, table or URL a column came from | Every reader |
| `format` | Display format carried from Stata or SPSS, for round trips | Stata, SPSS |
| `crs` | Coordinate reference system of a `Geometry` column | Spatial readers |

**4. What stays outside the engine.** Network data is an edge list, a frame, until the `Network` type arrives with its package. Gridded data (NetCDF, HDF5, rasters) is reduced to a frame by its reader, as zonal or district statistics, rather than given an array type. Images, audio and video never enter the engine: a `FileRef` column names them, and an `external()` step or a later package turns them into columns. Units are metadata, not a numeric type: a cutoff is a number with a `unit:` option, never a literal such as `100km`.

**5. Values reserved for later, with their meaning fixed.** `Matrix` and `Vector` are dense Float values for the user estimators of 1.8 (`mle`, `gmm`, `minimize`), for structural VAR identification, for restriction matrices beyond what `test` writes as Bool expressions, and for the document–term matrices of text analysis; sparse storage is an implementation detail. Their semantics are fixed now so that no operator changes meaning: `*` between two matrices is the matrix product, as in Stata, MATLAB and R's `%*%`, and `^` is not defined for them; elementwise operations are methods; `.transpose()` transposes, and there is no `'` token; `matrix`, `vector` and `transpose` are `core` names reserved before the 1.0 freeze and callable from 1.8, so `core` need not change when they arrive; `m[i, j]` indexes, a syntax error in 1.0 and so reserved; `matrix([[1, 0], [0, 1]])` and `vector([1, 2, 3])` build them from nested lists, and a `Table` of matrix results converts with `to(Matrix)`. Closures are written `|x| expr`, as Rust writes them, a syntax error in 1.0 and so reserved (section 14.1 of the Engine and Project Specification). Censored outcomes need no grammar change: the left side of `~` is already any expression (section 3.4), so a survival analysis writes `surv(time, event) ~ x` with a term function and a value type that arrive with the package (section 14.1 of the Engine and Project Specification). `Complex` numbers stay internal to the `series` package, and there is no `Set`, `Interval`, decimal or array type.

---

---

## 5. Control flow, functions and errors

Zeff has two conditionals, one loop and typed functions, all expressions in the Rust manner. `if` decides what the script does; `case` decides values row by row. Using `if` on a column is an error, which removes R's `if` versus `ifelse` trap.

### 5.1 if and case

```
if wages.n_rows() < 30 {
    warn("fewer than 30 rows: treat large-sample tests with care")
}
let reps = if wages.n_rows() < 500 { 5000 } else { 1000 }

let wages = wages.add(age_group = case {
    age < 25 => "young",
    age < 55 => "prime",
    _        => "older",
})
```

- `if` takes one `Bool`; a column or an `NA` condition is an error that names the problem.
- `case` checks arms in order and the first true one wins; `_` catches the rest. An arm whose condition is `NA` does not match. Without `_`, unmatched rows get `NA` and a warning gives the count.
- `case` is a chain of conditions, as SQL's `CASE WHEN`, not pattern matching: an arm's left side is any `Bool` or `Column<Bool>` expression, there is no value to match against and no destructuring. `_` may appear once and only as the last arm (Z0316); an arm that can never be reached because an earlier condition is always true is a warning (W0301).

### 5.2 for

`for` runs over a `List`, a `Range` or `Vars`. In brackets it collects one result per pass into a list.

```
let fits = [for y in vars(lwage, hours, union) {
    wages.reg(y ~ educ + exper, fe: firm, cluster: firm)
}]
report(fits).export("output/table2.docx")
```

| Rule | Detail |
| --- | --- |
| Loop variable in a formula | Stands for the variable it holds: `y ~ educ` becomes `lwage ~ educ`, then `hours ~ educ` |
| `break`, `continue` | Supported. In `[for ...]`, `continue` adds nothing for that pass, so the list may be shorter than the range, and `break` ends the list with the passes collected so far; outside a loop both are errors (Z0313) |
| Random methods | Each pass gets its own seed, derived from the script seed and recorded, so re-runs match |
| Parallel | Passes cannot change shared bindings, so the engine may run them in parallel (from 1.1); warnings are still printed in pass order, the error reported for a loop is that of the lowest-numbered failing pass, and cancellation happens between passes |
| Failures | A pass that errors stops the script, unless its body is wrapped in `attempt` (section 5.4) |
| `while` | Not in v1: every loop has a known length, so a script always finishes |
| Carrying state | Because bindings never change, one pass of a `for` loop cannot pass a value to the next. Iteration to convergence uses `iterate` (section 5.5) |

### 5.3 Functions

`fn` defines a function with typed parameters, optional defaults and a return type. The last expression is the result, `return` exits early, and any function can be called as a method on its first argument.

```
/// Share of rows where a variable is missing.
fn share_missing(data: Frame, var: Var) -> Float {
    data.where(is_missing(var)).n_rows() / data.n_rows()
}

/// Three standard robustness checks for one specification.
fn robustness(data: Frame, f: Formula, cluster: Var = firm) -> List<RegFit> {
    [
        data.reg(f, se: robust),
        data.reg(f, cluster: cluster),
        data.where(year >= 2015).reg(f, se: robust),
    ]
}

wages.share_missing(educ)
report(wages.robustness(lwage ~ educ + exper)).export("output/table3.docx")
```

| Rule | Detail |
| --- | --- |
| Types | Required on parameters and the result |
| Defaults | `cluster: Var = firm` makes a parameter optional; the default is evaluated at each call and a `Var` default names a column of the frame passed (section 3.7) |
| Optional without a default | `group: Var?` may be left out; inside the function it is `NA` when absent, so one function can offer several forms, as `t_test` does |
| Fixed lists | `effect: d \| g = g` accepts only the listed words |
| Pure | A function cannot change anything outside itself; it returns new values |
| Scope | A function body sees its parameters and other functions (built-in and the project's), never the script's bindings; anything else it needs is passed as an argument, so a function gives the same result wherever it is called |
| Values | A named function can be passed to a parameter typed `Fn(...) -> R` (`minimize(f, start)`, `iterate(step: em_step, ...)`); closures (`\|x\| ...`) come after 1.0 |
| Recursion | Not allowed: a function may not call itself directly or through other functions (Z0108); iteration is `for` and `iterate` |
| Placement | `fn` appears only at the top level of a file (Z0309): in `functions/` for reuse, or in a script, where it is private to that script |
| Order | Definitions are order-independent: a function may call one defined later in its file or in another module; a module file holds only `///` text, `fn` definitions and `use` lines, and `use` lines come before the first statement of a script |
| Matching | Positional arguments fill parameters in order, optional ones included, and may not follow a named one; argument names match exactly, with no partial matching (`se: rob` is an error suggesting `robust`) |
| Effects | `print`, `warn`, `error` and `assert` may be called from a function; `import`, `save`, `export`, `seed`, `today`, `now` and `timer` only from a script (Z0110) |
| Help | `///` lines directly above a `fn` (no blank or `//` line between) become its help, hover text and the text of its Zemp dialog; a `///` line anywhere else is an error (Z0318). The first line is the summary shown in completion; `parameter: text` lines document the parameters, `returns:` the value, and a line `/// > statement` is an example whose expected printed output follows, which `zeff test` checks. A module function must have the complete page of section 9.1 of the Engine and Project Specification (Z0319); a script function needs the summary line (W0303 without it) |
| Zemp dialog | Built from the signature: `Frame` a selector, `Var` a drop zone, `Float` a number field showing the default |
| Where they live | Modules: one file each in the project's `functions/` folder, brought in with `use functions::robustness` (every public function of `functions/robustness.zeff`) or `use functions::robustness::checks` (one function) |

**Modules.** A module is one file in `functions/`. `use functions::clean` brings every public function of `functions/clean.zeff` into the script (those whose names do not start with `_`, section 3.6); `use functions::clean::share_missing` brings in one. Functions in `functions/` see every other module's public functions without `use`; a script sees only what it has brought in. Two imported modules that define the same name are an error at the second `use` (Z0107) until the name is written in full, `clean::share_missing`. Built-in functions need no `use`. When a project function has the same name as a built-in one, both must be written in full: `functions::summary` and `data::summary`. Third-party packages, after 1.0, work the other way round: their functions are written `pkg::fn` unless `use pkg` brings them in (section 8.1 of the Engine and Project Specification, rule 9, and section 8.4 of the Engine and Project Specification for what a package is and how it is installed).

### 5.4 Errors, warnings and failed draws

`error(message)` ends the script, `warn(message)` prints a warning and carries on, and `assert(condition, message)` stops when a check fails on a re-run. There is no `try`/`catch`: an analysis that fails at the top level should stop where it fails. Repeated work is different, because one failed draw out of thousands should not throw the rest away. Zeff handles it in two ways, both of which count what failed.

**Built-in resampling** (from 1.1). Everything that takes `reps:` (bootstrap SEs, and later `bootstrap`, permutation tests and simulations) also takes `on_fail: stop | skip = stop` and `max_fail: Float = 0.01`. With `skip`, a draw that fails to converge or errors is left out and counted; the run stops if the failed share passes `max_fail`.

```
let fit = wages.logistic(union ~ educ + exper, se: bootstrap, reps: 2000, seed: 42, on_fail: skip)
```

```
warning[W0801]: 17 of 2,000 bootstrap draws did not converge and were skipped (line 48)
  = note: standard errors use the 1,983 completed draws; failed draws are listed in .zeff/runs/
```

**User loops** (1.0). `attempt(expr)` returns the value of `expr`, or a `Failed` value holding the error's code, message and line. `report`, `len` and other collecting functions skip `Failed` values and say how many; `x.is(Failed)` tests for one.

```
let fits = [for s in 1..200 {
    attempt(sims.where(rep == s).logistic(y ~ treat + x))
}]
report(fits)      // warning: 3 of 200 fits failed (Z0803 at line 51) and were skipped
```

Every warning, including those the engine raises, is printed with its line and kept in the run record. Section 11 lists the diagnostics.

### 5.5 Iterating to convergence

Custom EM algorithms, fixed-point solvers and iterative reweighting need a value that changes from one pass to the next, which immutable bindings rule out. `iterate` supplies it without mutation: it calls a step function on a state until a stopping function says it has converged, and returns the final state.

```
/// True when the log-likelihood has stopped improving.
fn converged(old: Record, new: Record) -> Bool {
    abs(new.loglik - old.loglik) < 1e-8
}

let s0 = {mu1: 0.0, mu2: 1.0, share: 0.5, loglik: -Inf}
let fit = iterate(start: s0, step: em_step, args: {data: wages}, done: converged, max: 500)   // keep: true returns every state
fit.loglik
```

Here `em_step(state: Record, data: Frame) -> Record` is a project function that performs one E and one M step and returns the new state. `iterate` passes `args:` to every call of `step`, records the number of passes in the run record, and warns (W0805) when it stops at `max:` without converging. With `keep: true` it returns the list of every state, for a simulated path or a sampler's chain. Every script still finishes, because `max:` is required to be finite. Each step is an interpreted call, which suits algorithms of hundreds of passes such as EM; samplers of many thousands of draws are the native Bayesian release (1.10), not user code.

### 5.6 Run-time errors and warnings

Every problem that can arise while a correct-looking script runs is one of the following. Errors stop the script, or become a `Failed` value under `attempt`; warnings carry on with a count. Problems found before anything runs are in section 11.

| Situation | Behaviour | Code |
| --- | --- | --- |
| Int overflow in `+`, `-`, `*`, unary `-`, `abs`, `sum`, `running_sum` or `div` | Error naming the row, or the operation when there is no row | Z0211 |
| `div(a, 0)` or `mod(a, 0)` with Int | Error | Z0212 |
| `x / 0` or `log(0)` with Float | `Inf` or `-Inf`, with a warning giving the count and a suggestion (section 4.1) | W0204 |
| `0/0`, `Inf - Inf`, `Inf * 0`, `sqrt(-1)`, `log(-1)` | `NA`, with a warning giving the cause and the count | W0205 |
| `xs[i]` outside `1..len(xs)`, `xs[NA]`, `first([])`, `last([])` | Error giving the index and the length | Z0207 |
| `m[k]` with an absent mapping key | Error naming the key | Z0204 |
| `to(x, T)` or `date(s)` that cannot convert | `NA`, with a warning giving the count and three failing values; a Float with a fraction converted to Int likewise | W0203, W0202 |
| A key of `recode` that no row holds, or that is no level of the category | The other keys are applied, with a warning naming the key | W0206 |
| `slice(s, start, len)` past the end of the text | The characters that exist; no warning | |
| A numeric range `5..2` | An empty list; a variable range out of order is a check-time error (section 3.7) | |
| `if` or `assert` on `NA` or on a column | Error | Z0203 |
| `error(message)`, `assert(false, message)` | Error with the message | Z0001 |
| A `Failed` value used outside a list | Error reporting the original failure's code and line (section 3.7) | Z0220 |
| A step whose estimated memory exceeds the budget | Error before the step starts, with the estimate (section 8.3 of the Engine and Project Specification) | Z0704 |
| A step cancelled from the interface | Error; nothing after it runs, and results so far stay in the session | Z0705 |
| A file missing, unreadable, outside the project or changed since the schema pass | Error naming the path (section 2 of the Engine and Project Specification) | Z0701, Z0702 |
| A CSV value that does not fit the type guessed from the first 1,000 rows | Error naming the row (section 11) | Z0606 |
| A model that cannot be estimated: no variation, perfect separation, a singular design after collinearity handling, non-convergence | Error with the diagnosis (section 11) | Z08xx |
| `today()` or `now()` used | Note with the recorded value (section 3.8) | W0703 |

---

---

## 6. Working with frames

A frame is never changed in place: every frame method returns a new frame, and `let` names the result. There is no active dataset; each call names its frame.

```
let firms = import("data/firms.csv")
let ind_year = import("data/sales_wide.csv")
    .to_long("sales_", id: firm_id, names_to: "year", values_to: "sales")
    .merge(firms, on: firm_id, relation: many_to_one)     // reports unmatched rows
    .aggregate(avg_sales = mean(sales), firms = n(), by: [industry, year])
```

**Names inside a frame's methods.** In a frame method's arguments, a formula or `case`, a bare name is resolved in the order of section 3.7: a name in call position is a function, so a column called `mean` or `date` never hides one; a loop variable or function parameter wins over a column, and a clash between the two is an error (Z0106) rather than a warning, because `for y in vars(lwage, hours) { wages.reg(y ~ educ) }` must never regress a column named `y`; otherwise the name is a column, then a script binding, with a warning when both exist. A binding holding a `Var` or `Vars` stands for the variables it names. A list of bare names or ranges, `[educ, exper]` or `[eng1..eng6]`, is a `Vars` wherever a parameter expects one. Outside a frame method there is no frame to look names up in, so a list of variables is written `vars(lwage, hours)`: in `for` loops and when a function builds a list.

**Statistics as methods and reducers.** Each statistic works on a frame, `wages.mean(lwage)`, and as a reducer inside `add` and `aggregate`, `mean(lwage)`. So `wages.add(dev = lwage - mean(lwage), by: firm)` gives deviations from firm means.

| Task | Functions |
| --- | --- |
| Look | `describe`, `head`, `tail`, `n_rows`, `n_cols`, `variables`, `summary`, `tabulate` |
| Rows | `where`, `sort`, `distinct`, `duplicates`, `sample`, `drop_missing` |
| Variables | `select`, `drop`, `rename`, `add`, `update`, `label`, `recode`; `add` may replace an existing column, with a note naming it, while `update` changes values only where a condition holds |
| Combine | `merge`, `append`; columns other than the keys that both frames share are an error (Z0604) listing them, unless `suffixes: ["_l", "_r"]` is given |
| Reshape and summarise | `to_long` (with `names_type: Str \| Int \| Date = Str` for the column built from the old names), `to_wide`, `aggregate` |
| Panels | `panel`, then `lag`, `lead`, `diff` and `fill_forward` inside `add`; the declaration survives a verb that keeps the unit and time columns unchanged (`where`, `add`, `merge` on other keys) and is dropped with a note by one that does not (`aggregate`, `to_wide`) |
| Read and write | `import`, `save`; `stream` from 1.7 |

**Shared arguments** mean the same wherever they appear and are not repeated in each function's entry.

| Argument | Type | Meaning | Accepted by |
| --- | --- | --- | --- |
| `by:` | `Var` or `Vars` | Repeat the computation within each group; plots draw one panel per group | Statistics, `add`, `update`, `aggregate`, plots, `ate` |
| `group:` | `Var` | The variable that defines groups: the groups a test compares, or the groups a split keeps together (section 7.5). A two-group test (`t_test`, `wilcoxon_test`, `proportion_test`) given one level or more than two stops with an error that suggests `anova` or `kruskal_test` (Z0412) | `t_test`, `wilcoxon_test`, `anova`, `kruskal_test`, `proportion_test`, `boxplot`; `holdout`, `folds` |
| `color:` | `Var` | Colour points, lines or bars by group within one panel | Plots |
| `fe:` | `Terms` | Fixed effects absorbed, as fixest and reghdfe: `fe: firm + year`; `firm:year` for interacted effects; singletons are dropped and reported | `reg`, `ivreg`; `poisson` from 1.2 |
| `re:` | `Var` or `Mapping` | Random effects: `re: school` a random intercept; `re: [school => 1 + x]` an intercept and slope; nested groups as `school:class`, crossed groups as separate entries | `mixed` (from 1.4); later `glmm` and `growth` |
| `se:` | `classical \| robust \| hc2 \| hc3 \| bootstrap`; default `classical`, or `robust` when `weights:` is given with `weight_type: sampling` | Standard-error type; `robust` is HC1, as Stata's `robust`; `bootstrap` from 1.1. With sampling weights, `se: classical` is an error (Z0410): classical errors treat the weights as precision weights and are not valid for probability weights. With `weight_type: frequency`, classical errors are valid and the observation count is the sum of the weights | Regression-type models |
| `cluster:` | `Var`; `Vars` after 1.0 | Cluster-robust standard errors (CR1); one-observation clusters are reported; two-way and multiway clustering arrive as dependence structures of the variance layer (section 8.3, point 2 of the Engine and Project Specification) | Regression-type models |
| `weights:` | `Var` | Weight variable | Models, statistics |
| `weight_type:` | `frequency \| analytic \| sampling = sampling` | Stata's fweight, aweight and pweight. In 1.0 the tests (`t_test`, `anova`, `chi2_test`, `proportion_test` and the rest) refuse `weights:` with `weight_type: sampling` (Z0411) and point to regression with robust errors; design-based tests arrive with `survey()` | Anything with `weights:` |
| `reps:` | `Int = 1000` | Bootstrap or simulation draws (from 1.1) | Anything with `se: bootstrap` |
| `on_fail:` | `stop \| skip = stop` | What happens when one draw fails; skipped draws are counted and reported | Anything with `reps:` |
| `max_fail:` | `Float = 0.01` | Largest share of draws that may fail before the run stops | Anything with `on_fail: skip` |
| `seed:` | `Int` | Random seed; one is chosen and recorded when omitted | Everything that draws random numbers |
| `level:` | `Float = 0.95` | Confidence level | Tests, `ci` |
| `title:` | `Str` | Title | Plots, `report` |

**Panels.** `data.panel(unit: firm, time: year)` declares the panel once. After it, `lag`, `lead`, `diff` and `fill_forward` follow each unit's time order, a gap in time gives `NA` as Stata's `L.` does, and `did` takes its unit and time from the panel. What a gap is depends on the time variable's type: consecutive integers for an Int, consecutive calendar days for a Date, and consecutive periods of its own frequency for a `Period` (section 4.3), so a monthly panel never mistakes a missing month for a 30-day gap; until `Period` is computable, a monthly or quarterly panel uses an Int index such as `year * 12 + month`. `unit:` takes one variable in 1.0; a list of variables for a composite unit (store and product) is an additive change for later.

**Row order.** `sort` is stable: rows with equal keys keep their previous order, and `NA` sorts last. `aggregate` and `tabulate` return groups sorted by the `by:` keys, `NA` last. `distinct`, `duplicates`, `unique`, `first` and `last` refer to the current row order and keep the first occurrence. `merge` keeps the left frame's order and, within one left row, the right frame's order. `append` keeps the left rows, then the right. Group-by never depends on hashing order. These orders are part of the result and are checked by the conformance suite.

**Memory.** Frames are immutable, so they can share storage instead of copying it:

| Operation | What happens in memory |
| --- | --- |
| `select`, `drop`, `rename` | The new frame points at the same Arrow columns |
| `add`, `update` | Unchanged columns are shared; only new or changed columns are allocated |
| `where`, `sort`, `sample` | A selection vector of row positions is recorded; a column is gathered into a compact buffer the first time a later step reads it, so `df.where(year > 2010).add(z = x + y)` computes `z` only for kept rows and returns one compact frame. All remaining columns are gathered at once, and the parent released, when the kept rows are fewer than half of the parent's or when the parent frame's binding is shadowed or leaves scope. A frame's reported size includes any parent buffers it keeps alive. Models always work on compact columns, never through a selection vector |
| A chain | In 1.0 each step runs when called, sharing columns as above. From 1.1 a chain is planned as a whole and computed once, when the result is bound, printed or passed to a model, so intermediate frames never exist in full |
| A binding no longer used | In `zeff run`, freed after its last use in the script; in an interactive session, kept until shadowed or cleared from the object panel |

Because steps share columns and record row selections, a five-step chain on a 1 GB file needs about 1 GB plus its new columns and selection vectors, not 5 GB, which keeps ordinary work inside the browser's 4 GB.


### 6.1 Importing and saving data

One function, `import(path, ...)`, picks the reader from the extension, or from `format:` when the extension is wrong, and applies the same cleaning options to every format: `na_values:`, `types:`, `names:`, `thousands:`, `encoding:`. Each import prints one note saying what the format carried (labels, categories, missing codes, dates) and what it could not. A compressed file (`.gz`, or a `.zip` holding one file) is read as its content. Nothing is fetched from a URL: `import("https://...")` is an error that points to `fetch` (section 8.6 of the Engine and Project Specification). Every imported file's hash is recorded in the run record (section 2 of the Engine and Project Specification). Every format but Parquet and Arrow is parsed once: the first import stores an Arrow copy in `.zeff/cache/`, keyed by the file's hash and the import options, and later runs read the copy until the file or an option changes (section 8.7 of the Engine and Project Specification).

| Format | Extensions | Options beyond the shared ones | What is kept | Release |
| --- | --- | --- | --- | --- |
| Delimited text | `.csv`, `.tsv`, `.txt`, `.dat` | `delim:` (`;` and tab detected), `header:`, `skip:`, `quote:`, `decimal: ","` for files with a decimal comma | Types guessed from 1,000 rows and overridable (section 3.8) | 0.1.0, options complete in 0.2.0 |
| Fixed width | `.txt`, `.fwf` | `widths:` or `positions:`, with `names:` | As text | 0.11.0 |
| Excel | `.xlsx`, `.xlsm`, `.xls` | `sheet:` (name or number), `range: "A2:F500"`, `header:` | Cell types; dates as dates, with both Excel epochs; formulas and merged cells read as their values, with a note | 0.11.0 |
| OpenDocument | `.ods` | As Excel | As Excel | 0.11.0 |
| Parquet and Arrow | `.parquet`, `.arrow`, `.feather` | `columns:` reads a subset without loading the rest | Everything: types, labels, categories, contrasts and the panel declaration, whether Zeff or another Arrow tool wrote the file | 0.11.0 |
| Stata | `.dta`, versions 102 to 119 | — | Variable and value labels, extended missing values `.a` to `.z` as missing codes, dates and times from display formats, `strL` text | 0.11.0 |
| JSON | `.json`, `.jsonl`, `.ndjson` | `records: "data.items"` to the array of records | Records become rows and nested objects dotted columns (`address.city`); an array inside a record becomes an opaque list column, carried until `unnest` arrives in 1.1 (section 4.3) | 0.11.0 |
| SPSS | `.sav`, `.zsav`, `.por` | — | Variable and value labels, user-missing values as missing codes, nominal and ordinal measurement levels as categories | 1.1 |
| SAS | `.sas7bdat`, `.xpt` | `catalog:` for formats | Labels; date formats applied | 1.1 |
| R | `.rds`, `.RData`, `.rda` | `object:` when a file holds several | Data frames and tibbles: factors as categories with their level order, `labelled` columns as labels, `Date` and `POSIXct` as dates and times; a list column is carried as an opaque column (section 4.3) | 1.1 |
| HTML tables | `.html`, `.htm` | `table:` (a number, or text the table contains) | Header rows; footnote markers stripped with a note | 1.1 |
| Local databases | `.sqlite`, `.duckdb` | `table:` or `query:` | Column types; read-only | After 1.1 |

Every reader is pure Rust or permissively licensed (for example `calamine` for spreadsheets and ReadStat for SPSS and SAS), so the browser reads the same formats as the desktop; each format's help page has a "what is lost" line.

**Readers.** `import` dispatches to readers behind one interface in the engine: `sniff` recognises a file from its extension and first bytes; `schema` gives column names, types and metadata (section 4.3) without reading the data, which is what `zeff check` caches in `.zeff/schemas/`; `read` reads with the column projection of `columns:` and, where the format allows, a row filter pushed down; `options` declares the reader's options as a typed signature, from which its help page and Zemp's import pane are generated. Every format in the table is one reader, and a format not in it is one reader away. From 1.11 a third-party package may ship a reader as a component: it receives the file's bytes and returns Arrow with metadata, and never opens a file itself, so the sandbox of section 8.3 of the Engine and Project Specification, point 6 holds. A reader that would need a C library (GDAL for ArcGIS geodatabases, for example) is not written; such a format is converted in its own tool to GeoParquet, or reached through `external()`.

**Saving.** `save(path)` mirrors `import`: CSV and TSV; Parquet and Arrow, lossless, the formats R, Python and Stata all read; Stata `.dta` version 118 with labels; Excel `.xlsx`; JSON records; SPSS `.sav` from 1.1. R users are served through Parquet (`arrow::read_parquet`) rather than `.rds`, whose format is R's internal serialisation. Parquet is written with fixed settings (zstd, row groups of 1,048,576 rows, statistics and a page index), so a file is byte-identical on every machine, and a panel is written sorted by unit and time so that a later `where` on unit or time skips whole row groups; the labels, categories and panel declaration travel under the Zeff Parquet profile (section 8.7 of the Engine and Project Specification). There is no file format of Zeff's own.

---

---

## 7. Formulas and other declarative forms

One formula grammar covers every model, from OLS to SEM: R's notation for regressions, lavaan's operators inside `model { }` blocks, and named arguments (`fe:`, `re:`, `instruments:`) for everything that is not a regressor, so `|` never appears in a formula. Because a model block is code rather than a quoted string, every name and operator in it is checked as it is typed.

### 7.1 Regression formulas and their arguments

| Form | Meaning | Example |
| --- | --- | --- |
| `y ~ terms` | Outcome and regressors; an intercept is included | `lwage ~ educ + exper` |
| `a + b` | Both terms | |
| `a * b` | `a`, `b` and their interaction | `treated * post` |
| `a:b` | The interaction only | `educ:female` |
| `x^2`, `log(x)` | Arithmetic, worked out before fitting, so no `I()` is needed | `exper + exper^2` |
| `category(x)`, `base(x, "level")` | Indicator variables, with a chosen base level | `base(industry, "Retail")` |
| `x1..x9` | A range of variables | `y ~ d + x1..x9` |
| `f + terms` | A formula binding plus more terms is a new formula | `wages.reg(f + educ:female)` |
| `- 1` | No intercept | `y ~ x - 1` |
| `y ~ 1` | The intercept-only model | `wages.reg(lwage ~ 1)` |
| `f - term` | A formula binding less a term is a new formula | `wages.reg(f - female)` |
| `fe: fe1 + fe2` | Fixed effects absorbed, as fixest and reghdfe; `fe:` admits `+` and `:` only, and `*` is an error with help | `reg(y ~ x, fe: firm + year)` |
| `instruments: endog ~ instr` | How the endogenous regressors, which appear in the formula, are instrumented; several of each are joined with `+` | `ivreg(y ~ exper + educ, instruments: educ ~ nearc4)`, `instruments: educ + exper ~ nearc4 + nearc2 + age2` |
| `re: school` | Random intercept by group, in `mixed` (from 1.4), and later `glmm` and `growth` | `mixed(y ~ x, re: school)` |
| `re: [g => terms]` | Random intercepts and slopes; nested groups as `school:class`, crossed groups as separate entries | `mixed(y ~ x, re: [school => 1 + x])` |

Fixed effects, random effects and instruments are named arguments rather than parts of the formula, so every piece of a model is labelled where it is written and a referee never has to count `|` signs. lme4's `(1 + x | school)` is written `re: [school => 1 + x]`, and the editor converts pasted lme4 terms. The instrumented regressors appear in the formula like any other regressor; `instruments:` lists them on its left and the excluded instruments on its right. An endogenous regressor missing from the formula is an error (Z0504); a variable on both sides of `~` is an error (Z0505); fewer excluded instruments than endogenous regressors is under-identification (Z0804); an excluded instrument that also appears in the formula is an included instrument and is moved there with a note. `first_stage` reports Hansen's J for over-identified models and the Durbin–Wu–Hausman endogeneity test.

```
let m1 = wages.reg(lwage ~ educ + exper + exper^2, se: robust)
let m2 = wages.reg(lwage ~ educ + exper + exper^2, fe: firm + year, cluster: firm)

let card = import("data/card.csv")
let iv = card.ivreg(lwage ~ exper + exper^2 + black + educ, fe: region,
                    instruments: educ ~ nearc4, first: true)
let ml = pupils.mixed(score ~ ses + female, re: [school => 1 + ses, school:class => 1])
let cr = reviews.mixed(rating ~ tenure, re: [employee => 1, rater => 1])   // crossed
```

Inside a formula the operators bind as section 3.3 gives them, tightest first: `^` and arithmetic functions, then `:`, then `*`, then `as`, then `+` and `-`, then `~`, `=~` and `~~`, and `:=` last. So `dem65 ~ ind60 as c + dem60 as b` labels each term, and `a * b:c` is `a * (b:c)`, as in R.

### 7.2 From formula to model matrix

A formula becomes a design matrix through fixed rules, applied in this order. Any rule that changes what the user wrote is reported in a note, and every note reaches the printed result and the `report` table footer.

| Rule | Detail |
| --- | --- |
| Expansion | `a * b` becomes `a + b + a:b`; `a * b * c` gives every interaction up to third order. `interact([a, b, c], order: 2)` gives all two-way interactions without writing them out |
| Duplicates | A term written twice is kept once, with a note |
| Intercept | Included unless `- 1` is written. With `- 1`, the first categorical variable gets all K indicators and later ones K − 1 |
| Categories | K − 1 indicator columns, coded by the variable's contrast (section 4.2); the base level is the first or the one named by `base()`. A category in an interaction whose main effect is absent gets full coding, as R does |
| Arithmetic | Inside brackets and function calls, operators are arithmetic: `log(a + b)` is one column and `(a + b)^2` squares the sum. Because R reads `(a + b)^2` as all two-way interactions, Zeff warns (W0503) and suggests `interact([a, b], order: 2)` |
| Collinearity | After the fixed effects are projected out, each regressor is scaled to unit norm; a regressor whose remaining norm is below 1e-8 of its original norm is absorbed by the fixed effects and reported so ("absorbed by fe: firm"). Pivoted QR then runs on the scaled columns, each pivot tested relative to its own column's norm (tolerance 1e-7): a column that fails is dropped as collinear with the columns before it, named in a note, and shown as "(omitted)" by `report`. Coefficients and the variance matrix are unscaled afterwards. The condition number of the scaled R is reported, with warning W1001 above 1e8 |
| Rows | Rows with `NA` in a model variable are dropped and counted; singletons in `fe:` groups are dropped and counted |
| Fixed effects | Applied in this order: drop rows with `NA`, then singletons; project the fixed effects out of the outcome and regressors; then run pivoted QR on what remains. Projection uses alternating projections with acceleration, as fixest does, demeaning only the model's columns into a scratch buffer of about n × (k + 1) values, to a tolerance of 1e-8 recorded in the run. With `fe:` the intercept is absorbed, every category gets K − 1 indicators, and `- 1` has no effect |
| Random effects (1.4) | `re:` builds a sparse Z matrix (compressed sparse column) beside the dense X |
| Term names | Every column of the model matrix has a stable name, used by `coef`, `report`, `test` and `derive`: a variable keeps its name (`educ`); a category level is `industry=Retail`; an interaction joins its parts with `:` (`educ:female`, `industry=Retail:female`); a transformation keeps its written form (`log(wage)`, `exper^2`); the intercept is `(intercept)`. A name that is not a valid identifier is given as text: `m.coef("industry=Retail")`; a level containing `=`, `:` or `+` is quoted inside the term name, `industry="A:B"`, so it cannot read as an interaction. Inside `fit.test` and `fit.derive`, bare names are term names and resolve to coefficients, never to columns; `test` takes Bool expressions, `fit.test(educ == exper, educ + exper == 1)` |

### 7.3 Causal graphs

A `dag { }` block states the causal assumptions behind an analysis, as code that is checked and can be tested against the model actually fitted. Like a formula, a graph is a value, kept in the run record.

```
let g = dag {
    ability -> training -> productivity
    ability -> productivity
    training -> tenure -> productivity
    [age, region] -> training
    latent ability
}
g.adjustment_sets(exposure: training, outcome: productivity)
g.implied_independencies()
let m = staff.reg(productivity ~ training + tenure + age + region)
m.check(g, exposure: training)
```

| Element | Meaning |
| --- | --- |
| `a -> b` | Direct causal effect; chains such as `a -> b -> c` allowed |
| `a <-> b` | Unobserved common cause |
| `a -- b` | Undirected link, for drawing only; adjustment sets are refused while one is present |
| `[a, b] -> c` | Several causes at once |
| `latent a, b` | Unobserved variables: never proposed for adjustment |
| `selected a, b` | Variables the data are conditioned on by design (only employed workers observed, only surviving firms); treated as conditioned in every criterion, so the paths this opens are reported, and never proposed for adjustment |

| Function | What it does |
| --- | --- |
| `g.adjustment_sets(exposure:, outcome:)` | Minimal sufficient adjustment sets by the generalized adjustment criterion, with latent and selected nodes taken into account; when none exists among observed variables, says so and suggests an instrument or a front-door design |
| `g.implied_independencies()` | Conditional independencies the graph implies, which the data can test; the basis set, as dagitty returns it, since the full set grows exponentially with the number of nodes |
| `fit.check(g, exposure:)` | Evaluates the generalized adjustment criterion (Shpitser, VanderWeele and Robins 2010; as dagitty implements it) for the model's regressors, by d-separation on the graph with latent and selected nodes expanded. A regressor that is a descendant of a node on a proper causal path from the exposure to the outcome is warned as removing part of the effect (W0510). A set that leaves a back-door path open, or opens one by conditioning on a collider or a descendant of one, including a collider that is not a descendant of the exposure (M-bias), is warned with the path named (W0511). When one variable both blocks a path and opens another (butterfly bias), both facts are stated and no observed set is recommended. A descendant of the exposure on no such path, and any pure cause of the outcome, produces no warning, since conditioning on it is harmless for bias. The check never refuses to estimate |
| `g.plot()` | Draws the graph in the plot viewer, latent nodes dashed and the exposure and outcome marked; in 1.0, from 0.10.0 |
| `iv.exclusion_check(g, instrument:)` | Checks the exclusion restriction: the instrument is d-separated from the outcome, given the regressors, in the graph with the exposure's outgoing edges removed, and is not a descendant of the exposure |

A directed cycle is an error naming it. Names in a graph are checked against a frame only when the graph is used with one. The lexer enters a graph mode at `dag {`, where `->`, `<->` and `--` are tokens; elsewhere `a--b` is `a - -b`, and `zeff fmt` writes it with spaces.

```
warning[W0510]: tenure lies on the path training → tenure → productivity
  = note: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect
warning[W0511]: no sufficient adjustment set exists: ability is latent and confounds training and productivity
  = help: consider an instrument for training, or a front-door design
```

### 7.4 Model blocks (from 1.3)

`model { }`, `cfa` and `sem` arrive in 1.3; the syntax is fixed now so that the parser, editor and Zemp can be designed around it.

```
let pd = import("data/political_democracy.csv")
let m = model {
    // measurement
    ind60 =~ x1 + x2 + x3
    dem60 =~ y1 + y2 + y3 + y4
    dem65 =~ y5 + y6 + y7 + y8
    // structural paths, labelled
    dem60 ~ ind60 as a
    dem65 ~ ind60 as c + dem60 as b
    // residual covariance and a defined effect
    y1 ~~ y5
    indirect := a * b
}
let fit = pd.sem(m, se: bootstrap, reps: 2000, seed: 42, on_fail: skip)
fit.effects()
```

| Operator | Meaning | lavaan equivalent |
| --- | --- | --- |
| `f =~ items` | Latent factor measured by items | `=~` |
| `y ~ x` | Regression path | `~` |
| `a ~~ b` | Covariance; `a ~~ a` a variance | `~~` |
| `x as a` | Label a path; `*` keeps its one meaning, interaction | `a*x` |
| `d := a * b` | Defined parameter, such as an indirect effect | `:=` |
| `efa(f1, f2, f3) =~ x1..x9` | A rotated exploratory block (ESEM); the 1.3 model parser accepts it, and fitting it waits for 1.6 | `efa("block1")*f1 + ...` |

In 1.3, `cfa` and `sem` fit continuous items by ML or MLR, with FIML for missing data. Items declared `ordered: true` with `category()` switch the fit to WLSMV from 1.6; until then they are an error that suggests `estimator: mlr` with `ordered: false` (section 4.2).

### 7.5 Splits, recipes and leakage (from 1.5)

Data leakage is information from the rows a model is judged on reaching the model before the judgement: a standardisation computed on all rows, an imputation that saw the test set, a lagged variable that looks forward, a unit whose rows sit on both sides of a split, a test set scored until it agrees. tidymodels prevents it by discipline: split first, express preprocessing as a `recipe` that is estimated on the analysis rows and applied to the assessment rows, re-estimate it inside every fold, touch the test set once. Zeff adopts that discipline and adds what a typed language can: since the IR knows every column's lineage, the checker finds the leaks the discipline misses. The functions arrive in 1.5 with the lasso and forests; the rules are fixed here so that the IR carries lineage from the start.

**Splits are roles, not copies.**

| Function | Rule |
| --- | --- |
| `data.holdout(frac: Float = 0.2, seed: Int, strata: Vars, group: Var, time: Var)` | Marks each row as training or testing; nothing is copied. `strata:` stratifies, `group:` keeps rows sharing a value on one side (a `panel` unit by default), and `time:` makes the split chronological, the latest rows testing. The seed is recorded |
| `data.training()`, `data.testing()` | Views of a split frame, sharing its columns. A testing view may be scored once per run; fitting on it, or scoring it again, is a warning (W1105), the "last fit" rule made explicit. Results computed on the training view are labelled exploratory in Zemp |
| `data.folds(k: Int = 5, repeats: Int = 1, strata: Var, group: Var, time: Var, seed: Int)` | A `Folds` value with the same options; `time:` gives rolling-origin folds that never assess on earlier rows than they train on |

**Recipes are fitted transformations.** A `Recipe` is built with the chain syntax every frame uses; no new grammar. Every step that needs a statistic is estimated on the analysis rows and applied with those estimates to the assessment rows, inside every fold and on the testing view, by the engine rather than by the user's care.

```
let d = loans.holdout(frac: 0.2, seed: 7, strata: default, group: borrower)

let rec = recipe()
    .impute([income, age], with: median)          // medians from the analysis rows only
    .add(lincome = log(income))                   // row-wise: nothing to estimate
    .standardize([lincome, age, x1..x9])          // means and sds from the analysis rows
    .one_hot(industry, other: 0.01)               // levels fixed at fit; rare levels pooled
    .pca([x1..x9], n: 5)                          // loadings from the analysis rows

let cv    = d.training().folds(k: 10, repeats: 3, strata: default)
let tuned = cv.tune(default ~ . - id, recipe: rec, model: spec.lasso(penalty: grid(20)), metric: auc)
let fit   = d.training().learn(default ~ . - id, recipe: rec, model: spec.lasso(penalty: tuned.best()))
fit.score(d.testing())                            // the one permitted look at the test rows
```

| Rule | Detail |
| --- | --- |
| Steps | `impute`, `standardize`, `center`, `one_hot`, `bin`, `pca` and `winsorize` estimate on the analysis rows; `add` is row-wise, and a cross-row reducer inside it is an error that names the estimated step to use instead (Z0506) |
| Order | Steps apply in the order written; a step may use columns an earlier step made |
| Levels | `one_hot` fixes the levels at fit; a level first seen at apply goes to `other` with a note, never to a new column |
| Inspection | `rec.prepare(train)` estimates the steps and `.apply(frame)` applies them, for looking at what the model saw; `learn`, `tune` and `score` do both without being asked |
| Model specifications | `spec.lasso(...)`, `spec.random_forest(...)`, `spec.boosting(...)`, `spec.logistic()` and `spec.reg()` are unfitted specifications, a namespace like `dist.` and `power.`; a parameter given as `grid(20)` or `grid([0.01, 0.1, 1])` is tuned |
| Formulas | `y ~ .` means every column except the outcome and the columns with a role: the split marker, a `panel` unit and time, and anything removed with `- id`; an identifier left in is warned (W1106) |
| Fitting | `data.learn(formula, recipe:, model:)` returns a `LearnFit` with the result and estimator interfaces plus `score`, `calibration`, `shap` and `partial_dependence`; `folds.tune(...)` returns a `TuneResult` whose `best()` picks the parameters by the metric |

**The leakage checker.** When a model is fitted on a training view or inside folds, `zeff check` walks each predictor's lineage in the typed IR and warns, with the fix, under the statistical-warning family W11xx; `--deny-warnings` turns them into errors for a replication package.

| Code | Finds | Fix offered |
| --- | --- | --- |
| W1101 | A predictor computed with a cross-row statistic (`mean`, `sd`, `quantile`, `standardize`, `impute`, `rank`, `winsorize`, `pca`, target encoding) over rows that include assessment rows | Move the step into the recipe |
| W1102 | A predictor whose definition uses the outcome, or a column derived from it (`add(ratio = y / x)`) | Drop it, or keep it deliberately with `// zeff: allow(W1102) reason` |
| W1103 | A split that puts rows of one `panel` unit, or one `cluster:` group, on both sides | `group:` |
| W1104 | A `lead()` column, or any column whose lineage reaches later time periods than the row's, used as a predictor on a panel | `lag`, or `holdout(time:)` |
| W1105 | The testing view fitted on, or scored more than once in a run | Score once; tune on folds of the training view |
| W1106 | `~ .` pulling in an identifier, key or high-cardinality text column | `- id`, or declare the column's role |
| Note | Exact duplicate rows on both sides of a split, with the count | `distinct` first |

**Cross-fitting in the causal methods.** `double_lasso`, `dml` and `causal_forest` split the sample so that nuisance estimation never sees the rows it is evaluated on (`folds: 5` and honesty by default), with the split seed recorded; `fit.check(g)` (section 7.3) catches the causal form of leakage, conditioning on post-treatment variables, which no split can fix.

### 7.6 Plots: a layered grammar

A plot is built the way Wilkinson's *Grammar of Graphics* and ggplot2's layered form describe it: data and aesthetic mappings, then layers (a mark, or a statistical transformation drawn as marks), then scales, coordinates, facets and the theme. In Zeff a layer is a method, so there is no operator to overload and no new grammar; the defaults follow Observable Plot, so most plots are one layer. The renderer is built on this grammar from 0.7.0, and the ten plotting functions of section 9 of the Engine and Project Specification are shorthands over it that return the same `Plot` value.

```
plot(wages, x: educ, y: lwage, color: female)      // data and aesthetic mappings
    .points(alpha: 0.3)                            // a mark
    .smooth(method: lm)                            // a statistical layer (from 1.2); .smooth(fit: m) draws a fitted model
    .facet(by: industry, scales: free_y)           // small multiples
    .scale_y(log)
    .labels(title: "Returns to schooling by industry", y: "log wage")
    .theme(journal)
    .export("output/fig2.pdf", size: two_column)

wages.scatter(educ, lwage)                         // the shorthand: the same Plot value
```

**Mappings and constants.** In an aesthetic argument a bare name maps a variable (`color: female`) and a string sets a constant (`color: "steelblue"`); the parameter's type, `Var | Str`, decides, as it does for words and variables everywhere else (section 3.7). A layer may name its own frame (`.points(data: outliers)`) or override a mapping.

| Part | Zeff | Rule |
| --- | --- | --- |
| Data and aesthetics | `plot(frame, x:, y:, color:, fill:, size:, shape:, group:, label:)` | `x` and `y` may be variables or expressions over the frame |
| Marks | `points`, `lines`, `step`, `bars`, `area`, `ribbon`, `text`, `tile`, `rug`, `hline`, `vline` | `bars` with `y` draws values and without `y` counts, and its baseline is always zero; `hline` and `vline` take a value or a variable |
| Statistical layers | `histogram(bins:)`, `density(bandwidth:)`, `boxplot`, `errorbars(ci:)`; `smooth(method: lm \| loess)`, `violin`, `aggregate(fun:, errorbars:)` from 1.2 | Computed by the engine's own functions, never by a package above `plot`: `smooth(method: lm)` and `scatter(fit: linear)` fit by the kernel's least squares (the QR `reg` also uses) with a classical band, `loess` and `density` come from `stats`, and `smooth(fit: m)` draws any fitted model's predictions and intervals through the result interface declared in `core`, so a robust, clustered or fixed-effects line is exactly the model `m` prints |
| Scales | `scale_x(linear \| log \| sqrt \| date, limits:, breaks:, labels:)`, `scale_y(...)`, `scale(color:, fill:, size:, shape:)` | A variable's label becomes its axis title; numbers format as in tables (section 3.8); a log scale on a value at or below zero is an error naming the rows |
| Coordinates | `coord(flip \| polar \| fixed)` | Coordinate systems are a registry of the renderer; `coord(map, crs:)` joins it with the `spatial` package (section 4.3) |
| Facets | `facet(by:, rows:, cols:, scales: fixed \| free \| free_x \| free_y)` | `by:` keeps its one meaning; panels share scales unless told otherwise |
| Guides and theme | `labels(title:, subtitle:, x:, y:, color:, caption:)`, `legend(right \| top \| bottom \| none \| direct)`, `theme(journal \| minimal \| slide)`, `annotate(text:, x:, y:)` | One default theme: white background, major gridlines only, no chart junk |
| Composition and inspection | `arrange([p1, p2], cols:)` and `p.computed()` from 1.2 | `computed` returns each layer's computed data (the histogram's bins, the smooth's fitted values) as frames |
| Result plots | `fit.plot()`, `fit.coef_plot()` | Built from the fit's estimates with `points` and `errorbars`; `plot` chooses the layers by result type (an event study for `did`, a coefficient plot for `reg`) |

**Shorthands.** `histogram(x)` is `plot(x:).histogram()`; `scatter(x, y)` is `plot(x:, y:).points()`; `line` is `.lines()`; `bar` is `.bars()`; `boxplot(y, group:)` is `plot(x: group, y:).boxplot()`; `density` is `.density()`; `heatmap(vars)` computes the correlations and draws `.tile()`; `qq_plot` draws the quantile pairs with `points` and a reference `lines` layer. Zemp's quick plots write the shorthands; its plot dialog shows the layers.

**Defaults.** These are fixed, since they are where most of a plot's quality lies, and a figure re-renders identically on every machine.

| Default | Rule |
| --- | --- |
| Colour | Okabe–Ito for categories (eight colour-blind-safe colours, in a fixed order), viridis for continuous scales; never a rainbow; `scale(color: [...])` overrides |
| Categories | Ordered categories keep their order; value labels appear as written; `NA` is its own category, labelled `NA` |
| Missing values | Rows dropped for `NA` in a mapped variable are counted in a note on the plot, as everywhere else (principle 6) |
| Bins and bandwidth | Freedman–Diaconis bins; Silverman's rule for densities; both overridable |
| Randomness | Jitter and sampling for large plots draw from the seeded stream (section 8.3 of the Engine and Project Specification), so a figure re-renders identically |
| Text | Sizes in points; never below 8 pt in exports; labels wrapped or thinned rather than overlapping |
| Sizes | `one_column`, `two_column`, `slide`, or a width and height in millimetres; `export` writes SVG or PDF (vector) or PNG at 300 dpi (raster), chosen by the extension |
| Scene | A resolution-independent scene the viewer redraws to fit (Zemp Specification, section 7); exported to PDF, SVG and PNG |

Accuracy class: exact. The same data give the same scene, checked by snapshot tests; the statistical layers inherit the class of the function that computes them.

---

**Rendering and export.** A plot is a *scene*: marks in data coordinates with their scales, guides and layout, a versioned format of the engine (section 8.3 of the Engine and Project Specification, point 5) that Zemp draws at the size of its pane and that `export` writes as SVG or PDF, both vector, or PNG. The renderer is part of the engine, in Rust, with its own fonts embedded (IBM Plex Sans and Mono, under the Open Font License) and never the system's, so a figure's SVG and PDF bytes are identical on every machine, as the numbers are; text in non-Latin scripts uses Noto subsets fetched on first use and cached like a dataset (section 8.5 of the Engine and Project Specification), with the subset's hash recorded in the run record. Drawing beyond data plots stays inside the grammar: `annotate` is in 1.0; `segment`, `rect`, `arrow` and `path` marks, `.plot()` for SEM path diagrams (`g.plot()` for causal graphs is already in 1.0, section 7.3), maps through `coord(map, crs:)` and `arrange` for multi-panel figures are additive (section 14.1 of the Engine and Project Specification).

---

## 10. A complete analysis, with its output

### 10.1 The script

One project script that loads and cleans data, describes it, runs regressions, an IV model, a two-period DiD, a test, a logistic regression and a reliability check, and exports tables and figures. Every line is valid Zeff 1.0.

```
// scripts/main.zeff — returns to schooling and a staggered policy
seed(20261015)

// 1. Load and clean
let wages = import("data/wages.csv", na_values: [-99])
    .where(year >= 2010)
    .add(lwage = log(wage),
         exper2 = exper^2,
         industry = category(industry),
         age_group = case {
             age < 25 => "young",
             age < 55 => "prime",
             _        => "older",
         })
    .label(female, [0 => "male", 1 => "female"])
    .panel(unit: worker, time: year)

// 2. Describe
assert(wages.n_rows() > 0, "no rows left after filtering")
wages.describe()
wages.summary([lwage, educ, exper], by: female)
wages.tabulate([industry, age_group], percent: row)
wages.histogram(lwage, by: female).export("output/fig1.pdf", size: one_column)

// 3. Regressions, collected into one table
let f = lwage ~ educ + exper + exper2 + female
let m1 = wages.reg(f, se: robust)
let m2 = wages.reg(f, fe: industry + year, cluster: worker)
report([m1, m2], stats: [n_obs, r_squared]).export("output/table1.docx")

let fits = [for y in vars(lwage, hours) {
    wages.reg(y ~ educ + exper, fe: industry + year, cluster: worker)
}]
report(fits).export("output/table2.docx")

// 4. Instrumental variables
let card = import("data/card.csv")
let iv = card.ivreg(lwage ~ exper + exper^2 + black + educ, fe: region,
                    instruments: educ ~ nearc4, se: robust, first: true)
iv.first_stage()                         // effective F and Kleibergen–Paap F

// 5. Two-period difference-in-differences (staggered designs: `did` from 1.2)
let counties = import("data/counties.csv").panel(unit: county, time: year)
let dd = counties.reg(emp ~ treated * post, fe: county + year, cluster: county)
dd.coef_plot().export("output/fig2.pdf", size: two_column)

// 6. A test and a binary outcome
wages.t_test(lwage, group: female)
let lg = wages.logistic(union ~ educ + exper + female, cluster: worker)

// 7. Reliability of a survey scale (CFA and SEM from 1.3)
let engage = import("data/engagement.csv")
engage.reliability([eng1..eng6])
```

### 10.2 What Zeff prints

Printed output is plain text with aligned columns, so it reads the same in Zemp, a terminal and a log file. Zemp renders the same results as interactive cards. The numbers below are illustrative.

`wages.describe()`

```
wages · 18,412 rows × 13 variables · panel worker × year

  variable    type       non-missing   missing   example
  worker      Int             18,412         0   1042
  year        Int             18,412         0   2014
  wage        Float           18,201       211   23.40
  lwage       Float           18,201       211   3.153
  educ        Int             18,412         0   16
  exper       Int             18,390        22   7
  exper2      Float           18,390        22   49
  age         Int             18,412         0   31
  hours       Float           18,377        35   40.0
  union       Int             18,412         0   0
  female      Int (lab)       18,412         0   female
  industry    Category        18,412         0   Retail (12 levels)
  age_group   Str             18,412         0   prime
```

`wages.summary([lwage, educ, exper], by: female)`

```
summary · by female

  female   variable        N   missing     mean       sd      min      max
  male     lwage       9,604       118    3.214    0.512    0.693    5.991
  male     educ        9,722         0   13.402    2.611    0.000   20.000
  male     exper       9,709        13    9.874    6.203    0.000   38.000
  female   lwage       8,597        93    3.061    0.498    0.811    5.703
  female   educ        8,690         0   13.718    2.488    0.000   20.000
  female   exper       8,681         9    8.902    5.977    0.000   36.000

warning: 211 rows with NA in lwage and 22 with NA in exper were skipped (line 21)
```

`m2` (a statement that is not a `let` prints its value)

```
reg · lwage ~ educ + exper + exper2 + female · fe: industry + year
N = 18,184 (228 rows dropped for NA in model variables; no singletons)
fixed effects: industry (12), year (12)
standard errors: clustered by worker (3,911 clusters, CR1)

  term        estimate   std.error   t value   p value        95% CI
  educ          0.0743      0.0061     12.18    <0.001   [0.0623, 0.0863]
  exper         0.0412      0.0048      8.58    <0.001   [0.0318, 0.0506]
  exper2       -0.0007      0.0001     -7.00    <0.001   [-0.0009, -0.0005]
  female       -0.1120      0.0215     -5.21    <0.001   [-0.1541, -0.0699]

R² within 0.262 · R² overall 0.301
```

`wages.t_test(lwage, group: female)`

```
t_test · Welch two-sample · lwage by female

  group      N      mean      sd
  male    9,604    3.214   0.512
  female  8,597    3.061   0.498

difference (male − female) = 0.153   95% CI [0.138, 0.168]
t = 20.42   df = 18,074.2   p < 0.001
Hedges' g = 0.30   95% CI [0.27, 0.33]
```

`report([m1, m2])` as printed text (the exported .docx and .tex carry the same layout)

```
                      (1)            (2)
  educ             0.071***       0.074***
                  (0.004)        (0.006)
  exper            0.039***       0.041***
                  (0.003)        (0.005)
  exper2          -0.001***      -0.001***
                  (0.000)        (0.000)
  female          -0.129***      -0.112***
                  (0.011)        (0.022)
  ───────────────────────────────────────
  Fixed effects       none   industry, year
  SEs               robust   cluster worker
  N                 18,184           18,184
  R²                 0.284    0.262 (within)

  * p < 0.05, ** p < 0.01, *** p < 0.001
```

Warnings and notes print under the output they belong to and are kept in the run record:

```
note: 211 values of wage read as NA from na_values -99 (line 5)
note: seed 20261015 set at line 2; recorded in .zeff/runs/2026-10-15T21-04/
warning: 12 rows gave −Inf from log(sales) where sales = 0 (line 9 of another script)
  = help: use log1p(sales), or filter with where(sales > 0) first
```

---

---

## 11. Diagnostics: errors, warnings and notes

Zeff reports problems the way Rust does: a code, the location, a marker under the problem and a suggestion.

**Checking before running.** Names, argument values, types and variable lists are checked before any model runs, so a mistake late in a long script stops the run at once. Checking needs each data file's columns but not its contents: the schema pass reads only the header and column types of every imported file (the first 1,000 rows of a CSV; the metadata of Parquet and Stata files) and caches them in `.zeff/schemas/`, refreshed when a file's size or modification time changes. It then works out the columns after each step (`add`, `select`, `rename`, `merge`, `aggregate` and `to_long` all produce known columns). Steps whose columns depend on the data, such as `to_wide` and an `import` of a file that does not yet exist, are marked "checked at run time", and the editor shows that mark. After each run, the columns such steps actually produced are cached in `.zeff/schemas/`, and later checks use them, marked "from last run", so code after a `to_wide` is still checked. Because a CSV's types are guessed from its first 1,000 rows, a later value that does not fit (text in a column read as numbers) stops the run with error Z0606, naming the row, the column and the value; Zeff never coerces it to `NA` silently, and the help suggests `import(..., types: {col: Str})` or fixing the file. A diagnostic whose only evidence is a schema cached from the last run is a warning, marked "from last run", never an error, since the data may have changed since. `zeff check` runs only this stage, and reports every error it finds, not only the first; `zeff run` stops at the first.

| Kind | When | Effect |
| --- | --- | --- |
| Error | The script cannot continue | Stops the run; nothing after it executes |
| Warning | The result stands but needs attention; always with a count | Printed under the output and kept in the run record |
| Note | Information worth keeping, such as a seed or a conversion | Kept in the run record; shown in Zemp's log |

| Codes | Area |
| --- | --- |
| Z01xx | Names: variables, bindings, functions, packages |
| Z02xx | Types and values |
| Z03xx | Syntax and layout |
| Z04xx | Arguments |
| Z05xx | Formulas and model blocks |
| Z06xx | Data: import, merge, reshape |
| Z07xx | Projects, paths, data roots, versions, network requests (Z0708), installation and engines (Z0709 to Z0711, section 2.1 of the Engine and Project Specification), resources and reproducibility (W0703 marks a use of `today()` or `now()`) |
| Z08xx | Estimation: convergence, identification |
| W0xxx | Warnings, numbered by the same areas (W08xx includes failed resampling draws) |
| W1xxx | Statistical warnings: the code is right but the result is fragile (section 11.1) |
| Z9xxx | Internal errors: a bug in Zeff, not in the script (section 11.1) |

**Codes numbered while the engine was built.** These conditions are described in the sections named, or in the Engine and Project Specification, and received their numbers during releases 0.1.0 to 0.3.0 (open decisions #41 to #44 and #46; Z0413 and Z0111 with the distributions). `zeff help <code>` gives each one's explained page.

| Code | Condition |
| --- | --- |
| Z0301, Z0303, Z0304 | Something the grammar does not allow at that place; a closing bracket with no opener; a construct the grammar has but this release does not run |
| Z0103, Z0105, W0101 | A call names no function; a `let` reuses a name of `core`; a `let` hides a function of another package (warning) |
| Z0401, Z0404 to Z0409 | An argument of the wrong type; an unknown argument name; an argument given twice; a required argument missing; too many positional arguments; a positional argument after a named one; a date format that is not one |
| Z0221 | An operator applied to types it does not take (a Bool in arithmetic, `!` on a number, `&&` on numbers, a comparison across types, an index of the wrong kind) |
| Z0222 | A mapping key written or computed twice |
| Z0410 | Two options of one call that exclude each other, given together (`round` with `digits:` and `sig:`; `se: classical` with sampling weights) |
| Z0413 | A parameter of a distribution outside its domain, or `NA` |
| Z0111 | `seed` called after a draw, or a second time |
| W0102 | A name in a frame method's argument that is both a variable of the frame and a binding: the variable is used, with a warning |
| W0201, W0204 | Missing values skipped by a reducer, with the count; `Inf` and `-Inf` both present, so the result is `NA` |
| Z0712 to Z0715 | `zeff.toml` is not valid TOML; valid TOML but wrong for Zeff; a script named on the command line that the project lacks; `import` of a project path with no readable file |

Examples:

```
error[Z0101]: no variable `incme` in `poll`
 --> scripts/main.zeff:4:11
  |
4 | poll.mean(incme)
  |           ^^^^^ not a variable of poll
  |
  = help: did you mean `income`? If you meant a category level, write it in quotes: `"income"`
```

```
error[Z0203]: `if` needs one Bool, but `wage > 20` is a Bool column
 --> scripts/main.zeff:12:4
   |
12 | if wage > 20 {
   |    ^^^^^^^^^ a column: one value per row
   |
   = help: for row-by-row values use `add(high = case { wage > 20 => 1, _ => 0 })`
```

```
error[Z0302]: this `(` is never closed
 --> scripts/main.zeff:7:18
  |
7 | let m = wages.reg(lwage ~ educ + exper, se: robust
  |                  ^ opened here
8 | report([m])
  | ------ the statement seems to end before this line
```

```
error[Z0402]: `robus` is not a value of `se:`
 --> scripts/main.zeff:9:37
  |
9 | let m = wages.reg(lwage ~ educ, se: robus)
  |                                     ^^^^^ expected one of classical, robust, hc2, hc3 (bootstrap from 1.1)
  |
  = help: did you mean `robust`?
```

```
error[Z0504]: `educ` is instrumented but is not a regressor
 --> scripts/main.zeff:15:40
   |
15 | card.ivreg(lwage ~ exper, instruments: educ ~ nearc4)
   |                                        ^^^^ instrumented here, missing from the formula
   |
   = help: add it to the regressors: `lwage ~ exper + educ`
```

```
error[Z0603]: many-to-many merge refused
 --> scripts/main.zeff:21:6
   |
21 |     .merge(firms, on: firm_id)
   |      ^^^^^ firm_id repeats in both frames (412 keys on the left, 37 on the right)
   |
   = help: find the repeated keys with `firms.duplicates(firm_id)` and remove them, then state `relation: many_to_one`
```

```
error[Z0701]: path `../shared/wages.csv` is outside the project
 --> scripts/main.zeff:2:20
  |
2 | let wages = import("../shared/wages.csv")
  |                    ^^^^^^^^^^^^^^^^^^^^^ projects read only their own files and declared data roots
  |
  = help: declare the folder under [data] in zeff.toml and write `import("shared:wages.csv")`, or copy the file into data/
```

```
error[Z0606]: row 14,207 of `data/wages.csv` has text in a numeric column
 --> scripts/main.zeff:5:13
  |
5 | let wages = import("data/wages.csv", na_values: [-99])
  |             ^^^^^^ column `hours` read as Float from the first 1,000 rows; row 14,207 holds "varies"
  |
  = help: declare it with `types: {hours: Str}`, add "varies" to `na_values:`, or fix the file
```

```
error[Z0803]: logistic: `female` perfectly predicts `union` (separation)
 --> scripts/main.zeff:49:16
   |
49 | let lg = wages.logistic(union ~ educ + exper + female, cluster: worker)
   |                ^^^^^^^^ estimates diverge; maximum likelihood does not exist
   |
   = help: drop or combine the predictor, or check how `union` was coded
```

### 11.1 Error handling and bug reports

Zeff separates three kinds of problem: the script is wrong (Z01xx to Z08xx), the result is fragile (W1xxx), and Zeff itself is wrong (Z9xxx). The third must never pass as a result.

**Handling problems in a script.**

| Feature | Behaviour | Release |
| --- | --- | --- |
| Explained codes | Every code has a page with its meaning, common causes, a small example and the fix, shown by `zeff explain Z0402` and linked from every message in the editor and Zemp | Pages from 0.2.0, growing with each code; `zeff explain` and the editor links from 0.8.0 |
| Statistical warnings (W1xxx) | Printed beside the result and listed in the `report` table's footer: near-singular regressors with the condition number (W1001), fewer than 30 clusters, suggesting the wild bootstrap (W1002), extreme weights with the effective sample size (W1003), little variation left after fixed effects (W1004), sparse cells in a χ² test (W1005), and observations whose leverage drives a coefficient (W1006); leakage into a predictive model, W1101 to W1106 (section 7.5); each function's help page lists the codes it can raise (section 9.1 of the Engine and Project Specification) | 0.4.0, growing with each method |
| Offending rows | A data error names the chain step and lists the first five offending rows, with a statement that selects them all, such as `wages.where(is_missing(exper))`; Zemp shows them with one click | 0.6.0 |
| Deliberate exceptions | `// zeff: allow(W0203) "varies" coded as NA by design` silences one warning on one line; the reason is kept in the run record. A pragma is a comment beginning `// zeff:` on its own line above a statement or at the end of the statement's first line; it attaches to the whole statement, continuation lines included; `allow(CODE)` accepts W codes only, the reason text is required, and an unknown code or a missing reason is an error (Z0317). `zeff run --deny-warnings` turns every warning not allowed this way into an error, for replication packages | 0.7.0 |
| Repeated warnings | A warning raised many times, as in a loop, prints once with its count and the first few cases | 0.5.0 |

**Protecting against engine bugs.**

| Feature | Behaviour | Release |
| --- | --- | --- |
| Internal errors (Z9xxx) | A failure inside the engine never crashes the window or shows a stack trace: it reads "error[Z9001] internal error: this is a bug in Zeff, not in your script", offers to build a bug report, and the session survives | 0.2.0 |
| Self-checks | Cheap checks run after every fit: residuals orthogonal to the regressors, a symmetric positive semi-definite variance matrix, converged demeaning, a log-likelihood that never fell during iteration. A failed check is reported as an internal error, never returned as a result | 0.4.0, with each estimator |
| Property tests | Run on every build: scaling the outcome scales the coefficients; reordering rows changes nothing; a variable constant within groups is absorbed by the fixed effects; adding a duplicate regressor drops it with a note | 0.4.0 |
| Fuzzing | Random programs are fed to the parser and checker, which must answer with a diagnostic, never a crash | 0.2.0 |
| Message snapshots | Every diagnostic's wording and position is checked against a stored copy on every build | 0.2.0 |

**Reporting and fixing bugs.**

| Feature | Behaviour | Release |
| --- | --- | --- |
| `zeff report-bug` | Builds a local bug report: engine version, edition and platform; the failing statement and only the statements it depends on, taken from the IR's dependency graph; the run record; and the data's schema. Data is never included by default; the report is a file the user reads before attaching it anywhere. Zemp offers the same as "Report a problem" | 0.8.0 |
| Shrinking | Because runs are deterministic and bindings never change, the report builder repeatedly removes rows and statements while the failure persists, typically reducing millions of rows to a few dozen; it can then replace values with synthetic ones that keep the failure, so private data need never leave the machine | 0.8.0 |
| Issue templates | "Wrong result" (requires the reference output being compared), "crash or internal error", "unclear message" and "slow"; wrong results are always triaged first | 0.8.0 |
| Every fix becomes a test | The shrunk reproducer of each fixed bug joins the conformance suite | 0.8.0 |
| Known-issues list and `zeff audit` | Each release ships a list of known wrong-result bugs with the engine versions and the functions and options they affect. `zeff audit` checks the project's run records against it, for example: "your run of 12 October used `reg` with `weights:`, affected by issue 123, fixed in 0.9.2" | 0.9.0 |
| Patch releases | Wrong-result fixes ship as patch releases (0.9.1, 0.9.2), installed beside earlier engines. A project's pin is exact: it does not move to a patch release by itself, so its results stay reproducible, and wrong, until the user accepts "Update engine"; `zeff audit` and Zemp say this plainly when a pinned version is affected | 0.9.0 onwards |
| Launcher and engines | The `zeff` command becomes a small launcher that selects the engine a project pins from the engine store `~/.zeff/engines/`, installs a missing one after verifying its checksum and signature, and keeps every version side by side; `zeff engine use` is "Update engine" from the command line (section 2.1 of the Engine and Project Specification) | 0.9.0 |
| `zeff export replication` and `zeff verify` | The package is built from the command line as Zemp builds it; `zeff verify` re-runs it in a temporary folder with `--fresh --offline --no-external` and compares every result hash, output hash and warning with its run record, printing *identical* or what differs, exit code 0 or 1; `--engine` replays it on another engine (section 2.1 of the Engine and Project Specification) | 0.11.0 |

---

---

## 16. Glossary

| Term | Meaning |
| --- | --- |
| Binding | A name given to a value by `let`; never changes, may be shadowed |
| Chain | A sequence of `.method()` steps on one value |
| Column | One variable of a frame: an Arrow array of one type plus a validity bitmap for `NA` |
| Conformance self-test | `zeff selftest`: the conformance cases shipped with an engine, compared with their published checksums on the user's machine |
| Data root | A read-only folder declared in `zeff.toml` and read in place as `root:file` |
| Definition | `name = expr` inside `add`, `update`, `aggregate`, `rename`, `derive` or `frame`, creating a column or quantity |
| Dependence structure | The rule in the variance layer that weights each pair of observations' score products: clusters, two-way clusters, a spatial kernel, a network (section 8.3 of the Engine and Project Specification, point 2) |
| Design | A tier-3 package that uses fitted models to answer a research question; also the `design` research goal and a `SurveyDesign` value |
| Draw | One random number taken from a stream; numbered by statement, pass and position |
| Edition | The set of syntax rules a project was written under, named by year and fixed in `zeff.toml` |
| Engine | The Rust program that checks and runs Zeff, as WebAssembly or natively, pinned by version in every project |
| Engine store | `~/.zeff/engines/`, one folder per engine version, shared by the command line and Zemp desktop |
| Estimator | A model's implementation of the estimator trait (design, fit, scores, bread); the estimator interface is its Zeff face |
| Frame | An immutable table of columns with a schema; every frame method returns a new frame |
| Identity | The relation `sort`, `distinct`, `is_in` and grouping use, under which `NA` equals `NA` (section 3.7) |
| Kernel | Rust code every package uses and no script can call (section 8.1 of the Engine and Project Specification, rule 8) |
| Launcher | The small `zeff` command that selects, verifies and installs engines and hands each command to the engine a project pins (section 2.1 of the Engine and Project Specification) |
| Metadata | The key–value store every column and frame carries: label, value labels, missing codes, measure, unit, question, source, format, crs (section 4.3) |
| Module | One file in `functions/`, brought into a script with `use` |
| Opaque column | A column of a type the engine stores, hashes, prints and carries through row verbs but does not yet compute on (section 4.3) |
| Option | `name: value` in a call; one meaning per name everywhere (principle 8) |
| Package | A group of functions that ships with the engine, on a tier; also a third-party WebAssembly component after 1.0 |
| Pass | One iteration of a `for` loop or of `iterate`; passes get their own seeds and may run in parallel |
| Period | A reserved column type for a unit of time with a frequency (year, quarter, month, week, day; fiscal years), so that panels and series know what a gap is (section 4.3) |
| Project | A folder with `zeff.toml`, scripts, functions, data and run records; the only place Zeff code runs |
| Purpose tag | A research goal attached to a function, from which Zemp builds its menus |
| Reader | The engine's implementation of one file format behind `import`: sniff, schema, read, options (section 6.1) |
| Reducer | A statistic usable inside `add` and `aggregate` (`mean`, `sum`, `n`, …) as well as on a frame |
| Replication package | A zip of a project's `zeff.toml`, `zeff.lock`, scripts, functions, data or data-root checksums, run record and vendored packages, built by `zeff export replication` and checked by `zeff verify` |
| Result | A value returned by a model or test that implements the result and estimator interfaces |
| Run record | What each run stores in `.zeff/runs/`: seeds, warnings, file hashes, engine and package versions, outputs, recorded clock values |
| Scene | A plot as the engine returns it: marks in data coordinates with scales, guides and layout, in a versioned format drawn by Zemp and exported as SVG, PDF or PNG (section 7.6) |
| Schema | A data file's column names and types, cached in `.zeff/schemas/` for `zeff check` |
| Scratch project | The temporary project in which `zeff repl` and `zeff eval` run outside a project, as Zemp's Scratch project |
| Session script | `.zeff/session.zeff`, to which `zeff repl` appends each statement, so that a console session is itself a script |
| Source | Anything a project reads from outside its folder and data roots, under the four rules of section 8.6 of the Engine and Project Specification: enabled per project, credentials through `secret()`, recorded, replayed |
| Term | One column of a model matrix with a stable name (`educ`, `industry=Retail`, `educ:female`) |
| Tier | A layer of the package bundle; calls go down freely and sideways only in order |
| Verb | A frame method that returns a new frame (`where`, `add`, `merge`, `aggregate`, …) |
| Word | A bare name accepted by a parameter that lists its allowed values (`se: robust`) |
