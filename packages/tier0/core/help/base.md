# base

## Summary
The reference level of a category

## Description
Moves one level of a category to the front and keeps the others in their order, so that it is the
base level: the level the indicators of a model are measured against, as R's `relevel` and Stata's
`ib#.` do. Inside a formula, `base(industry, "Retail")` names the base level of that term (section
7.1 of the Language Reference); on its own it returns the relevelled category, its order, contrast
and labels kept. A level the category does not have is an error that lists the levels and names the nearest one.

## Arguments
- var: A category column; inside a frame method or a formula, the variable's name.
- level: The level to put first, in quotes.

## Value
The category with `level` first.

## Missing values
A row with `NA` stays `NA`: only the order of the levels changes.

## A common mistake
Naming a level the category does not have, often a slip of spelling or of capitals. The message says where to find the levels:

> base(category(staff["dept"]), "Finance")
error[Z0216]: "Finance" is not a level of the category
  = help: write one of the levels, in quotes; `levels(x)` lists them

Write one of the levels as `levels` lists them: `base(category(staff["dept"]), "Sales")`.

## Coming from R or Stata
| base(x, "Retail") | relevel(x, ref = "Retail") | ib#.x, with # the code of the level |

In Stata the base is named by its number inside the model; here and in R by its label. Inside a
formula `base` is written where the variable stands, so the frame need not be changed first.

## Related tasks
- To see the levels and their order, use `levels`.

## See also
category, levels

## Examples
// One level moved to the front; the others keep their order.
> let staff = frame(name = ["Ann", "Bob", "Cy", "Dee"], dept = ["Sales", "IT", "Sales", "HR"], grade = [2, 1, 3, 2])
> let dept = category(staff["dept"])
> levels(base(dept, "Sales"))
Sales
HR
IT

// The category given is as it was.
> levels(dept)
HR
IT
Sales

// Inside a formula, on the shipped wages: every industry measured against retail.
> dataset(wages).reg(log(wage) ~ base(industry, "Retail"))
reg · log(wage) ~ base(industry, "Retail")
N = 18,201 (211 rows dropped for NA in model variables)
standard errors: classical

  term                             estimate   std.error   t value   p value   95% CI
  (intercept)                        3.1513      0.0090    349.30    <0.001   [3.1336, 3.1690]
  industry=Agriculture              -0.1474      0.0228     -6.47    <0.001   [-0.1921, -0.1028]
  industry=Mining                    0.0389      0.0352      1.10     0.270   [-0.0302, 0.1079]
  industry=Utilities                 0.0773      0.0267      2.90     0.004   [0.0250, 0.1296]
  industry=Construction              0.0926      0.0155      5.98    <0.001   [0.0622, 0.1229]
  industry=Manufacturing             0.0865      0.0131      6.59    <0.001   [0.0608, 0.1122]
  industry=Transport                 0.0242      0.0179      1.35     0.177   [-0.0109, 0.0594]
  industry=Finance                   0.1280      0.0152      8.43    <0.001   [0.0983, 0.1578]
  industry=Services                 -0.0547      0.0130     -4.20    <0.001   [-0.0803, -0.0292]
  industry=Education                 0.0467      0.0148      3.15     0.002   [0.0176, 0.0757]
  industry=Health                    0.0453      0.0139      3.26     0.001   [0.0181, 0.0725]
  industry=Public administration     0.1264      0.0157      8.07    <0.001   [0.0957, 0.1571]

R² 0.019 · adjusted R² 0.018
