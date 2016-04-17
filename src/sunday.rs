use nom::{IResult,digit};
use nom::IResult::*;

use std::str;
use std::str::FromStr;


#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Machine {
    LittleDrink,
    BigDrink,
    Snack
}

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


pub fn machine_from_str(string : &str) -> Result<Machine, &str> {
    match string {
        "ld"    => Ok(Machine::LittleDrink),
        "d"     => Ok(Machine::BigDrink),
        "s"     => Ok(Machine::Snack),
        _       => Err("Invalid Machine Alias")
    }
}

named!(number<u64>,
    map_res!(
        map_res!(
            digit,
            str::from_utf8
        ),
        FromStr::from_str
    )
);


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


named!(parse_ibutton <&[u8], Command>,
    chain!(
        tag!("IBUTTON ") ~
        ibutton: take_until!("\n") ~
        tag!("\n"),
        || { match str::from_utf8(ibutton) {
            Ok(ibs) => Command::IBUTTON{ ibutton: ibs },
            Err(_)  => Command::INVALID("IBUTTON: UTF8 Error")
        }}));


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


named!(parse_stat <&[u8], Command>,
    chain!(
        tag!("STAT") ~
        tag!("\n"),
        || { Command::STAT }
    )
);


named!(parse_getbalance <&[u8], Command>,
    chain!(
        tag!("GETBALANCE") ~
        tag!("\n"),
        || { Command::GETBALANCE }
    )
);


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


named!(parse_uptime <&[u8], Command>,
       chain!(
           tag!("UPTIME") ~
           tag!("\n"),
           || {Command::UPTIME}));

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
