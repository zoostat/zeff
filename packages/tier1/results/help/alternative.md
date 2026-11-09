# alternative

## Summary
The alternative a test was computed for

## Description
Gives the alternative the p-value of a test's result was computed for, as the word of the
argument `alternative:`: `"two_sided"`, `"less"` or `"greater"`; a test that takes no
`alternative:` gives `"two_sided"`. Like `t$alternative` in R.

## Arguments
- t: A test's result.

## Value
A `Str`.

## See also
h0, h1, p_values

## Examples
> let t = dataset(wages).t_test(educ, group: female)
> t.alternative()
two_sided
> dataset(wages).t_test(educ, group: female, alternative: greater).alternative()
greater
