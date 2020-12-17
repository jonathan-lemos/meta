use crate::cli::args::{Flag, HELP_FLAG, Positional, QUIET_FLAG, RECURSIVE_FLAG, Subcommand, FileSelector};

pub(crate) static SUBCOMMAND: Subcommand = Subcommand {
    name: "get",
    description: "Retrieves key/value pairs.",
    positional: Some(Positional {
        name: "([key,]*key)?",
        count: (None, None),
        description: "The command will print the values for the given keys. If no keys are given, it will print all key/value pairs.",
    }),
    file_selector: FileSelector::FILE_LIST | FileSelector::QUERY,
    flags: vec![HELP_FLAG, QUIET_FLAG, RECURSIVE_FLAG],
    on_parse: |e| {},
};