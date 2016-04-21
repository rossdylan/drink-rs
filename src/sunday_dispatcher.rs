use sunday::{Command, Machine};

/// All the actual logic for the sunday server is contained within the
/// SundayDispatcher. Each connection has its own isntance of SundayDispatcher
/// which handles each possible command.
#[derive(Debug)]
pub struct SundayDispatcher {
    user: String,
    authed: bool,
    machine: Machine,
}

impl SundayDispatcher {
    /// Create a new instance of SundayDispatcher. Right now nothing fancy
    /// happens here. Eventually this will need to setup ldap and mysql
    /// connections.
    pub fn new() -> SundayDispatcher {
        SundayDispatcher {
            user: String::from(""),
            authed: false,
            machine: Machine::Unknown,
        }
    }

    /// Internal function used to handle the USER command. This command is
    /// really simple, it just sets the current username for this session to
    /// the given username.
    fn handle_user(&mut self, username: &str) -> String {
        self.user = String::from(username);
        String::from("OK:\n")
    }

    /// Internal function used to handle the PASS command. Currently mocked out
    /// to accept the password "foobar".
    /// TODO: Replace hard coded password with call to ldapbind
    fn handle_pass(&mut self, password: &str) -> String {
        // check password against ldap here
        if password == "foobar" {
            self.authed = true;
            format!("OK: {}\n", 0) // returns number of credits
        } else {
            String::from("ERR 407 Invalid password.\n")
        }
    }

    /// Internal function used to handle the IBUTTON command. Currently
    /// mocked out to set the user to "unknown".
    /// TODO: Replace if statement with ldap lookup
    fn handle_ibutton(&mut self, ibutton: &str) -> String {
        // look up username based on ibutton
        if true {
            self.user = String::from("unknown");
            self.authed = true;
            format!("OK: {}\n", 0) // returns number of credits
        } else {
            String::from("ERR 207 Invalid Ibutton\n")
        }
    }

    /// Internal function used to handle the MACHINE command. This simply
    /// sets the machine value of the SundayDispatcher struct.
    fn handle_machine(&mut self, machine: Machine) -> String {
        self.machine = machine;
        format!("OK: Welcome to {:?}\n", machine)
    }

    /// Public function that takes a Command and returns a Vec<u8> of response
    /// data.
    /// TODO: Finish implementing all commands
    pub fn handle_command(&mut self, cmd_opt: Option<Command>) -> Vec<u8>{
        let ret = match cmd_opt {
            Some(cmd) => match cmd {
                Command::USER{username} => {
                    self.handle_user(username)
                }
                Command::PASS{password} => {
                    self.handle_pass(password)
                }
                Command::IBUTTON{ibutton} => {
                    self.handle_ibutton(ibutton)
                }
                Command::MACHINE{machine} => {
                    self.handle_machine(machine)
                }
                _ => String::from("ERR 420 Unimplemented")
            },
            None => String::from("ERR 415 Invalid command\n")
        };
        ret.as_str().as_bytes().into()
    }
}
