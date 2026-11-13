//! The typed intermediate representation and the checker that produces it.

pub mod check;
pub mod ir;
pub mod schema;
pub mod ty;
pub mod uses;
pub mod walk;

pub use check::{
    BuiltinChecked, BuiltinForm, BuiltinProblem, check_builtin, enclosing_param, type_text,
};
pub use check::{Checked, check, check_source, takes_first};
pub use check::{ModuleSource, check_project, check_project_with, check_with_text};
pub use ir::{
    Binding, BindingId, BindingKind, Call, Expr, ExprKind, FnId, Function, FunctionValue, Param,
    Program, RecordField, Slot, Spread, Statement,
};
pub use schema::{FrameSchema, ImportRequest, SchemaLookup, Schemas, column_ty};
pub use ty::{ResultKind, Ty};
