use crate::cli::args::{Flag, Subcommand};
use crate::linq::collectors::IntoVec;
use crate::cli::print::print;
use crate::cli::program::{version, program_name, description};
use colored::Colorize;

static INDENT: usize = 4;

fn flag_desc(f: &Flag) -> String {
    match f.short {
        Some(s) => f.long.to_owned() + ", " + s,
        None => f.long.to_owned()
    }
}

pub fn print_version() {
    print().line(&format!("{} {}", program_name(), version().bold().yellow()));
}

pub fn print_help(subcommands: &[Subcommand], flags: &[Flag], prog_invocation: &str) {
    let mut comm = subcommands.clone();
    comm.sort_by_key(|k| k.name);
    let global_subcomm_len = comm.iter().map(|x| x.name.chars().count()).max();

    let mut global_flags = flags.clone();
    global_flags.sort_by_key(|k| k.long);
    let global_flag_desc = global_flags.iter().map(flag_desc).into_vec();
    let global_flag_desc_len = global_flag_desc.iter().map(|x| x.chars().count()).max();

    // meta 1.0.0
    print().line(&format!("{} {}", program_name().bold().yellow(), version()));
    // A command-line utility for managing file/directory metadata.
    print().line(description());

    // USAGE:
    print().line("USAGE:");
    //     meta [flags*] [subcommand] [subcommand-argument*]
    print().indent(INDENT);
    print().line(&format!("{} {} {} {}", prog_invocation, "[flags*]".italic(), "[subcommand]".italic(), "[subcommand-argument*]".italic()));

    print().set_indent(0);
    print().newline();

    // FLAGS:
    print().line("FLAGS:");

    for (flag, desc) in global_flags.iter().zip(global_flag_desc.iter()) {
        print().indent(INDENT);
        // -f --flag
        print().str(&flag_desc(flag));
        print().space(global_flag_desc_len.unwrap() + INDENT);

        print().indent(global_flag_desc_len.unwrap() + INDENT);

        //              description
        print().line(desc);
    }

    print().set_indent(0);
    print().newline();

    // SUBCOMMANDS:
    print().line("SUBCOMMANDS:");

    for subcommand in comm {
        print().indent(INDENT);

        // subcommand
        print().str(subcommand.name);

        print().space(global_subcomm_len.unwrap() + INDENT);
        print().indent(global_subcomm_len.unwrap() + INDENT);

        //               description
        print().line(subcommand.description);
    }

    print().set_indent(0);
    print().newline();

    print().line(&format!("For more details about a subcommand, type {} {} {}", prog_invocation, "[subcommand]".italic(), "--help"));
}

pub fn print_help_subcommand(sc: &Subcommand, prog_invocation: &str) {
    let mut global_flags = sc.flags.clone();
    global_flags.sort_by_key(|k| k.long);
    let global_flag_desc = global_flags.iter().map(flag_desc).into_vec();
    let global_flag_desc_len = global_flag_desc.iter().map(|x| x.chars().count()).max();

    // meta 1.0.0
    print().line(&format!("{} {}", program_name().bold().yellow(), version()));
    // A command-line utility for managing file/directory metadata.
    print().line(description());

    // USAGE:
    print().line("USAGE:");
    //     meta [flags*] [subcommand] [subcommand-argument*]
    print().indent(INDENT);
    print().line(&format!("{} {} {} {}", prog_invocation, "[flags*]".italic(), "[subcommand]".italic(), "[subcommand-argument*]".italic()));

    print().set_indent(0);
    print().newline();

    // FLAGS:
    print().line("FLAGS:");

    for (flag, desc) in global_flags.iter().zip(global_flag_desc.iter()) {
        print().indent(INDENT);
        // -f --flag
        print().str(&flag_desc(flag));
        print().space(global_flag_desc_len.unwrap() + INDENT);

        print().indent(global_flag_desc_len.unwrap() + INDENT);

        //              description
        print().line(desc);
    }

    print().set_indent(0);
    print().newline();

    if let Some(p) = &sc.positional {
        // POSITIONAL:
        print().line("POSITIONAL:");
        print().set_indent(INDENT);
        print().str(p.name);
        print().set_indent(p.name.chars().count() + INDENT);
        print().line(p.description);
    }

    print().set_indent(0);
    print().newline();
}