
    use std::collections::HashMap;
    use std::env;
    #[derive(Debug)]
    #[derive(PartialEq)]
    pub enum ArgType {
        String(String),
        Int,
        Float(f64),
        Bool(bool),   
    }
    
    #[derive(Debug)]
    pub enum Type {
        Int(Vec<u32>),
        String(Vec<String>),
    }
    
    #[derive(Debug)]
    pub struct ArgParser {
        args: HashMap<String, Type>,
        user_args: HashMap<String, ArgType>,
    }
    
    
    impl ArgParser {
        pub fn new() -> ArgParser { 
            ArgParser {
                args: HashMap::new(),
                user_args: HashMap::new(),
            }
        }
    
        pub fn parse_args(&mut self) {
            let mut args: Vec<String> = env::args().collect();
            let mut args_mapped = HashMap::new();
            let mut popped: Vec<String> = vec![];
            while args.len() > 0 {
                // need to re write this, maybe match on the each arg, if none are found
                // to be in our args list then we know theres an invalid argument
                // so we must find it, somehow...
                let mut argument = args.pop().unwrap();
                if argument.starts_with("--") {
                    let argument = argument.replace("--", "");
                    match self.user_args.get(&argument) {
                        Some(arg) => {    
                            if *arg == ArgType::Int {
                                let mut popped: Vec<u32> = popped.iter().map(|x| x.parse::<u32>().expect("Invalid argument")).collect();
                                args_mapped.insert(argument, Type::Int(popped));
                                popped = vec![];
                        }  else {
                            args_mapped.insert(argument, Type::String(popped.to_owned()));
                            popped = vec![];
                    }
                        },
                        None => panic!("Invalid argument: {}", argument),
                    }
                } else {
                    popped.push(argument);
                }
            } 
            self.args = args_mapped;
        }
        pub fn get(&self, arg: &str) -> Option<&Type> {
            match self.args.get(arg) {
                Some(v) => {
                        Some(v)
                   
                    },
                None => None,
            }
        }
        pub fn add_argument(&mut self, name: &str, argtype: ArgType) {
            self.user_args.insert(name.to_owned(), argtype);
        }
    }
    