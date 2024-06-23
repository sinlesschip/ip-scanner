#![feature(ip_bits)]
use colored::Colorize;
use sqlite::Connection;
use std::collections::HashMap;
use std::{str, net::Ipv4Addr, process::Command, thread, env};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

fn main() {

    let args: Vec<String> = env::args().collect();
    let arg_parser = ArgParser::new(args);
    let threads = match arg_parser.get("threads") {
        Some(arg) => arg[0].parse::<u32>().unwrap(),
        None => 5, 
    };

    let db = DB::init("valid_ips.db");
    let mut app = App::new(&db, threads);
    app.run();

}

struct ArgParser<> {
    args: HashMap<String, Vec<String>>
 }

impl ArgParser {
    fn new(mut args: Vec<String>) -> ArgParser {
        let mut args_mapped = HashMap::new();
        let mut popped: Vec<String> = vec![];
        while args.len() > 0 {
            let mut arg = args.pop().unwrap();
            if arg.starts_with("--") { 
                let arg = arg.replace("--", "");
                args_mapped.insert(arg, popped.to_owned());
                popped = vec![];
            } else {
                popped.push(arg);
            }
        }

        ArgParser { args: args_mapped }
    }    
    fn get(&self, arg: &str) -> Option<Vec<String>> {
        match self.args.get(arg) {
            Some(v) => Some(v.clone()),
            None => None
    
    } 
}
}

struct App<'a> {
    last_checked: u32,
    max_ip_addr: u32,
    db: &'a DB,
    num_of_threads: u32,
    reserved_ip_addr: [(u32, u32); 16],
}

impl App<'_> {
    fn new(db: &DB, threads: u32) -> App {
        App {
            last_checked: db.last_checked(),
            max_ip_addr: Ipv4Addr::new(255, 255, 255, 255).to_bits(),
            db: &db,
            num_of_threads: threads,
            reserved_ip_addr: [
                (0, 16777215),
                (167772160, 184549375),
                (1681915904, 1686110207),
                (2130706432, 2147483647),
                (2851995648, 2852061183),
                (2886729728, 2887778303),
                (3221225472, 3221225727),
                (3221225984, 3221226239),
                (3227017984, 3227018239),
                (3232235520, 3232301055),
                (3323068416, 3323199487),
                (3325256704, 3325256959),
                (3405803776, 3405804031),
                (3758096384, 4026531839),
                (3925606400, 3925606655),
                (4026531840, 4294967295),
            ],
        }
    }

    fn run(&mut self) {
        let (tx, rx): (
            Sender<Result<Ipv4Addr, Ipv4Addr>>,
            Receiver<Result<Ipv4Addr, Ipv4Addr>>,
        ) = mpsc::channel();
        while self.last_checked < self.max_ip_addr {
            for _ in 0..self.num_of_threads {

                let ip_checker = IPChecker::new();
                
                match ip_checker.check_block(self.reserved_ip_addr, &self.last_checked) {
                    Some(block) => {
                        self.last_checked = block.1 + 1;
                    }
                    None => (),
                }

                

                let ip_address = Ipv4Addr::from(self.last_checked);

                // The sender endpoint can be copied
                let thread_tx = tx.clone();
                let _child = thread::spawn(move || {
                    // The thread takes ownership over `thread_tx`
                    // Each thread queues a message in the channel
                    thread_tx.send(ip_checker.ping_ip(ip_address)).unwrap();
                });

                self.last_checked += 1;
            }

            for _ in 0..self.num_of_threads {
                // The `recv` method picks a message from the channel
                match rx.recv().unwrap() {
                    Ok(ip) => {
                        println!("{}: host is up", ip.to_string().green().bold());
                        self.db.put_ip(&self.db.conn, ip)
                    }
                    Err(ip) => {
                        println!("{}: host is down", ip.to_string().red().bold());
                        self.db.put_ip(&self.db.conn, ip)
                    }
                }
            }
        }
    }
}

struct IPChecker {}

impl IPChecker {
    fn new() -> IPChecker {
        IPChecker {}
    }

    fn ping_ip(&self, ip_address: Ipv4Addr) -> Result<Ipv4Addr, Ipv4Addr> {
        println!(
            "Checking if {} is a valid ip",
            ip_address.to_string().yellow().bold()
        );
        // run "ping -c 1 ip_address"
        let output = Command::new("ping")
            .args(["-c", "1", &ip_address.to_string()])
            .output()
            .expect("failed to execute ping");
        if output.status.success() {
            Ok(ip_address)
        } else {
            Err(ip_address)
        }
    }

    fn check_ip(&self, ip_address: Ipv4Addr) {
        let output = Command::new("nmap")
            .args(["-Pn", "--script", "vuln", &ip_address.to_string()])
            .output()
            .expect("fn");
        let mut log = String::new();
        log.push_str(match str::from_utf8(&output.stdout) {
            Ok(v) => v,
            Err(e) => panic!(),
        });
        println!("{}", log);
    }
    
    fn check_block(&self, blocks: [(u32, u32); 16], ip: &u32) -> Option<(u32, u32)> {
        for i in blocks {
            if ip >= &i.0 {
                if ip <= &i.1 {
                    return Some(i);
                }
            }
        }
        None
    }
}

struct DB {
    conn: Connection,
}

impl DB {
    fn init(path: &str) -> DB {
        let connection = sqlite::open(path).unwrap();

        let create_tables = "
            CREATE TABLE IF NOT EXISTS checked(addr, UNIQUE(addr));
            CREATE TABLE IF NOT EXISTS ip(addr, UNIQUE(addr))
            ";
        connection.execute(create_tables).unwrap();

        DB { conn: connection }
    }

    fn last_checked(&self) -> u32 {
        //returns the last checked ip in db
        let query = "SELECT * FROM checked WHERE ROWID IN ( SELECT max( ROWID ) FROM checked)";
        let mut statement = self.conn.prepare(query).unwrap();
        let mut last_checked = 0;
        while let Ok(sqlite::State::Row) = statement.next() {
            last_checked = statement
                .read::<String, _>("addr")
                .unwrap()
                .parse()
                .unwrap();
        }

        last_checked
    }

    fn put_ip(&self, conn: &Connection, ip: Ipv4Addr) {
        //put ip in checked ips
        conn.execute(format!("INSERT OR IGNORE INTO {} VALUES ('{}')", "checked", ip.to_bits().to_string()))
            .unwrap();
    }
}
