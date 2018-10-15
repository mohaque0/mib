use std::collections::HashMap;

type OptHandler<T> = fn(&OptParser<T>, &mut T, &mut ArgExaminer) -> Result<(), String>;

pub struct ArgExaminer<'a> {
    args: &'a Vec<String>,
    i: usize
}

pub struct OptParser<T> {
    program_name: String,
    usage: HashMap<String, String>,
    opt_handlers: HashMap<String, OptHandler<T>>
}

impl <'a> ArgExaminer<'a> {

    fn new<'b>(args: &'b Vec<String>, i: usize) -> ArgExaminer<'b> {
        return ArgExaminer {
            args: args,
            i: i
        }
    }
    
    pub fn peek(&self, i: usize) -> Option<String> {
        let idx = self.i + i;
        if idx >= self.args.len() {
            return None;
        }
        return Some(self.args[self.i + i].clone());
    }

    pub fn pop(&mut self) -> Option<String> {
        let ret_val = self.peek(0);
        match ret_val {
            Some(_) => self.i = self.i + 1,
            None => ()
        }
        return ret_val;
    }
}

impl <T> OptParser<T> {

    pub fn new(program_name: &str) -> OptParser<T> {
        return OptParser {
            program_name: program_name.into(),
            usage: HashMap::new(),
            opt_handlers: HashMap::new()
        };
    }

    pub fn print_usage(&self) {
        println!("Usage: {} [options] <args...>", self.program_name);
        println!();
        println!("Options:");
        for (opt, usage) in &self.usage {
            println!("  {}", opt);
            println!("  \t{}", usage)
        }
    }

    pub fn opt<A: Into<String>>(&mut self, arg: A, usage: A, handler: OptHandler<T>) -> &mut OptParser<T> {
        let arg = arg.into();
        self.usage.insert(arg.clone(), usage.into());
        self.opt_handlers.insert(arg, handler);
        return self;
    }

    pub fn parse_arg_examiner<'a>(&self, config: &mut T, args: &ArgExaminer<'a>) -> Result<Vec<String>, String> {
        let mut positional_args : Vec<String> = vec!();
        let mut i = 1;
        while args.peek(i) != None {
            match self.opt_handlers.get(&args.peek(i).unwrap()) {
                Some(handler) => {
                    let mut arg_examiner = ArgExaminer::new(args.args, i + 1);
                    handler(self, config, &mut arg_examiner)?;
                    i = arg_examiner.i
                },
                None => {
                    positional_args.push(args.peek(i).unwrap().clone());
                    i = i + 1
                },
            }
        }
        return Ok(positional_args);
    }

    pub fn parse(&self, config: &mut T, args: &Vec<String>) -> Result<Vec<String>, String> {
        return self.parse_arg_examiner(config, &ArgExaminer::new(args, 0));
    }
}