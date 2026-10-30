//! Types as the checker sees them (Language Reference, sections 3.7 and 4).

use std::fmt;

/// The kinds of result of this release (section 4): what a value of type `Result` may be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultKind {
    /// A regression fit, `wages.reg(f)`.
    RegFit,
    /// The result of a test, `m.test(...)`.
    TestResult,
}

impl ResultKind {
    pub fn name(self) -> &'static str {
        match self {
            ResultKind::RegFit => "RegFit",
            ResultKind::TestResult => "TestResult",
        }
    }

    pub fn parse(text: &str) -> Option<ResultKind> {
        match text {
            "RegFit" => Some(ResultKind::RegFit),
            "TestResult" => Some(ResultKind::TestResult),
            _ => None,
        }
    }
}

/// A type. The variants are the types of section 4 that this release's functions take and
/// return, plus three the checker needs: `Na` for a bare `NA` whose type is not yet known,
/// `Union` for a parameter that takes several types (`Var | Vars`), and `Unknown` for an
/// expression that already produced an error, so that no second message follows from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Bool,
    Int,
    Float,
    Str,
    Date,
    DateTime,
    Frame,
    Table,
    /// A probability distribution (section 4; `dist.normal()`).
    Distribution,
    /// A formula value, `y ~ x` (sections 4 and 7.1).
    Formula,
    /// The right side of a formula without `~`, as `fe:` takes it at 0.6.0; nothing takes it yet.
    Terms,
    /// A result (section 4): the interface `Result`, which every fitted model and test
    /// implements, or one of its kinds. A method registered on `Result` applies to every kind and
    /// one registered on a kind to that kind alone (Engine and Project Specification, section
    /// 8.1, rules 5 and 6); two kinds join to `Result` (section 3.7, "Joining types").
    Result(Option<ResultKind>),
    /// A term name of a result's model matrix (section 7.2): a bare name or text in a call,
    /// checked against the result when the statement runs.
    Term,
    /// An expression over term names giving the inner type (`Term<Bool>` for a restriction of
    /// `test`, `Term<Float>` for a definition of `derive`): inside it a bare name is a
    /// coefficient, never a column or a binding (section 7.2), and the expression is kept as
    /// written for the function to read.
    TermExpr(Box<Ty>),
    /// The type of a call that returns no value; it cannot be bound or passed.
    Nothing,
    /// Any value at all (`print`).
    Any,
    /// A variable name, checked against a frame.
    Var,
    /// One or more variable names.
    Vars,
    /// A type name used as a value (`Str`, `Date("%d/%m/%Y")`), for `types:`.
    Type,
    Column(Box<Ty>),
    List(Box<Ty>),
    /// `Mapping<K, V>`: one key type and one value type (section 3.7).
    Mapping(Box<Ty>, Box<Ty>),
    /// A record literal's own type: its fields and their types.
    Record(Vec<(String, Ty)>),
    /// `Record` alone in a signature: any record.
    AnyRecord,
    /// A fixed list of bare words.
    Words(Vec<String>),
    Union(Vec<Ty>),
    Na,
    Unknown,
}

impl Ty {
    /// A type as a signature writes it: `Frame`, `Column<Bool>`, `List<Str>`, `List`, `Record`,
    /// `keep | snake`, `Var | Vars`. Alternatives that all begin with a capital are types; those
    /// in lower case are words.
    pub fn parse(text: &str) -> Option<Ty> {
        let text = text.trim();
        // A generic first, so that `List<n | missing>` is a list of words, not a union.
        if let Some(inner) = text
            .strip_prefix("Column<")
            .and_then(|t| t.strip_suffix('>'))
        {
            return Ty::parse(inner).map(|t| Ty::Column(Box::new(t)));
        }
        if let Some(inner) = text.strip_prefix("List<").and_then(|t| t.strip_suffix('>')) {
            return Ty::parse(inner).map(|t| Ty::List(Box::new(t)));
        }
        if let Some(inner) = text.strip_prefix("Term<").and_then(|t| t.strip_suffix('>')) {
            return Ty::parse(inner).map(|t| Ty::TermExpr(Box::new(t)));
        }
        if let Some(inner) = text
            .strip_prefix("Mapping<")
            .and_then(|t| t.strip_suffix('>'))
        {
            let (key, value) = inner.split_once(',')?;
            return Some(Ty::Mapping(
                Box::new(Ty::parse(key)?),
                Box::new(Ty::parse(value)?),
            ));
        }
        if text.contains('|') {
            let parts: Vec<&str> = text.split('|').map(str::trim).collect();
            if parts.iter().any(|p| p.is_empty()) {
                return None;
            }
            if parts
                .iter()
                .all(|p| p.starts_with(|c: char| c.is_ascii_uppercase()))
            {
                return parts
                    .iter()
                    .map(|p| Ty::parse(p))
                    .collect::<Option<Vec<Ty>>>()
                    .map(Ty::Union);
            }
            if parts.iter().all(|p| {
                p.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            }) {
                return Some(Ty::Words(parts.iter().map(|p| (*p).to_string()).collect()));
            }
            return None;
        }
        Some(match text {
            "Bool" => Ty::Bool,
            "Int" => Ty::Int,
            "Float" => Ty::Float,
            "Str" => Ty::Str,
            "Date" => Ty::Date,
            "DateTime" => Ty::DateTime,
            "Frame" => Ty::Frame,
            "Table" => Ty::Table,
            "Distribution" => Ty::Distribution,
            "Formula" => Ty::Formula,
            "Terms" => Ty::Terms,
            "Result" => Ty::Result(None),
            "Term" => Ty::Term,
            "Nothing" => Ty::Nothing,
            "Any" => Ty::Any,
            "Var" => Ty::Var,
            "Vars" => Ty::Vars,
            "Type" => Ty::Type,
            "List" => Ty::List(Box::new(Ty::Any)),
            "Mapping" => Ty::Mapping(Box::new(Ty::Any), Box::new(Ty::Any)),
            "Column" => Ty::Column(Box::new(Ty::Unknown)),
            "Record" => Ty::AnyRecord,
            _ => return ResultKind::parse(text).map(|kind| Ty::Result(Some(kind))),
        })
    }

    /// The words a parameter of this type accepts: those of a word list, inside a union or a
    /// list of words (`List<n | missing | mean>`); none for any other type.
    pub fn words(&self) -> Vec<String> {
        match self {
            Ty::Words(words) => words.clone(),
            Ty::Union(options) => options.iter().flat_map(Ty::words).collect(),
            Ty::List(inner) => inner.words(),
            _ => Vec::new(),
        }
    }

    /// Whether a value of type `actual` may be given where `self` is expected: an Int where a Float
    /// is expected (exact up to 2^53), a `Column<Int>` for a `Column<Float>`, `NA` anywhere, one
    /// name for `Vars`, any record for `Record`, any kind of result for `Result`, and text for a
    /// `Term`.
    pub fn accepts(&self, actual: &Ty) -> bool {
        match (self, actual) {
            (_, Ty::Unknown) | (Ty::Unknown, _) => true,
            (Ty::Any, Ty::Nothing) => false,
            (Ty::Any, _) => true,
            (_, Ty::Na) => true,
            (Ty::Float, Ty::Int) => true,
            (Ty::Vars, Ty::Var) => true,
            (Ty::Result(None), Ty::Result(_)) => true,
            (Ty::Term, Ty::Str) => true,
            (Ty::AnyRecord, Ty::Record(_) | Ty::AnyRecord) => true,
            // A word kept in a record, `{se: robust}`, has the type of that one word.
            (Ty::Words(allowed), Ty::Words(given)) => given.iter().all(|w| allowed.contains(w)),
            (Ty::Column(a), Ty::Column(b))
            | (Ty::List(a), Ty::List(b))
            | (Ty::TermExpr(a), Ty::TermExpr(b)) => a.accepts(b),
            (Ty::Mapping(k1, v1), Ty::Mapping(k2, v2)) => k1.accepts(k2) && v1.accepts(v2),
            (Ty::Union(options), other) => options.iter().any(|o| o.accepts(other)),
            (Ty::Record(a), Ty::Record(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|((n1, t1), (n2, t2))| n1 == n2 && t1.accepts(t2))
            }
            (a, b) => a == b,
        }
    }

    /// The one type of two expressions that must agree (section 3.7, "Joining types"): a type with
    /// itself, Int with Float as Float, anything with NA, two kinds of result as `Result`; `None`
    /// when they do not join.
    pub fn join(a: &Ty, b: &Ty) -> Option<Ty> {
        match (a, b) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => Some(Ty::Unknown),
            (Ty::Na, other) | (other, Ty::Na) => Some(other.clone()),
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Some(Ty::Float),
            (Ty::Result(x), Ty::Result(y)) => Some(Ty::Result(if x == y { *x } else { None })),
            (Ty::List(x), Ty::List(y)) => Ty::join(x, y).map(|t| Ty::List(Box::new(t))),
            (Ty::Column(x), Ty::Column(y)) => Ty::join(x, y).map(|t| Ty::Column(Box::new(t))),
            (Ty::Mapping(k1, v1), Ty::Mapping(k2, v2)) => Some(Ty::Mapping(
                Box::new(Ty::join(k1, k2)?),
                Box::new(Ty::join(v1, v2)?),
            )),
            (x, y) if x == y => Some(x.clone()),
            _ => None,
        }
    }

    /// A type's name as a message writes it, with an article: `a Frame`, `an Int`.
    pub fn with_article(&self) -> String {
        // A union reads as a choice: `a Bool, Int, Float or Str`.
        if let Ty::Union(options) = self
            && options.len() > 1
        {
            let names: Vec<String> = options.iter().map(ToString::to_string).collect();
            let (last, first) = names.split_last().unwrap();
            let head = options[0].with_article();
            let article = head.split(' ').next().unwrap_or("a");
            return format!("{article} {} or {last}", first.join(", "));
        }
        if let Ty::Words(words) = self
            && words.len() == 1
        {
            return format!("the word `{}`", words[0]);
        }
        let name = self.to_string();
        // `NA` is said letter by letter, so it takes `an`.
        let vowel = name == "NA"
            || (name.starts_with(['A', 'E', 'I', 'O', 'U']) && !name.starts_with("Union"));
        format!("{} {name}", if vowel { "an" } else { "a" })
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Bool => write!(f, "Bool"),
            Ty::Int => write!(f, "Int"),
            Ty::Float => write!(f, "Float"),
            Ty::Str => write!(f, "Str"),
            Ty::Date => write!(f, "Date"),
            Ty::DateTime => write!(f, "DateTime"),
            Ty::Frame => write!(f, "Frame"),
            Ty::Table => write!(f, "Table"),
            Ty::Distribution => write!(f, "Distribution"),
            Ty::Formula => write!(f, "Formula"),
            Ty::Terms => write!(f, "Terms"),
            Ty::Result(None) => write!(f, "Result"),
            Ty::Result(Some(kind)) => write!(f, "{}", kind.name()),
            Ty::Term => write!(f, "Term"),
            Ty::TermExpr(inner) => write!(f, "Term<{inner}>"),
            Ty::Nothing => write!(f, "Nothing"),
            Ty::Any => write!(f, "Any"),
            Ty::Var => write!(f, "Var"),
            Ty::Vars => write!(f, "Vars"),
            Ty::Type => write!(f, "Type"),
            Ty::Column(t) if **t == Ty::Unknown => write!(f, "Column"),
            Ty::Column(t) => write!(f, "Column<{t}>"),
            Ty::List(t) if **t == Ty::Any => write!(f, "List"),
            Ty::List(t) => write!(f, "List<{t}>"),
            Ty::Mapping(k, v) if **k == Ty::Any && **v == Ty::Any => write!(f, "Mapping"),
            Ty::Mapping(k, v) => write!(f, "Mapping<{k}, {v}>"),
            Ty::Record(fields) => {
                write!(f, "{{")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}: {ty}")?;
                }
                write!(f, "}}")
            }
            Ty::AnyRecord => write!(f, "Record"),
            Ty::Words(words) => write!(f, "{}", words.join(" | ")),
            Ty::Union(types) => {
                let names: Vec<String> = types.iter().map(ToString::to_string).collect();
                write!(f, "{}", names.join(" | "))
            }
            Ty::Na => write!(f, "NA"),
            Ty::Unknown => write!(f, "?"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_parse_and_print() {
        for text in [
            "Frame",
            "Formula",
            "Terms",
            "Result",
            "RegFit",
            "TestResult",
            "Term",
            "Term<Bool>",
            "Term<Float>",
            "Mapping",
            "Var",
            "Vars",
            "Str",
            "Int",
            "Float",
            "Bool",
            "Date",
            "DateTime",
            "Table",
            "Nothing",
            "Any",
            "Type",
            "List",
            "Record",
            "Column<Bool>",
            "List<Str>",
            "List<List<Int>>",
            "Mapping<Int, Str>",
        ] {
            let ty = Ty::parse(text).unwrap_or_else(|| panic!("{text}"));
            assert_eq!(ty.to_string(), text);
        }
        assert_eq!(
            Ty::parse("keep | snake"),
            Some(Ty::Words(vec!["keep".into(), "snake".into()]))
        );
        assert_eq!(
            Ty::parse("Var | Vars"),
            Some(Ty::Union(vec![Ty::Var, Ty::Vars]))
        );
        assert_eq!(Ty::parse("Str | Int").unwrap().to_string(), "Str | Int");
        for bad in ["", "Frame<Int>", "keep | Snake", "Column<", "Thing", "a | "] {
            assert_eq!(Ty::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn what_a_parameter_accepts() {
        assert!(Ty::Float.accepts(&Ty::Int) && !Ty::Int.accepts(&Ty::Float));
        assert!(Ty::Any.accepts(&Ty::Frame) && !Ty::Any.accepts(&Ty::Nothing));
        assert!(
            Ty::Str.accepts(&Ty::Na) && Ty::Vars.accepts(&Ty::Var) && !Ty::Var.accepts(&Ty::Vars)
        );
        assert!(
            Ty::parse("Column<Float>")
                .unwrap()
                .accepts(&Ty::parse("Column<Int>").unwrap())
        );
        assert!(
            Ty::parse("List")
                .unwrap()
                .accepts(&Ty::parse("List<Str>").unwrap())
        );
        assert!(
            !Ty::parse("List<Str>")
                .unwrap()
                .accepts(&Ty::parse("List<Int>").unwrap())
        );
        assert!(Ty::AnyRecord.accepts(&Ty::Record(vec![("a".into(), Ty::Int)])));
        assert!(!Ty::Str.accepts(&Ty::Int) && !Ty::Frame.accepts(&Ty::Table));
        assert!(Ty::parse("Var | Vars").unwrap().accepts(&Ty::Vars));
        assert!(
            Ty::Str.accepts(&Ty::Unknown),
            "an expression that already failed is not reported twice"
        );
        let (any, reg, test) = (
            Ty::Result(None),
            Ty::Result(Some(ResultKind::RegFit)),
            Ty::Result(Some(ResultKind::TestResult)),
        );
        assert!(any.accepts(&reg) && any.accepts(&test) && any.accepts(&any));
        assert!(reg.accepts(&reg) && !reg.accepts(&test) && !reg.accepts(&any));
        assert!(!Ty::Frame.accepts(&reg) && !reg.accepts(&Ty::Frame));
        assert!(Ty::Term.accepts(&Ty::Str) && Ty::Term.accepts(&Ty::Term));
        assert!(!Ty::Term.accepts(&Ty::Int) && !Ty::Str.accepts(&Ty::Term));
    }

    #[test]
    fn the_result_kinds_of_the_registry_are_the_checkers() {
        for kind in zeff_registry::RESULT_KINDS {
            assert!(
                matches!(Ty::parse(kind), Some(Ty::Result(Some(_)))),
                "{kind}"
            );
        }
        assert!(zeff_registry::is_result_type("Result"));
        assert_eq!(
            zeff_registry::RESULT_KINDS.len(),
            [ResultKind::RegFit, ResultKind::TestResult].len()
        );
    }

    #[test]
    fn joining_types() {
        assert_eq!(Ty::join(&Ty::Int, &Ty::Float), Some(Ty::Float));
        assert_eq!(Ty::join(&Ty::Na, &Ty::Str), Some(Ty::Str));
        assert_eq!(Ty::join(&Ty::Str, &Ty::Str), Some(Ty::Str));
        assert_eq!(Ty::join(&Ty::Str, &Ty::Int), None);
        assert_eq!(
            Ty::join(&Ty::List(Box::new(Ty::Int)), &Ty::List(Box::new(Ty::Float))),
            Some(Ty::List(Box::new(Ty::Float)))
        );
        let (reg, test) = (
            Ty::Result(Some(ResultKind::RegFit)),
            Ty::Result(Some(ResultKind::TestResult)),
        );
        assert_eq!(Ty::join(&reg, &reg), Some(reg.clone()));
        assert_eq!(Ty::join(&reg, &test), Some(Ty::Result(None)));
        assert_eq!(Ty::join(&Ty::Result(None), &reg), Some(Ty::Result(None)));
        assert_eq!(Ty::join(&reg, &Ty::Frame), None);
        assert_eq!(
            Ty::join(&Ty::List(Box::new(reg)), &Ty::List(Box::new(test))),
            Some(Ty::List(Box::new(Ty::Result(None))))
        );
    }

    #[test]
    fn articles() {
        assert_eq!(Ty::Int.with_article(), "an Int");
        assert_eq!(Ty::Frame.with_article(), "a Frame");
        assert_eq!(Ty::Any.with_article(), "an Any");
    }
}
