use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::str::FromStr;


/// Type for all sunday protocol commands. Each command has its arguments
/// contained within it. If you are extending the sunday protocol you will need
/// to add entries here.
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Command<'a> {
    USER{username: &'a str},
    PASS{password: &'a str},
    IBUTTON{ibutton: &'a str},
    MACHINE{machine: Machine},
    STAT,
    GETBALANCE,
    DROP{slot: u64, delay: u64},
    SENDCREDITS{credits: u64, username: &'a str},
    UPTIME,
    INVALID(&'a str),
}


/// Define the possible drink machines. If there is a new drink machine a new
/// enum entry should be added here.
/// NOTE: This should probably be generated from a config file.
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Machine {
    LittleDrink,
    BigDrink,
    Snack,
    Unknown,
}

/// Define the pairing between a string alias for a drink machine, and the
/// enum entry for that machine. If there is a new drink machine, a new pairing
/// must be added here.
pub fn machine_from_str(string : &str) -> Result<Machine, &str> {
    match string {
        "ld"    => Ok(Machine::LittleDrink),
        "d"     => Ok(Machine::BigDrink),
        "s"     => Ok(Machine::Snack),
        _       => Err("Invalid Machine Alias")
    }
}


/// Parse a number into a unsigned 64bit integer
named!(number<u64>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8
        ),
        FromStr::from_str
    )
);


/// Parse the USER command and turn it into an instance of Command::USER
named!(parse_user <&[u8], Command>,
    chain!(
        tag!("USER ") ~
        username: take_until!("\n") ~
        tag!("\n"),
        || { match str::from_utf8(username) {
            Ok(uns) => Command::USER{ username: uns },
            Err(_)  => Command::INVALID("USER: UTF8 Error")
        }}
    )
);


/// Parse the PASS command and turn it into an instance of Command::PASS
named!(parse_pass <&[u8], Command>,
    chain!(
        tag!("PASS ") ~
        pass: take_until!("\n") ~
        tag!("\n"),
        || { match str::from_utf8(pass) {
            Ok(ps) => Command::PASS{password: ps},
            Err(_) => Command::INVALID("PASS: UTF8 Error")
        }}
    )
);


/// Parse the IBUTTON command and turn it into an instance of Command::IBUTTON
named!(parse_ibutton <&[u8], Command>,
    chain!(
        tag!("IBUTTON ") ~
        ibutton: take_until!("\n") ~
        tag!("\n"),
        || { match str::from_utf8(ibutton) {
            Ok(ibs) => Command::IBUTTON{ ibutton: ibs },
            Err(_)  => Command::INVALID("IBUTTON: UTF8 Error")
        }}));


/// Parse the MACHINE command and turn it into an instance of Command::MACHINE
/// If you are adding a new drink machine you will need to add the alias as a
/// tag here.
named!(parse_machine <&[u8], Command>,
    chain!(
       tag!("MACHINE ") ~
       alias: alt!( tag!("ld") | tag!("d") | tag!("s") ) ~
       tag!("\n"),
       || { match str::from_utf8(alias) {
                Ok(als) => match machine_from_str(als) {
                    Ok(m)  => Command::MACHINE{machine: m},
                    Err(_) => Command::INVALID("MACHINE: Invalid Alias")
                },
                Err(_) => Command::INVALID("MACHINE: UTF8 Error")
       }}
    )
);


/// Parse the STAT command and turn it into an instance of Command::STAT
named!(parse_stat <&[u8], Command>,
    chain!(
        tag!("STAT") ~
        tag!("\n"),
        || { Command::STAT }
    )
);


/// Parse the GETBALANCE command and turn it into an instance of
/// Command::GETBALANCE
named!(parse_getbalance <&[u8], Command>,
    chain!(
        tag!("GETBALANCE") ~
        tag!("\n"),
        || { Command::GETBALANCE }
    )
);


/// Parse the DROP command and turn it into an instance of Command::DROP
/// By default the delay int will be set to zero if not specified.
named!(parse_drop <&[u8], Command>,
    chain!(
        tag!("DROP ") ~
        slot: number ~
        tag!(" ") ~
        delay: opt!(number) ~
        tag!("\n"),
        || { match delay {
                Some(x) => Command::DROP{slot: slot, delay: x},
                None    => Command::DROP{slot: slot, delay: 0}
        }}
    )
);


/// Parse the SENDCREDITS command and turn it into an instance of
/// Command::SENDCREDITS
named!(parse_sendcredits <&[u8], Command>,
    chain!(
        tag!("SENDCREDITS ") ~
        credits: number ~
        tag!(" ") ~
        username: take_until!("\n") ~
        tag!("\n"),
        || { match str::from_utf8(username) {
                Ok(uns) => Command::SENDCREDITS{credits: credits, username: uns},
                Err(_)  => Command::INVALID("SENDCREDITS: UTF8 Error")
        }}
    )
);


/// Parse the UPTIME command and turn it into an instance of Command::UPTIME
named!(parse_uptime <&[u8], Command>,
       chain!(
           tag!("UPTIME") ~
           tag!("\n"),
           || {Command::UPTIME}));


/// The full sunday parser, this combines all the individual command parsers
/// into a single parser that outputs a Command.
named!(parse_sunday <&[u8], Command>,
       alt!(parse_uptime        |
            parse_drop          |
            parse_getbalance    |
            parse_stat          |
            parse_machine       |
            parse_ibutton       |
            parse_pass          |
            parse_user          |
            parse_sendcredits));


/// Run the parser on an array of chars (u8) and turn the nom IResult into
/// a simple Option type. If parsing fails None is output, if it succeeds it
/// will return Some(Command).
pub fn parse_sunday_line(input: &[u8]) -> Option<Command> {
    match parse_sunday(input) {
        IResult::Done(_, res) => Some(res),
        IResult::Error(_) => None,
        IResult::Incomplete(_) => None,
    }
}


#[cfg(test)]
mod tests {
    use nom::{IResult, digit};

    #[test]
    fn test_parse_user() {
        let thingy = "USER rossdylan\n".as_bytes();
        let result: super::Command = super::parse_sunday_line(thingy).unwrap();
        assert_eq!(result, super::Command::USER{username: "rossdylan"});
    }

    #[test]
    fn test_parse_pass() {
        let thingy = "PASS herpderp\n".as_bytes();
        let result: super::Command = super::parse_sunday_line(thingy).unwrap();
        assert_eq!(result, super::Command::PASS{password: "herpderp"});
    }

    #[test]
    fn test_parse_valid_machine() {
        let thingy = "MACHINE d\n".as_bytes();
        let result: super::Command = super::parse_sunday_line(thingy).unwrap();
        assert_eq!(result, super::Command::MACHINE{machine: super::Machine::BigDrink});
    }

    #[test]
    fn test_parse_invalid_machine() {
        let thingy = "MACHINE h\n".as_bytes();
        let result: Option<super::Command> = super::parse_sunday_line(thingy);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_ibutton() {
        let thingy = "IBUTTON 47000023def1f402\n".as_bytes();
        let result: super::Command = super::parse_sunday_line(thingy).unwrap();
        assert_eq!(result, super::Command::IBUTTON{ibutton: "47000023def1f402"});
    }
}
