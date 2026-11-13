; The highlight queries of Zeff (Engine and Project Specification, section 12, "Syntax
; highlighting"): keywords, formulas, model-block operators and bare option words each styled
; distinctly.
;
; Where several rules fit one name or sign, the last of them in this file is the one an
; editor takes. So the rule for any name and the rules for signs come first, and what a name
; or a sign is in particular (a function, a parameter, a type, the `~` of a formula, an
; option word) comes after them.

; ----- comments -----

(doc_comment) @comment.documentation
(pragma) @keyword.directive
(comment) @comment

; ----- literals -----

(number) @number
(string) @string
(raw_string) @string.special
(escape) @string.escape
(boolean) @boolean
(missing) @constant.builtin
(infinity) @constant.builtin
(quoted_name) @string.special.symbol

; ----- any other name -----

(identifier) @variable

; ----- operators and punctuation -----

["+" "-" "*" "/" "^" "!" "==" "!=" "<" "<=" ">" ">=" "&&" "||" ".." "=" "=>" "->" "?" "|"] @operator
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," "." "::" ":"] @punctuation.delimiter

; ----- keywords -----

["let" "use" "as"] @keyword
"fn" @keyword.function
"return" @keyword.return
["if" "else" "case"] @keyword.conditional
["for" "in" "break" "continue"] @keyword.repeat
["model" "dag"] @keyword.type
(catch_all) @keyword.conditional
(node_roles role: _ @keyword.modifier)

; ----- functions -----

(function_definition name: (identifier) @function)
(call function: (qualified_name (identifier) @function.call .))
(method_call function: (qualified_name (identifier) @function.method.call .))
; What stands before `::` is a package or a module.
(call function: (qualified_name (identifier) @module (identifier)))
(method_call function: (qualified_name (identifier) @module (identifier)))
(use_declaration path: (qualified_name (identifier) @module))

; ----- parameters, options, definitions and fields -----

(parameter name: (identifier) @variable.parameter)
(function_literal parameter: (identifier) @variable.parameter)
(named_argument name: (identifier) @property)
(field name: (identifier) @property)
(definition name: (identifier) @variable.member)
(field_access field: (identifier) @variable.member)
(for_loop variable: (identifier) @variable)
(let_statement name: (identifier) @variable)

; ----- types -----

(type_name (qualified_name (identifier) @type))
(generic_type (qualified_name (identifier) @type))
(record_type (identifier) @property)
"Fn" @type.builtin

; ----- formulas -----

; `~` makes a formula; `:` is an interaction of its terms; `.` alone is every other variable.
(formula operator: "~" @operator.formula)
(interaction ":" @operator.formula)
(every_other "." @variable.builtin)

; ----- model blocks and causal graphs -----

; The measurement `=~`, the covariance `~~`, a defined parameter `:=` and a label `as`.
(formula operator: ["=~" "~~"] @operator.model)
(defined_parameter ":=" @operator.model)
(defined_parameter name: (identifier) @variable.parameter)
(label_expression label: (identifier) @label)
(path edge: _ @operator.model)

; ----- bare option words -----

; A bare word that is the whole value of a named argument or of a record's field, or an
; item of a list that is, and that some function takes as one of a fixed list
; (`se: robust`). The words are the registry's: `cargo run -p xtask -- grammar-words` writes
; the lines between the two marks, and `check-grammar` fails when they are not current.
; BEGIN WORDS
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "alternative") (#any-of? @constant.option "greater" "less" "two_sided"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "alternative") (#any-of? @constant.option "greater" "less" "two_sided"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "bins") (#any-of? @constant.option "fd" "scott" "sturges"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "bins") (#any-of? @constant.option "fd" "scott" "sturges"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "ci") (#any-of? @constant.option "exact" "wald" "wilson"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "ci") (#any-of? @constant.option "exact" "wald" "wilson"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "closed") (#any-of? @constant.option "left" "right"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "closed") (#any-of? @constant.option "left" "right"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "color") (#any-of? @constant.option "okabe_ito" "viridis"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "color") (#any-of? @constant.option "okabe_ito" "viridis"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "contrast") (#any-of? @constant.option "helmert" "poly" "sum" "treatment"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "contrast") (#any-of? @constant.option "helmert" "poly" "sum" "treatment"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "direction") (#any-of? @constant.option "hv" "mid" "vh"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "direction") (#any-of? @constant.option "hv" "mid" "vh"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "effect") (#any-of? @constant.option "d" "epsilon" "eta" "g" "omega" "omega_f"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "effect") (#any-of? @constant.option "d" "epsilon" "eta" "g" "omega" "omega_f"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "fill") (#any-of? @constant.option "okabe_ito" "viridis"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "fill") (#any-of? @constant.option "okabe_ito" "viridis"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "fit") (#any-of? @constant.option "linear" "none"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "fit") (#any-of? @constant.option "linear" "none"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "join") (#any-of? @constant.option "full" "inner" "left"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "join") (#any-of? @constant.option "full" "inner" "left"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "kernel") (#any-of? @constant.option "biweight" "cosine" "epanechnikov" "gaussian" "rectangular" "triangular"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "kernel") (#any-of? @constant.option "biweight" "cosine" "epanechnikov" "gaussian" "rectangular" "triangular"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "method") (#any-of? @constant.option "bh" "bonferroni" "by" "hochberg" "holm" "hommel" "kendall" "none" "pearson" "scheffe" "sidak" "spearman" "tukey"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "method") (#any-of? @constant.option "bh" "bonferroni" "by" "hochberg" "holm" "hommel" "kendall" "none" "pearson" "scheffe" "sidak" "spearman" "tukey"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "missing") (#any-of? @constant.option "listwise" "pairwise"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "missing") (#any-of? @constant.option "listwise" "pairwise"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "mode") (#any-of? @constant.option "away" "even"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "mode") (#any-of? @constant.option "away" "even"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "name") (#any-of? @constant.option "journal" "minimal" "slide"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "name") (#any-of? @constant.option "journal" "minimal" "slide"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "names") (#any-of? @constant.option "keep" "snake"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "names") (#any-of? @constant.option "keep" "snake"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "percent") (#any-of? @constant.option "col" "row" "total"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "percent") (#any-of? @constant.option "col" "row" "total"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "position") (#any-of? @constant.option "bottom" "direct" "dodge" "fill" "none" "overlay" "right" "stack" "top"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "position") (#any-of? @constant.option "bottom" "direct" "dodge" "fill" "none" "overlay" "right" "stack" "top"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "relation") (#any-of? @constant.option "many_to_one" "one_to_many" "one_to_one"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "relation") (#any-of? @constant.option "many_to_one" "one_to_many" "one_to_one"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "scale") (#any-of? @constant.option "count" "density" "fraction" "percent"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "scale") (#any-of? @constant.option "count" "density" "fraction" "percent"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "scales") (#any-of? @constant.option "fixed" "free" "free_x" "free_y"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "scales") (#any-of? @constant.option "fixed" "free" "free_x" "free_y"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "se") (#any-of? @constant.option "classical" "hc2" "hc3" "robust"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "se") (#any-of? @constant.option "classical" "hc2" "hc3" "robust"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "sides") (#any-of? @constant.option "both" "x" "y"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "sides") (#any-of? @constant.option "both" "x" "y"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "size") (#any-of? @constant.option "one_column" "slide" "two_column"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "size") (#any-of? @constant.option "one_column" "slide" "two_column"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "stats") (#any-of? @constant.option "aic" "bic" "max" "mean" "min" "missing" "n" "n_obs" "r_squared" "r_squared_within" "sd"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "stats") (#any-of? @constant.option "aic" "bic" "max" "mean" "min" "missing" "n" "n_obs" "r_squared" "r_squared_within" "sd"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "style") (#any-of? @constant.option "dash_dot" "dash_dot_dot" "dashed" "dotted" "long_dash" "solid"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "style") (#any-of? @constant.option "dash_dot" "dash_dot_dot" "dashed" "dotted" "long_dash" "solid"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "ties") (#any-of? @constant.option "average" "first" "max" "min"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "ties") (#any-of? @constant.option "average" "first" "max" "min"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "type") (#any-of? @constant.option "cartesian" "date" "fixed" "flip" "linear" "log" "polar" "sqrt"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "type") (#any-of? @constant.option "cartesian" "date" "fixed" "flip" "linear" "log" "polar" "sqrt"))
((named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "weight_type") (#any-of? @constant.option "analytic" "frequency" "sampling"))
((field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))
  (#eq? @property "weight_type") (#any-of? @constant.option "analytic" "frequency" "sampling"))
((named_argument value: (list (qualified_name . (identifier) @constant.option .)))
  (#any-of? @constant.option "aic" "bic" "max" "mean" "min" "missing" "n" "n_obs" "r_squared" "r_squared_within" "sd"))
((field value: (list (qualified_name . (identifier) @constant.option .)))
  (#any-of? @constant.option "aic" "bic" "max" "mean" "min" "missing" "n" "n_obs" "r_squared" "r_squared_within" "sd"))
; END WORDS
