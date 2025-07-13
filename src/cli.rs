use crate::{AbxError, AbxToXmlConverter, Result};
use clap::{Arg, Command};

pub struct Cli;

impl Cli {
    pub fn build_command() -> Command {
        Command::new("abx2xml")
            .about("Converts Android Binary XML (ABX) to human-readable XML")
            .long_about("Converts between Android Binary XML and human-readable XML.\n\nWhen invoked with the '-i' argument, the output of a successful conversion will overwrite the original input file. Input can be '-' to use stdin, and output can be '-' to use stdout.")
            .arg(
                Arg::new("in-place")
                    .short('i')
                    .long("in-place")
                    .help("Overwrite input file with converted output")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("input")
                    .help("Input file path (use '-' for stdin)")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::new("output")
                    .help("Output file path (use '-' for stdout)")
                    .index(2),
            )
    }

    pub fn run() -> Result<()> {
        let matches = Self::build_command().get_matches();
        Self::run_with_matches(matches)
    }

    pub fn run_with_matches(matches: clap::ArgMatches) -> Result<()> {
        let input_path = matches.get_one::<String>("input").unwrap();
        let output_path = matches.get_one::<String>("output");
        let in_place = matches.get_flag("in-place");

        if in_place && input_path == "-" {
            return Err(AbxError::ParseError(
                "Cannot use -i option with stdin input".to_string(),
            ));
        }

        let output_path = match output_path {
            Some(path) => path.clone(),
            None => {
                if in_place {
                    input_path.clone()
                } else {
                    "-".to_string()
                }
            }
        };

        match (input_path.as_str(), output_path.as_str()) {
            ("-", "-") => AbxToXmlConverter::convert_stdin_stdout(),
            ("-", output) => AbxToXmlConverter::convert_stdin_to_file(output),
            (input, "-") => AbxToXmlConverter::convert_file_to_stdout(input),
            (input, output) => AbxToXmlConverter::convert_file(input, output),
        }
    }
}

// test

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ArgMatches;

    #[test]
    fn test_build_command() {
        let cmd = Cli::build_command();
        assert_eq!(cmd.get_name(), "abx2xml");
    }

    #[test]
    fn test_in_place_with_stdin_error() {
        let matches = Cli::build_command()
            .try_get_matches_from(vec!["abx2xml", "-i", "-"])
            .unwrap();

        let result = Cli::run_with_matches(matches);
        assert!(result.is_err());

        if let Err(AbxError::ParseError(msg)) = result {
            assert!(msg.contains("Cannot use -i option with stdin input"));
        } else {
            panic!("Expected ParseError");
        }
    }
}
