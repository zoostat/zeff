//! Fixed effects projected out of a design (Language Reference, section 7.2, "Fixed effects",
//! "Rows" and "Collinearity"; Engine and Project Specification, section 8.3, point 10; release
//! 0.6.0, step 18): the groups of each effect over the rows a design uses, the kernel of step 17
//! called on the design's columns and outcome, and the transformed design the estimator fits,
//! with the rows the effects' singletons removed, the columns the effects absorbed zeroed and
//! named, and the effects' own estimates recovered afterwards for `predict`.

use crate::design::{Dense, Design};
use crate::estimator::Fitted;
use std::collections::BTreeMap;
use std::rc::Rc;
use zeff_columns::{Column, Key};
use zeff_kernels::demean::{Effect, Options, demean};

/// The share of a regressor's original norm below which the fixed effects absorb it (section
/// 7.2, "Collinearity": 1e-8).
pub const ABSORBED_SHARE: f64 = 1e-8;
/// The share of a regressor's original norm below which little variation is left after the
/// fixed effects (W1004): one per cent.
pub const LITTLE_VARIATION_SHARE: f64 = 1e-2;

/// One fixed effect's groups over the rows of a design.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectGroups {
    /// The effect as written: `firm`, `industry:year`.
    pub name: String,
    /// The variables it groups by.
    pub variables: Vec<String>,
    /// The key of each level, in level order (section 3.7, "Order"; `NA` never among them, its
    /// rows being dropped before).
    pub keys: Vec<Vec<Key>>,
    /// The group of each row used, as a position in `keys`.
    pub group_of_row: Vec<usize>,
}

/// The key of a row for a fixed effect: the identity of section 3.7, with a category's level
/// as its label, so that a fit's levels match new data by label whatever the new column's level
/// order.
pub fn effect_key(column: &Column, row: usize) -> Key {
    match Key::of(column, row) {
        Some(Key::Level(code)) => Key::Text(
            column
                .levels()
                .and_then(|levels| levels.get(code as usize).map(|l| (*l).to_string()))
                .unwrap_or_default(),
        ),
        Some(key) => key,
        None => Key::Na,
    }
}

impl EffectGroups {
    /// The groups of the rows `rows` under the columns of one effect, in key order.
    pub fn over(
        name: &str,
        variables: &[String],
        columns: &[&Column],
        rows: &[usize],
    ) -> EffectGroups {
        let row_keys: Vec<Vec<Key>> = rows
            .iter()
            .map(|&row| columns.iter().map(|c| effect_key(c, row)).collect())
            .collect();
        let mut by_key: BTreeMap<&[Key], usize> = BTreeMap::new();
        for key in &row_keys {
            by_key.entry(key.as_slice()).or_insert(0);
        }
        for (g, slot) in by_key.values_mut().enumerate() {
            *slot = g;
        }
        let keys: Vec<Vec<Key>> = by_key.keys().map(|k| k.to_vec()).collect();
        let group_of_row = row_keys.iter().map(|k| by_key[k.as_slice()]).collect();
        EffectGroups {
            name: name.to_string(),
            variables: variables.to_vec(),
            keys,
            group_of_row,
        }
    }

    pub fn levels(&self) -> usize {
        self.keys.len()
    }

    /// The same effect over the rows at `kept` (positions among its rows), the levels no kept
    /// row has left out.
    fn restricted(&self, kept: &[usize]) -> EffectGroups {
        let mut present = vec![false; self.keys.len()];
        for &p in kept {
            present[self.group_of_row[p]] = true;
        }
        let mut renumbered = vec![usize::MAX; self.keys.len()];
        let mut keys = Vec::new();
        for (g, key) in self.keys.iter().enumerate() {
            if present[g] {
                renumbered[g] = keys.len();
                keys.push(key.clone());
            }
        }
        EffectGroups {
            name: self.name.clone(),
            variables: self.variables.clone(),
            keys,
            group_of_row: kept
                .iter()
                .map(|&p| renumbered[self.group_of_row[p]])
                .collect(),
        }
    }
}

/// What projecting the effects out found.
#[derive(Debug, Clone, PartialEq)]
pub struct Absorbed {
    /// The effects over the rows of the transformed design, in its order, the levels of the
    /// dropped singletons left out.
    pub effects: Vec<EffectGroups>,
    /// The position of each row of the transformed design among the rows of the original.
    pub kept: Vec<usize>,
    /// The rows dropped as singletons, over every round.
    pub singletons: usize,
    pub passes: usize,
    pub converged: bool,
    pub largest_change: f64,
    /// The columns the effects absorbed (a remaining norm below [`ABSORBED_SHARE`] of the
    /// original), zeroed in the transformed design so that the fit drops them.
    pub absorbed_columns: Vec<usize>,
    /// The columns keeping less than [`LITTLE_VARIATION_SHARE`] of their norm, with the share
    /// kept.
    pub little_variation: Vec<(usize, f64)>,
    /// Every demeaned column by its name (the outcome under `~` and its text), the ones taken
    /// from `known` included, for the next design over the same effects and rows.
    pub demeaned: BTreeMap<String, Rc<Vec<f64>>>,
}

impl Design {
    /// The design with the effects projected out of its columns and outcome: the kernel of step
    /// 17 on the unweighted values with the design's weights, the rows it kept in its order
    /// (sorted once by the largest effect), the columns and the outcome held as values, no
    /// intercept column, and the original outcome kept beside. `effects` are the effects'
    /// groups over this design's rows; `known` holds columns already demeaned over the same
    /// effects, rows and weights by their names, which are taken as they are (the outcome under
    /// `~` and its text), so that a robustness table demeans each column once.
    pub fn absorb(
        &self,
        effects: &[EffectGroups],
        options: Options,
        known: &BTreeMap<String, Rc<Vec<f64>>>,
    ) -> (Design, Absorbed) {
        let n = self.n();
        let k = self.k();
        let mut columns: Vec<Vec<f64>> = (0..k)
            .map(|c| (0..n).map(|i| self.value_at(c, i)).collect())
            .collect();
        let mut names: Vec<String> = self.names.clone();
        if self.has_outcome() {
            columns.push((0..n).map(|i| self.outcome_value_at(i)).collect());
            names.push(format!(
                "~{}",
                self.outcome_text.clone().unwrap_or_default()
            ));
        }
        // The columns to demean: those `known` lacks, and the first of all when it has every
        // one, since the kernel also decides the rows kept.
        let mut to_demean: Vec<usize> = (0..columns.len())
            .filter(|&c| !known.contains_key(&names[c]))
            .collect();
        if to_demean.is_empty() && !columns.is_empty() {
            to_demean.push(0);
        }
        let refs: Vec<&[f64]> = to_demean.iter().map(|&c| columns[c].as_slice()).collect();
        let kernel_effects: Vec<Effect<'_>> = effects
            .iter()
            .map(|e| Effect {
                group_of_row: &e.group_of_row,
                groups: e.levels(),
            })
            .collect();
        let demeaned = demean(&refs, &kernel_effects, self.weights.as_deref(), options);
        let kept = demeaned.rows;
        let weights: Option<Vec<f64>> = self
            .weights
            .as_ref()
            .map(|w| kept.iter().map(|&p| w[p]).collect());
        let mut fresh = demeaned.columns.into_iter();
        let weight = |i: usize| weights.as_ref().map_or(1.0, |w| w[i]);
        // Each column demeaned or taken from `known`; a fresh column whose remaining norm is a
        // vanishing share of its original (both over the rows kept, weighted as the fit weighs
        // them) is absorbed by the effects and zeroed; one keeping less than a hundredth of
        // its norm has little variation left. A known column was judged when it was made and
        // is judged again from its values, which cost nothing to sum.
        let mut results: Vec<Rc<Vec<f64>>> = Vec::with_capacity(columns.len());
        let mut absorbed_columns = Vec::new();
        let mut little_variation = Vec::new();
        for c in 0..columns.len() {
            let mut column: Vec<f64> = if to_demean.contains(&c) {
                fresh.next().expect("one result per column demeaned")
            } else {
                known[&names[c]].as_ref().clone()
            };
            let before: Vec<f64> = kept
                .iter()
                .enumerate()
                .map(|(i, &p)| columns[c][p] * columns[c][p] * weight(i))
                .collect();
            let after: Vec<f64> = column
                .iter()
                .enumerate()
                .map(|(i, v)| v * v * weight(i))
                .collect();
            let (before, after) = (zeff_kernels::sum(&before), zeff_kernels::sum(&after));
            if c < k && before > 0.0 && before.is_finite() {
                let share = (after / before).sqrt();
                if share < ABSORBED_SHARE {
                    column.iter_mut().for_each(|v| *v = 0.0);
                    absorbed_columns.push(c);
                } else if share < LITTLE_VARIATION_SHARE {
                    little_variation.push((c, share));
                }
            }
            results.push(Rc::new(column));
        }
        let demeaned_by_name: BTreeMap<String, Rc<Vec<f64>>> = names
            .iter()
            .cloned()
            .zip(results.iter().map(Rc::clone))
            .collect();
        let outcome: Rc<Vec<f64>> = if self.has_outcome() {
            results.pop().expect("the outcome's column")
        } else {
            Rc::new(Vec::new())
        };
        let original_outcome = self
            .has_outcome()
            .then(|| kept.iter().map(|&p| columns[k][p]).collect());
        let design = Design {
            names: self.names.clone(),
            rows: kept.iter().map(|&p| self.rows[p]).collect(),
            dropped: self.dropped,
            dropped_in: self.dropped_in.clone(),
            notes: self.notes.clone(),
            intercept: false,
            weights,
            weight_kind: self.weight_kind,
            zero_weights: self.zero_weights,
            unseen: self.unseen.clone(),
            columns: Vec::new(),
            sources: Vec::new(),
            outcome: self.outcome,
            outcome_text: self.outcome_text.clone(),
            block_rows: self.block_rows,
            dense: Some(Dense {
                columns: results,
                outcome,
            }),
            original_outcome,
        };
        let absorbed = Absorbed {
            effects: effects.iter().map(|e| e.restricted(&kept)).collect(),
            kept,
            singletons: demeaned.singletons_dropped,
            passes: demeaned.passes,
            converged: demeaned.converged,
            largest_change: demeaned.largest_change,
            absorbed_columns,
            little_variation,
            demeaned: demeaned_by_name,
        };
        (design, absorbed)
    }
}

/// The estimated effect of each level of each effect, as fixest's `fixef` recovers them: the
/// effects' part of the fitted values, y − Xβ over the rows kept (unweighted), split between the
/// effects by alternating group means (weighted under weights) until a pass changes nothing
/// beyond 1e-10 of the part's size, which one effect reaches in one pass. With several effects
/// the split is one of many that agree on every row's sum, and a prediction for new data
/// combines levels that were observed together, where the sum is the same for every split.
pub fn effect_values(
    original: &Design,
    absorbed: &Design,
    fitted: &Fitted,
    found: &Absorbed,
) -> Vec<Vec<f64>> {
    let n = absorbed.n();
    let outcome = absorbed.original_outcome.as_deref().unwrap_or(&[]);
    let part: Vec<f64> = (0..n)
        .map(|i| {
            let xb: f64 = fitted
                .kept
                .iter()
                .zip(&fitted.estimates)
                .map(|(&c, b)| original.value_at(c, found.kept[i]) * b)
                .sum();
            outcome.get(i).copied().unwrap_or(0.0) - xb
        })
        .collect();
    let weight = |i: usize| absorbed.weights.as_ref().map_or(1.0, |w| w[i]);
    let scale = part.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    let mut values: Vec<Vec<f64>> = found
        .effects
        .iter()
        .map(|e| vec![0.0; e.levels()])
        .collect();
    if found.effects.is_empty() || n == 0 {
        return values;
    }
    for _ in 0..10_000 {
        let mut largest_change = 0.0_f64;
        for e in 0..found.effects.len() {
            let groups = found.effects[e].levels();
            let mut sums = vec![0.0; groups];
            let mut weights = vec![0.0; groups];
            for i in 0..n {
                let others: f64 = (0..found.effects.len())
                    .filter(|&o| o != e)
                    .map(|o| values[o][found.effects[o].group_of_row[i]])
                    .sum();
                let g = found.effects[e].group_of_row[i];
                sums[g] += (part[i] - others) * weight(i);
                weights[g] += weight(i);
            }
            for g in 0..groups {
                let next = if weights[g] > 0.0 {
                    sums[g] / weights[g]
                } else {
                    0.0
                };
                largest_change = largest_change.max((next - values[e][g]).abs());
                values[e][g] = next;
            }
        }
        if largest_change <= 1e-10 * scale.max(f64::MIN_POSITIVE) {
            break;
        }
    }
    values
}
