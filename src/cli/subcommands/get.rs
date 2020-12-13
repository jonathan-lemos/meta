use crate::cli::args::{Subcommand, Flag, Positional};

static SUBCOMMAND: Subcommand = Subcommand {
    name: "get",
    description: "Retrieves key/value pairs.",
    positional: vec![
        Positional {
            name: "([key,]*key)?",
            description: "The command will print the values for the given keys. If no keys are given, it will print all key/value pairs."
        }
    ],
    positional_expr: true,
    flags: vec![HELP_FLAG, VERBOSE_FLAG],
};
