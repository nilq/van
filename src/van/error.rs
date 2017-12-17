use super::TokenPosition;
use colored::Colorize;

pub struct ErrorLocation(TokenPosition, usize);

impl ErrorLocation {
    pub fn new(position: TokenPosition, span: usize) -> ErrorLocation {
        ErrorLocation(position, span)
    }
}

pub enum Response {
    Error(Option<ErrorLocation>,      String),
    Note(Option<ErrorLocation>,       String),
    Group(Vec<Response>),
}

impl Response {
    pub fn error(location: Option<ErrorLocation>, message: String) -> Response {
        Response::Error(location, message)
    }

    pub fn note(location: Option<ErrorLocation>, message: String) -> Response {
        Response::Note(location, message)
    }

    pub fn group(responses: Vec<Response>) -> Response {
        Response::Group(responses)
    }

    pub fn display(&self, lines: &Vec<&str>) {
        match *self {
            Response::Group(ref responses) => for response in responses {
                response.display(lines)
            },

            Response::Error(ref location, ref message) |
            Response::Note(ref location, ref message)  => {
                let (color, message_t) = match *self {
                    Response::Error(..)      => ("red", "error"),
                    Response::Note(..)       => ("green", "note"),
                    _                        => unreachable!(),
                };

                let message = format!("{}{}{}\n", message_t.color(color).bold(), ": ".white().bold(), message.bold());

                if let &Some(ref pos) = location {
                    let line = lines.get(pos.0.line);

                    if let Some(line) = line {
                        let prefix      = format!("{:5} |", pos.0.line + 1).blue().bold();
                        let source_line = format!("{} {}\n", prefix, line);
                        let indicator   = format!(
                            "{:offset$}{:^<count$}", " ", " ".color(color).bold(),
                            offset = prefix.len() + pos.0.col - 2,
                            count  = pos.1 + 1,
                        );
                        
                        println!("{}{}{}{}\n", message, "      |\n".blue().bold(), source_line, indicator)
                    }
                } else {
                    println!("{}", message);
                }
            }
        }
    }
}
