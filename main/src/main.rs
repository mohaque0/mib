extern crate build;
extern crate config;
extern crate failure;
extern crate plugin;
extern crate simple_logger;

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

mod opt;

use std::path::PathBuf;
use std::env;
use std::fs;

pub const PLUGIN_DIR_NAME: &'static str = "plugins";
pub const DEFAULT_BUILD_TASK_NAME: &'static str = "build";
pub const DEFAULT_CLEAN_TASK_NAME: &'static str = "clean";

#[derive(Debug)]
struct CmdLineOptions {
    config_dir: Option<PathBuf>,
    buildfile: Option<PathBuf>,
    root_dir: Option<PathBuf>,
    target: Option<String>,
    display_help: Option<Option<String>>, // Optional value specifies a module whose help must be displayed.
    log_level: log::Level
}

impl CmdLineOptions {
    fn new() -> CmdLineOptions {
        return CmdLineOptions {
            config_dir: None,
            buildfile: None,
            root_dir: None,
            target: None,
            display_help: None,
            log_level: log::Level::Info
        }
    }
}

struct EmptyTask;
impl build::Task for EmptyTask {
    fn needs_execution(&self) -> bool {
        false
    }
    fn execute(&self) -> Result<(), failure::Error> {
        Ok(())
    }
}

///
/// Search this path or the working directory if not specified and all parent paths for a build file.
/// 
fn get_default_build_file(root: &Option<PathBuf>) -> Option<PathBuf> {
    let default_names = ["build.yml", "build.yaml"];

    let path = match root.as_ref() {
        Some(path) => path.clone(),
        None => PathBuf::from(".")
    };

    for name in default_names.iter() {
        let mut file = path.clone();
        file.push(name);
        if file.exists() { return Some(file); }
    }

    return None;
}

fn get_default_config_dir() -> Result<PathBuf, String> {
    let path_buf = match env::home_dir() {
        Some(p) => { let mut p = p.clone(); p.push(".builder"); p },
        None => return Err("No config directory specified and home directory is unknown.".to_string())
    };

    // Ensure path is a directory.
    {
        let p = path_buf.as_path();
        if !p.exists() {
            if let Err(e) = fs::create_dir_all(p) { return Err(e.to_string()) }
        } else if !p.is_dir() {
            return Err(format!("Config directory {} is not a directory.", p.display()))
        }
    }

    return Ok(path_buf);
}

fn main() {
    let args : Vec<String> = std::env::args().collect();

    let mut opt_parser : opt::OptParser<CmdLineOptions> = opt::OptParser::new(&args[0]);
    opt_parser
        .opt("--debug", "Enable debug logging.",
            |_, cmdline_options, _| {
                cmdline_options.log_level = log::Level::Debug;
                Ok(())
            }
        )
        .opt("--trace", "Enable trace logging.",
            |_, cmdline_options, _| {
                cmdline_options.log_level = log::Level::Trace;
                Ok(())
            }
        )
        .opt("--config", "Path to the builder config directory. (Default: ~/.builder)",
            |_, cmdline_options, args| {
                match args.pop() {
                    Some(p) => {
                        cmdline_options.config_dir = Some(PathBuf::from(p));
                        return Ok(())
                    },
                    None => {
                        return Err("\"--config\" expects a path.".to_string())
                    }
                }
            }
        )
        .opt("--buildfile", "Path to the build file or directory containing the build file.",
            |_, cmdline_options, args| {
                match args.pop() {
                    Some(s) => {
                        let mut p = PathBuf::from(s);
                        if p.is_dir() {
                            p.push("build.yml")
                        }
                        cmdline_options.buildfile = Some(p);
                        return Ok(())
                    },
                    None => {
                        return Err("\"--buildfile\" expects a path.".to_string())
                    }
                }
            }
        )
        .opt("--root", "Path to the root directory of the modules (usually also containing the build file.)",
            |_, cmdline_options, args| {
                match args.pop() {
                    Some(s) => {
                        let mut p = PathBuf::from(s);
                        if !p.is_dir() {
                            return Err(format!("Root path {} is not a directory.", p.display()))
                        }
                        cmdline_options.root_dir = Some(p);
                        return Ok(())
                    },
                    None => {
                        return Err("\"--root\" expects a path.".to_string())
                    }
                }
            }
        )
        .opt("--help", "Print this usage.",
            |_, cmdline_options, args| {
                cmdline_options.display_help = Some(args.pop());
                Ok(())
            }
        );

    // Parse options.
    let mut cmdline_options = CmdLineOptions::new();
    let command_args = match opt_parser.parse(&mut cmdline_options, &args) {
        Ok(args) => args,
        Err(e) => {
            error!("Error: {}", e);
            opt_parser.print_usage();
            return;
        }
    };

    simple_logger::init_with_level(cmdline_options.log_level).unwrap();

    if command_args.len() > 0 {
        cmdline_options.target = Some(command_args[0].clone());
    }

    //
    // Display usage if requested.
    //
    if Some(None) == cmdline_options.display_help {
        opt_parser.print_usage();
        return;
    }

    // We need a context to display usage of individual builders.
    if cmdline_options.config_dir == None {
        cmdline_options.config_dir = Some(get_default_config_dir().expect("No config dir specified and no default found."))
    }

    // Load plugins.
    let mut plugins_dir = cmdline_options.config_dir.as_ref().unwrap().clone();
    plugins_dir.push(PLUGIN_DIR_NAME);
    let plugins = plugin::Framework::new(&plugins_dir);

    let plugins = match plugins.get_plugins() {
        Ok(p) => p,
        Err(e) => {
            error!("Error retrieving plugins: {:?}", e);
            vec!()
        }
    };

    // Display help for plugins.
    if let Some(_) = cmdline_options.display_help {
        for plugin in &plugins {
            for (k,v) in plugin.get_documentation() {
                println!("{}", k);
                println!("\t{}", v.replace("\n", "\n\t"));
                println!()
            }
        }
    }

    //
    // Handle other options.
    //

    // Set project root dir.
    if &cmdline_options.buildfile == &None {
        cmdline_options.buildfile = get_default_build_file(&cmdline_options.root_dir)
    }
    if &cmdline_options.buildfile == &None {
        error!("Unable to find buildfile.");
        return;
    }

    // Create context.
    let mut context = build::Context::new();
    context.add_task(DEFAULT_BUILD_TASK_NAME, Box::new(EmptyTask));
    context.add_task(DEFAULT_CLEAN_TASK_NAME, Box::new(EmptyTask));

    // Parse build file.
    let buildfile = cmdline_options.buildfile.as_ref().unwrap().as_path();
    if let Err(e) = match buildfile.extension() {
        Some(osstr) => {
            if      osstr == "yml"  { config::parse_file(&mut context, buildfile) }
            else if osstr == "yaml" { config::parse_file(&mut context, buildfile) }
            else { panic!("Unable to determine format from buildfile extension.".to_string()) }
        },
        None => panic!("Buildfile has no extension. Unable to determine format.".to_string())
    } {
        error!("Error parsing document {}: {}", buildfile.display(), e);
    }

    // Generate tasks.
    for plugin in plugins {
        if let Err(e) = plugin.configure(&mut context) {
            error!("Error applying plugin: {}", e)
        }
    }

    // Execute build.
    let mut executor = build::Executor::new(&context);
    if let Some(t) = cmdline_options.target {
        if let Err(e) = executor.execute(&t) {
            error!("Error executing task: {}", e);
            return;
        }
    } else {
        if let Err(e) = executor.execute(DEFAULT_BUILD_TASK_NAME) {
            error!("Error building project: {}", e);
            return;
        }
    }
}
