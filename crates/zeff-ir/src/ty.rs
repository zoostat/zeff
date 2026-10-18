//! Types as the checker sees them (Language Reference, sections 3.7 and 4).

use std::fmt;

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
            "Nothing" => Ty::Nothing,
            "Any" => Ty::Any,
            "Var" => Ty::Var,
            "Vars" => Ty::Vars,
            "Type" => Ty::Type,
            "List" => Ty::List(Box::new(Ty::Any)),
            "Record" => Ty::AnyRecord,
            _ => return None,
        })
    }

    /// Whether a value of type `actual` may be given where `self` is expected: an Int where a Float
    /// is expected (exact up to 2^53), a `Column<Int>` for a `Column<Float>`, `NA` anywhere, one
    /// name for `Vars`, and any record for `Record`.
    pub fn accepts(&self, actual: &Ty) -> bool {
        match (self, actual) {
            (_, Ty::Unknown) | (Ty::Unknown, _) => true,
            (Ty::Any, Ty::Nothing) => false,
            (Ty::Any, _) => true,
            (_, Ty::Na) => true,
            (Ty::Float, Ty::Int) => true,
            (Ty::Vars, Ty::Var) => true,
            (Ty::AnyRecord, Ty::Record(_) | Ty::AnyRecord) => true,
            (Ty::Column(a), Ty::Column(b)) | (Ty::List(a), Ty::List(b)) => a.accepts(b),
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
    /// itself, Int with Float as Float, anything with NA; `None` when they do not join.
    pub fn join(a: &Ty, b: &Ty) -> Option<Ty> {
        match (a, b) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => Some(Ty::Unknown),
            (Ty::Na, other) | (other, Ty::Na) => Some(other.clone()),
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Some(Ty::Float),
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
            Ty::Nothing => write!(f, "Nothing"),
            Ty::Any => write!(f, "Any"),
            Ty::Var => write!(f, "Var"),
            Ty::Vars => write!(f, "Vars"),
            Ty::Type => write!(f, "Type"),
            Ty::Column(t) => write!(f, "Column<{t}>"),
            Ty::List(t) if **t == Ty::Any => write!(f, "List"),
            Ty::List(t) => write!(f, "List<{t}>"),
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
    }

    #[test]
    fn articles() {
        assert_eq!(Ty::Int.with_article(), "an Int");
        assert_eq!(Ty::Frame.with_article(), "a Frame");
        assert_eq!(Ty::Any.with_article(), "an Any");
    }
}
