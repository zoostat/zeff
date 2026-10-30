//! The typed intermediate representation and the checker that produces it.

pub mod check;
pub mod ir;
pub mod ty;

pub use check::{Checked, check, check_source};
pub use ir::{
    Binding, BindingId, BindingKind, Call, Expr, ExprKind, Program, RecordField, Slot, Spread,
    Statement,
};
pub use ty::{ResultKind, Ty};
