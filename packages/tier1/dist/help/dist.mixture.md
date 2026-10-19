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

## See also
dist.truncated, dist.normal

## References
- McLachlan, G. J. and Peel, D. (2000). *Finite Mixture Models*, chapter 1. Wiley.
- Frühwirth-Schnatter, S. (2006). *Finite Mixture and Markov Switching Models*, chapter 1.
  Springer.

## Examples
> let m = dist.mixture([dist.normal(), dist.normal(mean: 3, sd: 0.5)], probs: [0.3, 0.7])
> m
mixture · 0.3 × (normal distribution · mean: 0, sd: 1) + 0.7 × (normal distribution · mean: 3, sd: 0.5)
> m.pdf(3)
0.559849
> m.cdf(1.5)
0.280903
> m.quantile(0.3)
1.88354
> m.draw(3, seed: 5)
Column<Float> · 3 values
4.06894
3.09384
2.71414
