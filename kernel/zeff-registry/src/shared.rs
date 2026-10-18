//! The shared arguments (Language Reference, section 6, "Shared arguments"): they mean the same
//! wherever they appear, and a help page takes their text from this table, so that `se:` reads the
//! same everywhere; the function may add one sentence.

/// One shared argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedArgument {
    pub name: &'static str,
    /// The type as the table writes it, default included where it has one.
    pub ty: &'static str,
    pub meaning: &'static str,
}

pub const SHARED: [SharedArgument; 15] = [
    SharedArgument {
        name: "by",
        ty: "Var | Vars",
        meaning: "Repeat the computation within each group; plots draw one panel per group.",
    },
    SharedArgument {
        name: "group",
        ty: "Var",
        meaning: "The variable that defines groups: the groups a test compares, or the groups a split keeps together. A two-group test given one level or more than two stops with an error that suggests `anova` or `kruskal_test` (Z0412).",
    },
    SharedArgument {
        name: "color",
        ty: "Var",
        meaning: "Colour points, lines or bars by group within one panel.",
    },
    SharedArgument {
        name: "fe",
        ty: "Terms",
        meaning: "Fixed effects absorbed, as fixest and reghdfe: `fe: firm + year`; `firm:year` for interacted effects; singletons are dropped and reported.",
    },
    SharedArgument {
        name: "re",
        ty: "Var | Mapping",
        meaning: "Random effects: `re: school` a random intercept; `re: [school => 1 + x]` an intercept and slope; nested groups as `school:class`, crossed groups as separate entries.",
    },
    SharedArgument {
        name: "se",
        ty: "classical | robust | hc2 | hc3 | bootstrap = classical",
        meaning: "Standard-error type; `robust` is HC1, as Stata's `robust`; `bootstrap` from 1.1. With sampling weights the default is `robust`, and `se: classical` is an error (Z0410). With `weight_type: frequency`, classical errors are valid and the observation count is the sum of the weights.",
    },
    SharedArgument {
        name: "cluster",
        ty: "Var",
        meaning: "Cluster-robust standard errors (CR1); one-observation clusters are reported; two-way and multiway clustering arrive after 1.0.",
    },
    SharedArgument {
        name: "weights",
        ty: "Var",
        meaning: "Weight variable.",
    },
    SharedArgument {
        name: "weight_type",
        ty: "frequency | analytic | sampling = sampling",
        meaning: "Stata's fweight, aweight and pweight. In 1.0 the tests refuse `weights:` with `weight_type: sampling` (Z0411) and point to regression with robust errors.",
    },
    SharedArgument {
        name: "reps",
        ty: "Int = 1000",
        meaning: "Bootstrap or simulation draws (from 1.1).",
    },
    SharedArgument {
        name: "on_fail",
        ty: "stop | skip = stop",
        meaning: "What happens when one draw fails; skipped draws are counted and reported.",
    },
    SharedArgument {
        name: "max_fail",
        ty: "Float = 0.01",
        meaning: "Largest share of draws that may fail before the run stops.",
    },
    SharedArgument {
        name: "seed",
        ty: "Int",
        meaning: "Random seed; one is chosen and recorded when omitted.",
    },
    SharedArgument {
        name: "level",
        ty: "Float = 0.95",
        meaning: "Confidence level.",
    },
    SharedArgument {
        name: "title",
        ty: "Str",
        meaning: "Title.",
    },
];

/// The shared argument called `name`, if there is one.
pub fn shared_argument(name: &str) -> Option<&'static SharedArgument> {
    SHARED.iter().find(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fifteen_shared_arguments_of_section_6() {
        let names: Vec<&str> = SHARED.iter().map(|s| s.name).collect();
        assert_eq!(
            names,
            [
                "by",
                "group",
                "color",
                "fe",
                "re",
                "se",
                "cluster",
                "weights",
                "weight_type",
                "reps",
                "on_fail",
                "max_fail",
                "seed",
                "level",
                "title"
            ]
        );
        assert!(shared_argument("se").is_some());
        assert!(shared_argument("var").is_none());
        assert!(SHARED.iter().all(|s| s.meaning.ends_with('.')));
    }
}
