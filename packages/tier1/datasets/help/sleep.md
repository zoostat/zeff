# sleep

## Summary
Student's 1908 data on the extra sleep given by two drugs

## Description
The extra hours of sleep, against a control night, of ten patients given each of two soporific
drugs, the data William Sealy Gosset ("Student") used in 1908 to introduce the t test. Twenty
rows: the extra sleep in hours, the drug (1 or 2, labelled with the drugs' names) and the patient
(1 to 10, so the two groups are paired). R ships the same values as `sleep`.

## Source
Student (1908), table on page 20, who took the figures from Cushny and Peebles (1905). The file is
written from the published values by `generate/sleep.py` in the datasets package, so the derivation
is the file itself.

## Licence and citation
Public domain: a scientific table published in 1908 by an author who died in 1937. Cite Student
(1908) when the data are used.

## References
Student (1908). The probable error of a mean. Biometrika 6, 1–25.
Cushny, A. R. and Peebles, A. R. (1905). The action of optical isomers II: hyoscines. Journal of Physiology 32, 501–510.

## See also
dataset, datasets, wages

## Examples
> let sleep = dataset(sleep)
> sleep.head(3)
frame · 3 rows × 3 variables

      extra   group                id
  1     0.7   dextro-hyoscyamine    1
  2    -1.6   dextro-hyoscyamine    2
  3    -0.2   dextro-hyoscyamine    3
> sleep.mean(extra, by: group)
mean · extra by group

  group                 mean
  dextro-hyoscyamine   0.750
  laevo-hyoscyamine    2.330
