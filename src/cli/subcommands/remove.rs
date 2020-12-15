use crate::cli::args::{Flag, HELP_FLAG, QUIET_FLAG, Subcommand, Positional, RECURSIVE_FLAG};

pub static SUBCOMMAND: Subcommand = Subcommand {
    name: "remove",
    description: "Removes metadata.",
    positional: Some(Positional {
        name: "([key,]*key)*",
        count: (Some(1), None),
        description: "The command will remove the given keys.",
    }
    ),
    file_entry_expr: true,
    flags: vec![HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG, Flag {
        aliases: vec!["--all", "-a"]
        equals_name: None,
        description: "Removes all of the keys from the given targets.",
    }],
    on_parse: |_| {}
};