# influence

## Summary
Leverage, Cook's distance and studentized residuals by observation

## Description
Gives, for each row of the frame a regression was fitted on, its leverage (the diagonal of the
hat matrix), Cook's distance (how far the estimates move when the row is left out, scaled) and
its externally studentized residual (the residual scaled by the error variance estimated without
the row), as a frame of the fitted frame's full length with `NA` in the rows the fit dropped, so
that `wages.add(h = m.influence()["leverage"])` lines up. A row of leverage one has no distance
and no studentized residual. Like `influence.measures` in R and `predict, cooksd` and
`predict, rstudent` in Stata; the warning W1006 names the rows whose removal moves an estimate by
more than its standard error. With one effect in `fe:` the measures are those of the fit with the
effect's indicators written out, the levels counted among the parameters; a fit with several
effects is refused (Z0304), its leverage in the full design waiting for release 1.1.

## Arguments
- fit: The regression fit.

## Value
A `Frame` with the variables `leverage`, `cooks_distance` and `studentized`.

## Details
The leverage comes from the fit's R factor in one pass over the rows, h = x′(XᵀX)⁻¹x; Cook's
distance is (e / (σ(1 − h)))² h / k, with σ² the residual sum of squares over N − k and k the
terms with an estimate; the externally studentized residual is e / (σ₍ᵢ₎ √(1 − h)) with
σ₍ᵢ₎² = (RSS − e² / (1 − h)) / (N − k − 1), the error variance without the row, as R's
`hatvalues`, `cooks.distance` and `rstudent` give them. Under analytic and sampling weights the
residual is weighted by √w, as R's `weighted.residuals` is; under frequency weights a row stands
for w observations, each with leverage h / w and the row's residual, which is what R computes on
the replicated rows. With one effect in `fe:` the leverage is that in the demeaned columns plus
the row's share of its level's weight, 1/n of the level without weights, which is the leverage
with the effect's indicators written out; k then counts the levels too.

## References
Cook, R. D. (1977). Detection of influential observation in linear regression. *Technometrics*,
19, 15–18. Belsley, D. A., Kuh, E. and Welsch, R. E. (1980). *Regression Diagnostics*. Wiley,
chapter 2.

## See also
residuals, fitted, r_squared

## Examples
> let wages = dataset(wages)
> let m = wages.reg(log(wage) ~ educ + exper + female)
> m.influence().summary([leverage, cooks_distance, studentized])
summary · frame

  variable              N   missing    mean      sd      min     max
  leverage         18,179       233   0.000   0.000    0.000   0.001
  cooks_distance   18,179       233   0.000   0.000    0.000   0.002
  studentized      18,179       233   0.000   1.000   -4.639   3.537
warning[W0201]: 233 rows with NA in leverage, 233 with NA in cooks_distance and 233 with NA in studentized were skipped (line 3)
