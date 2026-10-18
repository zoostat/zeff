//! The typed intermediate representation and the checker that produces it.

pub mod check;
pub mod ir;
pub mod messages;
pub mod ty;

pub use check::{Checked, check, check_source};
pub use ir::{Binding, BindingId, Call, Expr, ExprKind, Program, Statement};
pub use ty::Ty;
