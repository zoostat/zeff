# levels

## Summary
A category's levels, in order

## Description
The levels of a category column as a list of text, in the order the category has them: the order
`category` gave or `levels:` named, which is the order of the indicators in a model, with the
first level the base unless `base` chooses another. R has `levels`.

## Arguments
- var: A category column; inside a frame method, the variable's name.

## Value
A `List<Str>`.

## Missing values
`NA` is not a level and is not listed; a level with no rows is still a level and is listed.

## A common mistake
Asking for the levels of a text variable. Text has distinct values, not levels, until it is made a category:

> levels(staff["dept"])
error[Z0401]: `levels` needs a category, but this is a Str column
  = help: `category(x)` makes one; `describe` shows each variable's type

Make the category first: `levels(category(staff["dept"]))`.

## Coming from R or Stata
| levels(x) | levels(x) | levelsof x |

R's `levels` of a character vector is `NULL`, without a word; here that is an error. Stata's
`levelsof` lists the distinct values present; `levels` lists the levels the category has, with
or without rows.

## Related tasks
- To make a category, use `category`.
- To count a variable's distinct values, use `n_distinct`.

## See also
category, base, n_distinct

## Examples
// The levels of a category made from text, sorted.
> let staff = frame(name = ["Ann", "Bob", "Cy", "Dee"], dept = ["Sales", "IT", "Sales", "HR"], grade = [2, 1, 3, 2])
> levels(category(staff["dept"]))
HR
IT
Sales

// The levels of a category of the shipped wages, in the order the data gives them.
> levels(dataset(wages)["industry"])
Agriculture
Mining
Utilities
Construction
Manufacturing
Retail
Transport
Finance
Services
Education
Health
Public administration
