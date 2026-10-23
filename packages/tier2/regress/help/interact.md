# interact

## Summary
Every interaction of some variables up to an order, inside a formula

## Description
A term of a formula that stands for the variables listed and every interaction between them up
to `order:`: `interact([a, b, c], order: 2)` is `a + b + c + a:b + a:c + b:c`, which R writes
`(a + b + c)^2` and Stata `a##b##c`. With `order: 3` the three-way interaction `a:b:c` joins.
A category among the variables is coded by its contrast in each interaction, as section 7.2 of
the Language Reference says; a term the formula already has is kept once, with a note. The model
matrix reads this term when a model is fitted; `interact` has no value of its own, so outside a
formula it is an error that says where it belongs.

## Arguments
- vars: The variables, two or more, in brackets.
- order: The highest order of interaction, from 2 to the number of variables.

## Value
A term of a formula, read by the model matrix; no value outside one.

## See also
category, base

## Examples
> let f = y ~ interact([a, b, c], order: 2)
> f
y ~ interact([a, b, c], order: 2)
