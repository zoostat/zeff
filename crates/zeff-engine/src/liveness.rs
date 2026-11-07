//! Where each binding of a script is last used (Language Reference, section 6, "Memory": in
//! `zeff run` a binding no longer used is freed after its last use in the script). One pass over
//! the typed IR gives, for every `let` of the script's top level, the index of the last top-level
//! statement that reads it; a use inside a loop or a block counts for the statement that holds
//! the loop, so a binding read in a loop lives to the loop's end. A binding never read is freed
//! right after its `let`.

use std::collections::HashMap;
use zeff_ir::uses::statement_uses;
use zeff_ir::{BindingId, Program};

/// The last top-level statement, by index, that uses each binding; a binding with no entry is
/// never used after the statement that makes it.
pub fn last_uses(program: &Program) -> HashMap<BindingId, usize> {
    let mut last: HashMap<BindingId, usize> = HashMap::new();
    for (index, statement) in program.statements.iter().enumerate() {
        let mut used: Vec<BindingId> = Vec::new();
        statement_uses(statement, &mut used);
        for id in used {
            last.insert(id, index);
        }
    }
    last
}
