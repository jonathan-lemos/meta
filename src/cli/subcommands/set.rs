use crate::cli::args::{Positional, Subcommand, HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG};

pub(crate) static SUBCOMMAND: Subcommand = Subcommand {
    name: "set",
    description: "Sets the value associated with a key.",
    positional: Some(Positional {
        name: "(key=value)+",
        count: (Some(1), None),
        description: "One or more key=value assignments, meaning assign the value to the key.",
    }),
    file_entry_expr: true,
    flags: vec![HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG],
    on_parse: |_| {}
};