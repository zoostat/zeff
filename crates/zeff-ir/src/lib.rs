//! The typed intermediate representation and the checker that produces it.

pub mod check;
pub mod ir;
pub mod ty;

pub use check::{BuiltinChecked, BuiltinForm, BuiltinProblem, check_builtin, type_text};
pub use check::{Checked, check, check_source};
pub use check::{ModuleSource, check_project, check_with_text};
pub use ir::{
    Binding, BindingId, BindingKind, Call, Expr, ExprKind, FnId, Function, FunctionValue, Param,
    Program, RecordField, Slot, Spread, Statement,
};
pub use ty::{ResultKind, Ty};
