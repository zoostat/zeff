// The tree-sitter grammar of Zeff (Engine and Project Specification, section 12, "Syntax
// highlighting"): the grammar of section 3.4 of the Language Reference as tree-sitter reads
// one, for editors to colour and fold a script by. The engine's own parser decides what a
// script means; this grammar is held to it by a test on every script of the repository.
//
// A line break ends a statement at the top level and in a block, and means nothing inside
// round or square brackets, a record or `case { }`, after an operator or a comma, and before a
// line that begins with `.name` (section 3.1, "Statements"). The scanner of `src/scanner.c`
// gives the line break as a token only where the grammar asks for one and the next line is
// no continuation.

const PREC = {
  formula: 1,
  or: 2,
  and: 3,
  compare: 4,
  range: 5,
  add: 6,
  label: 7,
  multiply: 8,
  interact: 9,
  unary: 10,
  power: 11,
  postfix: 12,
};

// The reserved words (section 3.1): each may name an argument or a field, since it then
// stands before a colon.
const RESERVED = [
  'let', 'fn', 'if', 'else', 'case', 'for', 'in', 'break', 'continue', 'return', 'use',
  'model', 'dag', 'as', 'true', 'false', 'NA', 'Inf',
  'type', 'while', 'match', 'mut', 'pub', 'loop', 'trait', 'impl',
];

const commaSep1 = (rule) => seq(rule, repeat(seq(',', rule)), optional(','));
const commaSep = (rule) => optional(commaSep1(rule));

module.exports = grammar({
  name: 'zeff',

  externals: ($) => [$._newline],

  extras: ($) => [/[ \t\r\n]/, $.doc_comment, $.pragma, $.comment],

  word: ($) => $.identifier,

  conflicts: ($) => [
    // `=> {` opens a block or a record: what follows the brace tells.
    [$.record, $.block],
  ],

  rules: {
    script: ($) =>
      repeat(seq(choice($._statement, $.function_definition, $.use_declaration), $._newline)),

    use_declaration: ($) => seq('use', field('path', $.qualified_name)),

    _statement: ($) =>
      choice(
        $.let_statement,
        $.for_loop,
        $.return_statement,
        $.break_statement,
        $.continue_statement,
        $._expression,
      ),

    let_statement: ($) =>
      seq('let', field('name', $._name), '=', field('value', $._expression)),

    for_loop: ($) =>
      seq(
        'for',
        field('variable', $._name),
        'in',
        field('items', $._expression),
        field('body', $.block),
      ),

    return_statement: ($) => prec.right(seq('return', optional($._expression))),
    break_statement: (_) => 'break',
    continue_statement: (_) => 'continue',

    function_definition: ($) =>
      seq(
        'fn',
        field('name', $._name),
        field('parameters', $.parameters),
        '->',
        field('returns', $._type),
        field('body', $.block),
      ),

    parameters: ($) => seq('(', commaSep($.parameter), ')'),

    parameter: ($) =>
      seq(
        field('name', $._name),
        ':',
        field('type', $._type),
        optional(seq('=', field('default', $._expression))),
      ),

    // Types (section 3.4): a name with its arguments, a record of fields, a function, an
    // optional, and alternatives joined by `|`, which are a fixed list of words or, in a
    // built-in's file, several types.
    _type: ($) => choice($.union_type, $._single_type),
    union_type: ($) => prec.left(1, seq($._single_type, repeat1(seq('|', $._single_type)))),
    _single_type: ($) =>
      choice($.optional_type, $.generic_type, $.record_type, $.function_type, $.type_name),
    optional_type: ($) => seq(choice($.generic_type, $.record_type, $.type_name), '?'),
    type_name: ($) => $.qualified_name,
    generic_type: ($) => seq($.qualified_name, '<', commaSep1($._type), '>'),
    record_type: ($) => seq('{', commaSep1(seq($._name, ':', $._type)), '}'),
    function_type: ($) =>
      prec.right(
        seq(
          'Fn',
          '(',
          optional(seq($._type, repeat(seq(',', $._type)), optional(seq(',', '..')))),
          ')',
          '->',
          $._type,
        ),
      ),

    // A block: statements, each ended by a line break; the last may stand before the brace
    // on its line, `{ x / 2 }`.
    block: ($) => seq('{', repeat(seq($._statement, $._newline)), optional($._statement), '}'),

    _expression: ($) =>
      choice(
        $.formula,
        $.binary_expression,
        $.range_expression,
        $.label_expression,
        $.interaction,
        $.unary_expression,
        $.power_expression,
        $.method_call,
        $.field_access,
        $.index_expression,
        $._primary,
      ),

    // `y ~ x`, the measurement `=~` and the covariance `~~` of a model block, and the
    // one-sided `~ z`.
    formula: ($) =>
      choice(
        prec.left(
          PREC.formula,
          seq(
            field('left', $._expression),
            field('operator', choice('~', '=~', '~~')),
            field('right', $._expression),
          ),
        ),
        prec.left(PREC.formula, seq(field('operator', '~'), field('right', $._expression))),
      ),

    binary_expression: ($) => {
      const table = [
        ['||', PREC.or],
        ['&&', PREC.and],
        ['==', PREC.compare],
        ['!=', PREC.compare],
        ['<', PREC.compare],
        ['<=', PREC.compare],
        ['>', PREC.compare],
        ['>=', PREC.compare],
        ['+', PREC.add],
        ['-', PREC.add],
        ['*', PREC.multiply],
        ['/', PREC.multiply],
      ];
      return choice(
        ...table.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field('left', $._expression),
              field('operator', operator),
              field('right', $._expression),
            ),
          ),
        ),
      );
    },

    range_expression: ($) =>
      prec.left(PREC.range, seq(field('from', $._expression), '..', field('to', $._expression))),

    // `expr as name`, a label in a model block.
    label_expression: ($) =>
      prec.left(PREC.label, seq($._expression, 'as', field('label', $._name))),

    // `a:b`, an interaction of a formula.
    interaction: ($) =>
      prec.left(PREC.interact, seq(field('left', $._expression), ':', field('right', $._expression))),

    unary_expression: ($) =>
      prec(PREC.unary, seq(field('operator', choice('-', '!')), field('operand', $._expression))),

    power_expression: ($) =>
      prec.right(
        PREC.power,
        seq(field('base', $._expression), '^', field('exponent', $._expression)),
      ),

    // `x.f(a)`, and with its package `x.survival::coxph(a)`.
    method_call: ($) =>
      prec(
        PREC.postfix,
        seq(
          field('value', $._expression),
          '.',
          field('function', $.qualified_name),
          field('arguments', $.arguments),
        ),
      ),

    // `r.name`, a field of a record; `dist.normal` before its brackets reads the same.
    field_access: ($) =>
      prec(PREC.postfix, seq(field('value', $._expression), '.', field('field', $._name))),

    index_expression: ($) =>
      prec(
        PREC.postfix,
        seq(field('value', $._expression), '[', field('index', $._expression), ']'),
      ),

    _primary: ($) =>
      choice(
        $.number,
        $.string,
        $.raw_string,
        $.boolean,
        $.missing,
        $.infinity,
        $.list,
        $.mapping,
        $.record,
        $.call,
        $.qualified_name,
        $.parenthesized,
        $.if_expression,
        $.case_expression,
        $.model_block,
        $.dag_block,
        $.list_for,
        $.function_literal,
        $.every_other,
      ),

    parenthesized: ($) => seq('(', $._expression, ')'),

    call: ($) =>
      prec(
        PREC.postfix,
        seq(field('function', $.qualified_name), field('arguments', $.arguments)),
      ),

    arguments: ($) => seq('(', commaSep($._argument), ')'),

    _argument: ($) => choice($.named_argument, $.definition, $.spread, $._expression),

    // `name: value`, an option; the name may be a reserved word.
    named_argument: ($) =>
      prec(
        PREC.postfix + 1,
        seq(
          field('name', choice($._name, $._reserved_name)),
          ':',
          field('value', $._expression),
        ),
      ),

    // `name = expr`, a definition (`add`, `update`, `aggregate`, `rename`, `derive`, `frame`).
    definition: ($) => seq(field('name', $._name), '=', field('value', $._expression)),

    // `..record`, the fields of a record as arguments or fields.
    spread: ($) => seq('..', $._expression),

    // A reserved word where a name stands before a colon.
    _reserved_name: ($) => prec(1, alias(choice(...RESERVED), $.identifier)),

    list: ($) => seq('[', commaSep($._expression), ']'),

    // `[key => value, ...]`.
    mapping: ($) => seq('[', commaSep1($.pair), ']'),
    pair: ($) => seq(field('key', $._expression), '=>', field('value', $._expression)),

    // `{name: value, ..more}`.
    // After `=>` a brace opens a record where `}`, `..` or `name:` follows it, and a block
    // otherwise, as the engine's lexer has it: of the two readings the record is taken.
    record: ($) => prec.dynamic(1, seq('{', commaSep(choice($.field, $.spread)), '}')),
    // `name:` after a brace is a field, as `name:` at the start of an argument is an option:
    // the colon is taken before the name is read as a value.
    field: ($) =>
      prec(
        PREC.postfix + 1,
        seq(
          field('name', choice($._name, $._reserved_name)),
          ':',
          field('value', $._expression),
        ),
      ),

    if_expression: ($) =>
      prec.right(
        seq(
          'if',
          field('condition', $._expression),
          field('then', $.block),
          optional(seq('else', field('else', choice($.if_expression, $.block)))),
        ),
      ),

    // `case { condition => value, _ => value }`: line breaks inside mean nothing.
    case_expression: ($) => seq('case', '{', commaSep1($.arm), '}'),
    arm: ($) =>
      seq(
        field('condition', choice($._expression, $.catch_all)),
        '=>',
        field('value', choice($._expression, $.block)),
      ),
    catch_all: (_) => '_',

    // `[for x in xs { body }]`, a list built by a loop.
    list_for: ($) => seq('[', $.for_loop, ']'),

    // `fn(x) x / 2`, a function where a parameter takes one.
    function_literal: ($) =>
      prec.right(
        seq(
          'fn',
          '(',
          commaSep(seq(field('parameter', $._name), optional(seq(':', $._type)))),
          ')',
          field('body', $._expression),
        ),
      ),

    // `.` alone on the right of `~`: every other variable (section 7.5).
    every_other: (_) => '.',

    // A model block (section 7.4): formulas, defined parameters and `efa(...)` lines.
    model_block: ($) =>
      seq('model', '{', repeat(seq($._model_line, $._newline)), optional($._model_line), '}'),
    _model_line: ($) => choice($._expression, $.defined_parameter),
    defined_parameter: ($) => seq(field('name', $._name), ':=', field('value', $._expression)),

    // A causal graph (section 7.3): paths of nodes, and the latent and selected nodes.
    dag_block: ($) =>
      seq('dag', '{', repeat(seq($._dag_line, $._newline)), optional($._dag_line), '}'),
    _dag_line: ($) => choice($.path, $.node_roles),
    path: ($) => seq($._node, repeat(seq(field('edge', choice('->', '<->', '--')), $._node))),
    _node: ($) => choice($._name, $.node_set),
    node_set: ($) => seq('[', commaSep1($._name), ']'),
    node_roles: ($) =>
      seq(field('role', choice('latent', 'selected')), $._name, repeat(seq(',', $._name))),

    qualified_name: ($) => prec.left(seq($._name, repeat(seq('::', $._name)))),

    _name: ($) => choice($.identifier, $.quoted_name),
    identifier: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
    // A name that is no valid name, between backticks (section 3.6).
    quoted_name: (_) => /`[^`\n]*`/,

    number: (_) => {
      const int = /[0-9]+(_[0-9]+)*/;
      const exponent = /[eE][+-]?[0-9]+(_[0-9]+)*/;
      return token(choice(seq(int, '.', int, optional(exponent)), seq(int, exponent), int));
    },
    string: ($) => seq('"', repeat(choice($.escape, $._string_text)), '"'),
    _string_text: (_) => token.immediate(prec(1, /[^"\\\n]+/)),
    escape: (_) => token.immediate(/\\./),
    raw_string: (_) => /r"[^"\n]*"/,
    boolean: (_) => choice('true', 'false'),
    missing: (_) => 'NA',
    infinity: (_) => 'Inf',

    // `///`, the help of the function below it; `// zeff:`, a pragma; `//`, a comment.
    doc_comment: (_) => token(prec(3, /\/\/\/[^\n]*/)),
    pragma: (_) => token(prec(2, /\/\/[ \t]*zeff:[^\n]*/)),
    comment: (_) => token(prec(1, /\/\/[^\n]*/)),
  },
});
