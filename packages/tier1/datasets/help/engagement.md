# engagement

## Summary
Synthetic staff survey: six engagement statements and a question asked twice

## Description
One row per respondent of a staff survey answered by 1,200 of the 2,400 employees of `staff`,
in two waves, before and after that dataset's training course. `employee` is the employee's
number in `staff`, so the two merge on it. `eng1` to `eng6` are the answers of the second wave to
six statements about engagement at work, each from 1, "strongly disagree", to 5, "strongly
agree"; the fourth, "I often think of leaving", is worded in reverse, so that a high answer
means low engagement. `recommend_before` and `recommend_after` are the answers, no or yes, to
one question put in both waves: "would you recommend the firm as a place to work?". About
three answers in a hundred to the statements are missing, and two in a hundred to the question
of the second wave.

The data are generated, not collected, so what a scale built from the six statements should
show is known: once the fourth is scored the right way round they measure one thing, and
Cronbach's alpha of the scale in the population the model describes is 0.8469. With the fourth
left as written, it correlates negatively with the other five, which is what an analyst who
overlooks the reversed wording sees. The question asked twice gives paired answers: more
respondents moved from no to yes than from yes to no, and those who took the course more often
than those who did not.

## Generation
`generate/engagement.py` in the datasets package, run with a fixed seed (20261127), writes the
CSV, reading `staff.csv` for the employees' numbers and their `training`; the Arrow file with the
labels and the value labels is built from it by `cargo run -p xtask -- build-datasets`. The
respondents are drawn without replacement from the employees of `staff`. Each has an engagement
that is standard normal and is not written. The answer to statement j comes from
x = λ engagement + √(1 − λ²) noise, with standard normal noise and λ = 0.80, 0.75, 0.70, 0.75,
0.65 and 0.70 for the six: it is 1 plus the number of the cut points −1.5, −0.5, 0.5 and 1.5
that x exceeds, and the fourth answer is written as 6 less that.

The question is answered yes before the course with probability 1 / (1 + exp(−u)),
u = −0.3 + 0.9 engagement, and after it with u = −1.2 + 0.9 engagement + 0.8 training
+ 1.8 before, so that most keep their answer and the trained move towards yes. Each answer to a
statement is then left out with probability 0.03 and the second answer to the question with
probability 0.02, independently of everything else.

The alpha of 0.8469 is not an estimate from the file: the generating script works it out from
the model, by integrating the bivariate normal over the cells of each pair of answers, and
stops if its figure is another. The alpha of the 986 rows with all six answers is 0.8508.

## Licence and citation
CC0-1.0: the data are synthetic and dedicated to the public domain. Cite as "Zeff synthetic
engagement, version 0.10.0".

## See also
dataset, datasets, staff, merge, tabulate

## Examples
> let survey = dataset(engagement)
> survey.describe()
survey · 1,200 rows × 9 variables

  variable           type        non-missing   missing   example
  employee           Int               1,200         0   1
  eng1               Int               1,162        38   4
  eng2               Int               1,165        35   3
  eng3               Int               1,160        40   3
  eng4               Int               1,167        33   1
  eng5               Int               1,151        49   5
  eng6               Int               1,153        47   4
  recommend_before   Int (lab)         1,200         0   yes
  recommend_after    Int (lab)         1,177        23   yes
> survey.head(3)
frame · 3 rows × 9 variables

      employee   eng1   eng2   eng3   eng4   eng5   eng6   recommend_before   recommend_after
  1          1      4      3      3      1      5      4   yes                yes
  2          2      4      3      4      2      3      2   yes                yes
  3          3      2      3      2      3      3      4   yes                yes
> survey.tabulate([recommend_before, recommend_after])
tabulate · survey · recommend_before × recommend_after

  recommend_before    no   yes   NA   Total
  no                 465   183   15     663
  yes                114   415    8     537
  ─────────────────────────────────────────
  Total              579   598   23   1,200
> survey.correlation([eng1, eng4])
correlation · survey · pearson

           eng1     eng4
  eng1    1.000   -0.577
  eng4   -0.577    1.000

N per pair

          eng1    eng4
  eng1   1,162   1,129
  eng4   1,129   1,167
> survey.merge(dataset(staff), on: employee, join: inner, relation: one_to_one).aggregate(by: training, n = n(), recommends = mean(recommend_after))
frame · 2 rows × 3 variables

      training     n   recommends
  1   no         607     0.445378
  2   yes        593     0.572165
note: merge: 1,200 rows matched; 0 left rows unmatched; 1,200 right rows unmatched (line 6)
warning[W0201]: 23 rows with NA in recommend_after were skipped (line 6)
