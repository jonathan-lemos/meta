use crate::cli::args::{Flag, HELP_FLAG, Positional, QUIET_FLAG, RECURSIVE_FLAG, Subcommand};

pub(crate) static SUBCOMMAND: Subcommand = Subcommand {
    name: "list",
    description: "Lists files matching an expression.",
    positional: Some(Positional {
        name: "([key,]*key)?",
        count: (None, None),
        description: "The command will print the values for the given keys. If no keys are given, it will print all key/value pairs.",
    }),
    file_entry_expr: true,
    flags: vec![HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG],
    on_parse: |e| {},
};