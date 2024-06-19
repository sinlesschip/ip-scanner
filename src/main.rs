#![feature(ip_bits)]
use colored::Colorize;
use sqlite::Connection;
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

fn main() {
    const MAX_IP_ADDR: u32 = Ipv4Addr::new(255, 255, 255, 255).to_bits();
    // pre calculated ip address in int
    // data: [(start_of_range, end_of_range), (start_of_range, end_of_range),]
    const RESERVED_IP_ADDR: [(u32, u32); 16] = [
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
    ];

    static NTHREADS: usize = 4000;
    
    let connection = DB::init().conn;

    // get last checked ip address
    let query = "SELECT * FROM checked WHERE ROWID IN ( SELECT max( ROWID ) FROM checked)";
    let mut statement = connection.prepare(query).unwrap();
    let mut last_checked = 0;
    while let Ok(sqlite::State::Row) = statement.next() {
        last_checked = statement
            .read::<String, _>("addr")
            .unwrap()
            .parse()
            .unwrap();
    }

    let (tx, rx): (
        Sender<Result<Ipv4Addr, Ipv4Addr>>,
        Receiver<Result<Ipv4Addr, Ipv4Addr>>,
    ) = mpsc::channel();
    while last_checked < MAX_IP_ADDR {

        for _ in 0..NTHREADS {
            match check_block(RESERVED_IP_ADDR, &last_checked) {
                Some(block) => {
                    last_checked = block.1 + 1;
                }
                None => (),
            }

            let ip_address = Ipv4Addr::from(last_checked);

            // The sender endpoint can be copied
            let thread_tx = tx.clone();
            let _child = thread::spawn(move || {
                // The thread takes ownership over `thread_tx`
                // Each thread queues a message in the channel
                thread_tx.send(ping_ip(ip_address)).unwrap();
            });

            last_checked += 1;
        }

        for _ in 0..NTHREADS {
            // The `recv` method picks a message from the channel
            match rx.recv().unwrap() {
                Ok(ip) => {
                    println!("{}: host is up", ip.to_string().green().bold());
                    put_ip(&connection, ip)
                }
                Err(ip) => {
                    println!("{}: host is down", ip.to_string().red().bold());
                    put_ip(&connection, ip)
                }
            }
        }
    }
}

macro_rules! insert_str {
    () => {
        "INSERT OR IGNORE INTO {} VALUES ('{}')"
    };
}

fn put_ip(conn: &Connection, ip: Ipv4Addr) {
    //put ip in checked ips
    conn.execute(format!(insert_str!(), "checked", ip.to_bits().to_string()))
        .unwrap();
}

fn check_block(blocks: [(u32, u32); 16], ip: &u32) -> Option<(u32, u32)> {
    for i in blocks {
        if ip >= &i.0 {
            if ip <= &i.1 {
                return Some(i);
            }
        }
    }
    None
}

fn ping_ip(ip_address: Ipv4Addr) -> Result<Ipv4Addr, Ipv4Addr> {
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

struct DB {
    conn: Connection,
}

impl DB {
    fn init() -> DB {
        let connection = sqlite::open("valid_ips.db").unwrap();

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
}