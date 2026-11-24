# dist.mixture

## Summary
A finite mixture of distributions with mixing probabilities probs:

## Description
A `Distribution` value for a finite mixture: a draw comes from the first component with
probability `probs:[1]`, from the second with `probs:[2]`, and so on. Mixtures describe populations
made of groups (two kinds of worker, a spike at zero and a continuous part) and are the model behind
latent-class and finite-mixture estimation. The components may be any distributions, discrete ones
and other mixtures included; when every component is discrete, the mixture is and its draws are
Ints. The value prints its components with their probabilities. R has no mixture distribution in
base; the `distr` and `mixtools` packages build one.

## Arguments
- parts: A list of distributions, at least one, as in `[dist.normal(), dist.normal(mean: 3)]`.
- probs: A list of mixing probabilities, one per distribution, each 0 or more, summing to one. A
  list of another length is refused (Z0401); probabilities that do not sum to one are refused
  (Z0413); a missing `probs:` is refused (Z0406).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The density is Σ pᵢ fᵢ(x) and the distribution function Σ pᵢ Fᵢ(x), its upper tail the same sum of
the components' upper tails, so that both keep their digits. The quantile is the smallest x with the
mixed distribution function at or above p, found by halving a bracket set by the components' own
quantiles; for a discrete mixture it lands on a whole number. A draw spends two uniform numbers of
the stream: the first chooses the component, the second is inverted through it (Engine and Project
Specification, section 8.3, point 3). The probabilities are normalised by their sum, so a sum off
by rounding is accepted within 1e-9.

## A common mistake
Giving probabilities that do not add up to one:

> dist.mixture([dist.normal(), dist.normal(mean: 3)], probs: [0.3, 0.3])
error[Z0413]: `probs` of `dist.mixture` must be probabilities of 0 or more that sum to one
  = help: `zeff help dist.mixture` gives the domain of every parameter

The probabilities are the shares of the components and sum to one: `probs: [0.3, 0.7]`.

## Coming from R or Stata
| dist.mixture([dist.normal(), dist.normal(mean: 3)], probs: [0.3, 0.7]).cdf(x) | 0.3 * pnorm(x) + 0.7 * pnorm(x, 3) | 0.3 * normal(x) + 0.7 * normal(x - 3) |
| dist.mixture([dist.normal(), dist.normal(mean: 3)], probs: [0.3, 0.7]).draw(n) | ifelse(runif(n) < 0.3, rnorm(n), rnorm(n, 3)) | cond(runiform() < 0.3, rnormal(), rnormal(3, 1)) |

Neither R's base nor Stata has a mixture as a distribution: the density and the distribution
function are weighted sums written out, and the quantile has to be found by a search. Here the
mixture is a distribution like any other, and `quantile` does the search.

## Related tasks
- For the density, a probability, a quantile or draws of it, use its methods, as for any distribution: `pdf`, `cdf`, `quantile` and `draw`.
- To keep a part of one distribution, use `dist.truncated`.

## References
- McLachlan, G. J. and Peel, D. (2000). *Finite Mixture Models*, chapter 1. Wiley.
- Frühwirth-Schnatter, S. (2006). *Finite Mixture and Markov Switching Models*, chapter 1.
  Springer.

## See also
dist.truncated, dist.normal

## Examples
// Three in ten from a standard normal, seven in ten from a narrower one centred at 3.
> let m = dist.mixture([dist.normal(), dist.normal(mean: 3, sd: 0.5)], probs: [0.3, 0.7])
> m
mixture · 0.3 × (normal distribution · mean: 0, sd: 1) + 0.7 × (normal distribution · mean: 3, sd: 0.5)

// The density at the second centre, the probability below 1.5, and the quantile at 0.3.
> m.pdf(3)
> m.cdf(1.5)
> m.quantile(0.3)
0.559849
0.280903
1.88354

// Three draws.
> m.draw(3, seed: 5)
Column<Float> · 3 values
4.06894
3.09384
2.71414

// A spike at zero beside a count: one period in five has no event for a reason of its own.
> let zi = dist.mixture([dist.binomial(n: 1, p: 0), dist.poisson(lambda: 3)], probs: [0.2, 0.8])
> zi.pdf(0)
0.23983
