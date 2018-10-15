use ::Plugin;
use build::BuildError;
use build::Context;
use build::Module;
use build::Task;
use chrono;
use chrono::DateTime;
use dunce;
use failure::Error;
use path_util;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use timeago;

pub const CONAN_MODULE_TYPE: &'static str = "conan";
const CONAN_CLEAN_TASK: &'static str = "clean";
const CONAN_BUILD_TASK: &'static str = "build";
const CONAN_BUILD_DIRECTORY: &'static str = "build";
const CONAN_TEST_DIRECTORY: &'static str = "build";

pub const CONAN_NAME_PROPERTY: &'static str = "conan.name";
pub const CONAN_VERSION_PROPERTY: &'static str = "conan.version";
pub const CONAN_USER_PROPERTY: &'static str = "conan.user";
pub const CONAN_CHANNEL_PROPERTY: &'static str = "conan.channel";
pub const CONAN_DESCRIPTION_PROPERTY: &'static str = "conan.description";
pub const CONAN_REQUIRES_PROPERTY: &'static str = "conan.requires";
pub const CONAN_MODULE_PATH_PROPERTY: &'static str = "conan.module_path";
pub const CONAN_CONANFILE_PATH_PROPERTY: &'static str = "conan.conanfile";
pub const CONAN_ARTIFACT_TYPE: &'static str = "conan.artifact_type";
pub const CONAN_ARTIFACT_NAME: &'static str = "conan.artifact_name";

const CONAN_ARTIFACT_TYPE_LIB: &'static str = "lib";
const CONAN_ARTIFACT_TYPE_BIN: &'static str = "bin";

pub struct ConanPlugin {}

#[derive(Debug)]
struct CreateConfig {
        wd: PathBuf,
        conanfile: PathBuf,
        user: String,
        channel: String
}

#[derive(Debug)]
struct InstallConfig {
        wd: PathBuf,
        conanfile: PathBuf,
        install_folder: PathBuf
}

#[derive(Debug)]
struct BuildConfig {
        wd: PathBuf,
        conanfile: PathBuf,
        source_folder: PathBuf,
        install_folder: PathBuf,
        build_folder: PathBuf
}

#[derive(Debug)]
enum ConanConfig {
    Create(CreateConfig),
    Install(InstallConfig),
    Build(BuildConfig)
}

struct CleanTask {
    name: String,
    module_path: PathBuf, // Directory of the module.
    build_dir: PathBuf
}

struct BuildTask {
    name: String,
    module_path: PathBuf, // Directory of the module.
    build_dir: PathBuf,
    conanfile: PathBuf,
    artifact_type: String,
    user: String,
    channel: String,
    config: HashMap<String, String>
}

impl ConanPlugin {
    pub fn new() -> ConanPlugin {
        ConanPlugin {}
    }

    fn can_handle(&self, module: &Module) -> bool { module.types().contains(&CONAN_MODULE_TYPE.to_string()) }

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<(), Error> {

        for (k,v) in config {
            println!("{}={}", k, v);
        }
        println!();

        let is_defined = |var| -> Result<(), Error> {
            if !config.contains_key(var) {
                Err(BuildError::ConfigError(format!("{} must be defined.", var)))?
            } else {
                Ok(())
            }
        };

        let is_value = |var, val : &str| -> Result<(), Error> {
            let val = String::from(val);
            if config.contains_key(var) && config.get(var) == Some(&val) {
                Ok(())
            } else {
                Err(BuildError::ConfigError(format!("{} is not defined as {}.", var, val)))?
            }
        };

        let is_value_msg = |var : &str, val : &str, msg: String| -> Result<(), Error> {
            let val = String::from(val);
            if config.contains_key(var) && config.get(var) == Some(&val) {
                Ok(())
            } else {
                Err(BuildError::ConfigError(msg))?
            }
        };

        let is_one_of = |var, val : &[&str]| -> Result<(), Error> {
            val
            .iter()
            .map(|x| is_value(var, x))
            .fold(
                Err(Error::from(BuildError::ConfigError(format!("{} is not defined as any of {:?}", var, val)))),
                |acc, val| acc.or(val)
            )
        };

        return
            is_defined(CONAN_CONANFILE_PATH_PROPERTY).or(
                is_defined(CONAN_VERSION_PROPERTY)
                .and(is_one_of(CONAN_ARTIFACT_TYPE, &[CONAN_ARTIFACT_TYPE_BIN, CONAN_ARTIFACT_TYPE_LIB]))
                .and(
                    is_value(CONAN_ARTIFACT_TYPE, CONAN_ARTIFACT_TYPE_LIB)
                    .and(is_defined(CONAN_USER_PROPERTY))
                    .and(is_defined(CONAN_CHANNEL_PROPERTY))
                    .and(is_defined(CONAN_ARTIFACT_NAME))
                    .or(
                        is_value_msg(CONAN_ARTIFACT_TYPE, CONAN_ARTIFACT_TYPE_BIN,
                            format!(
                                "{} was defined as \"{}\" but all of {}, {}, and {} were not also defined.",
                                CONAN_ARTIFACT_TYPE,
                                CONAN_ARTIFACT_TYPE_LIB,
                                CONAN_USER_PROPERTY,
                                CONAN_CHANNEL_PROPERTY,
                                CONAN_ARTIFACT_NAME
                            )
                        )
                    )
                )
            );
    }
}

impl Plugin for ConanPlugin {

    fn get_documentation(&self) -> HashMap<String, String> {
        let mut doc = HashMap::new();

        doc.insert(CONAN_NAME_PROPERTY, "(Optional) The name of the Conan project. The default value is the module name.");
        doc.insert(CONAN_VERSION_PROPERTY, "(Required) The version of the Conan project.");
        doc.insert(CONAN_USER_PROPERTY, "(Required for artifacts of type \"lib\") The user of the Conan project.");
        doc.insert(CONAN_CHANNEL_PROPERTY, "(Required for artifacts of type \"lib\") The channel of the Conan project.");
        doc.insert(CONAN_ARTIFACT_TYPE, "(Required) The type of artifact this module produces. Possible values are \"lib\" for a library and \"bin\" for a binary.");
        doc.insert(CONAN_ARTIFACT_NAME, "(Required for artifacts of type \"lib\") The name of the shared library/executable this module produces. This does not include the file extension or the \"lib\" prefix on Unix systems.");
        doc.insert(CONAN_DESCRIPTION_PROPERTY, "(Optional) The description of the Conan project.");
        doc.insert(
            CONAN_REQUIRES_PROPERTY, indoc!(
            "(Optional) The dependencies of the Conan project as a list. \
            If unspecified then there are no dependencies.")
            );
        doc.insert(
            CONAN_MODULE_PATH_PROPERTY, indoc!(
            "(Optional) This is the path to the module root directory.

            If this is not specified then it is automatically determined based on the module name.
            
            If this is specified then it may be relative or absolute. \
            Relative paths are relative to the directory containing the root build file."
            )
        );
        doc.insert(
            CONAN_CONANFILE_PATH_PROPERTY, indoc!(
            "(Optional) The path to the conanfile for this module. This path is relative to the module root directory.
            
            If it is not specified then a default conanfile will be used.")
        );

        return doc
            .iter()
            .map(|(k,v)| { (k.to_string(), v.to_string() )})
            .fold(HashMap::new(), |mut acc, (k,v)| {
                acc.insert(k,v);
                acc
            });
    }

    fn configure(&self, context: &mut Context) -> Result<(), Error> {
        let mut build_tasks : HashMap<String, Box<Task>> = HashMap::new();
        let mut clean_tasks : HashMap<String, Box<Task>> = HashMap::new();
        let mut handled_modules : HashSet<String> = HashSet::new();

        let build_task_name = |module_name: &String| {format!("{}:build", module_name)};
        let clean_task_name = |module_name: &String| {format!("{}:clean", module_name)};

        for (name, module) in context.modules_mut() {
            let module = module.as_ref();

            if self.can_handle(module) && self.validate_config(module.config())? == () {
                trace!("Conan plugin configuring: {}", module.name());

                build_tasks.insert(build_task_name(name), Box::new(BuildTask::new(module)?));
                clean_tasks.insert(clean_task_name(name), Box::new(CleanTask::new(module)?));

                handled_modules.insert(name.clone());

            } else {
                trace!("Conan plugin skipping: {}", module.name())
            }
        }

        for (name, task) in build_tasks {
            context.add_task(&name, task);

            debug!("Task {} depends on {}.", CONAN_BUILD_TASK, name);
            context.task_mut(CONAN_BUILD_TASK)?.depends_on(&name)?;
        }

        for (name, task) in clean_tasks {
            context.add_task(&name, task);

            debug!("Task {} depends on {}.", CONAN_CLEAN_TASK, name);
            context.task_mut(CONAN_CLEAN_TASK)?.depends_on(&name)?;
        }

        // Set dependency relationships.
        for module in &handled_modules {
            for dep in context.get_module_deps(module)?.clone() {
                if handled_modules.contains(&dep) {
                    let build_task = build_task_name(module);
                    let dep_task = build_task_name(&dep);

                    debug!("Task {} depends on {}.", build_task, dep_task);
                    context.task_mut(&build_task)?.depends_on(&dep_task)?;
                }
            }
        }

        Ok(())
    }
}

fn get_required_config(m: &Module, key: &str) -> Result<String, Error> {
    match m.config().get(&key.to_string()) {
        Some(value) => Ok(value.clone()),
        None => Err(BuildError::ConfigError(format!("{} is not defined.", key)))?
    }
}

///
/// Get the artifact type if it is necessary or use the default "lib" if a conanfile is specified.
/// 
fn get_artifact_type(m: &Module) -> Result<String, Error> {
    if let Some(v) = m.config().get(&CONAN_ARTIFACT_TYPE.to_string()) {
        Ok(v.clone())
    } else if let Some(_) = m.config().get(&CONAN_CONANFILE_PATH_PROPERTY.to_string()) {
        Ok(CONAN_ARTIFACT_TYPE_LIB.to_string())
    } else {
        Err(BuildError::ConfigError(format!("{} must be defined.", CONAN_ARTIFACT_TYPE)))?
    }
}

impl CleanTask {
    fn new(m: &Module) -> Result<CleanTask, Error> {
        // TODO: This should not be duplicated code.
        let module_path = match dunce::canonicalize(&m.module_dir()) { Ok(a) => a, Err(e) => return Err(BuildError::IOError(format!("Error canonicalizing module path {}: {}", m.module_dir().display(), e.to_string())))? };
        let build_dir = path_util::PathBuilder::from(&module_path).push(CONAN_BUILD_DIRECTORY).build();

        Ok(
            CleanTask {
                name: String::from(CONAN_CLEAN_TASK),
                module_path: module_path,
                build_dir: build_dir
            }
        )
    }
}

impl Task for CleanTask {

    fn needs_execution(&self) -> bool {
        true
    }

    fn execute(&self) -> Result<(), Error> {
        if !self.build_dir.exists() {
            return Ok(());
        }

        match fs::remove_dir_all(&self.build_dir) {
            Ok(_) => Ok(()),
            Err(e) => Err(BuildError::IOError(format!("Unable to remove directory {:?}: {}", &self.build_dir, e)))?
        }
    }

}

impl BuildTask {

    fn new(m: &Module) -> Result<BuildTask, Error> {
        let mut config = m.config().clone();

        let module_path = match dunce::canonicalize(&m.module_dir()) { Ok(a) => a, Err(e) => Err(BuildError::IOError(format!("Error canonicalizing module path {}: {}", m.module_dir().display(), e.to_string())))? };
        let build_dir = path_util::PathBuilder::from(&module_path).push(CONAN_BUILD_DIRECTORY).build();
        let conanfile = match config.get(CONAN_CONANFILE_PATH_PROPERTY) {
            Some(p) => PathBuf::from(p),
            None => build_dir.join("conanfile.py")
        };

        if !config.contains_key(CONAN_NAME_PROPERTY) {
            config.insert(CONAN_NAME_PROPERTY.to_string(), m.name().clone());
        }
        if !config.contains_key(CONAN_MODULE_PATH_PROPERTY) {
            let module_path_string = match module_path.to_str() { Some(a) => a.to_string(), None => Err(BuildError::IOError(format!("Unable to convert {} to utf-8 string.", module_path.display())))? };
            config.insert(CONAN_MODULE_PATH_PROPERTY.to_string(), module_path_string);
        }
        if !config.contains_key(CONAN_REQUIRES_PROPERTY) {
            config.insert(CONAN_REQUIRES_PROPERTY.to_string(), String::new());
        }

        Ok(
            BuildTask {
                name : CONAN_BUILD_TASK.to_string(),
                module_path: module_path.clone(),
                build_dir: build_dir,
                conanfile: conanfile,
                artifact_type: get_artifact_type(&m)?,
                user: get_required_config(&m, CONAN_USER_PROPERTY)?,
                channel: get_required_config(&m, CONAN_CHANNEL_PROPERTY)?,
                config: config
            }
        )
    }

    fn get_timestamp_file_path(&self) -> PathBuf {
        let mut timestamp_path = PathBuf::from(&self.module_path);
        timestamp_path.push("build");
        timestamp_path.push("timestamp");
        timestamp_path
    }

    fn set_timestamp_file(&self) -> Result<(),Error> {
        let timestamp_path = self.get_timestamp_file_path();
        if !timestamp_path.parent().unwrap().exists() {
            if let Err(e) = fs::create_dir(timestamp_path.parent().unwrap()) {
                warn!("Error creating timestamp file {}: {}", self.get_timestamp_file_path().display(), e);
                Err(BuildError::IOError(format!("Error creating timestamp file {}: {}", self.get_timestamp_file_path().display(), e)))?
            }
        }
        if let Err(e) = fs::File::create(self.get_timestamp_file_path()) {
            warn!("Error creating timestamp file {}: {}", self.get_timestamp_file_path().display(), e);
            Err(BuildError::IOError(format!("Error creating timestamp file {}: {}", self.get_timestamp_file_path().display(), e)))?
        }
        Ok(())
    }

    fn ensure_clean_build_dir(&self) -> Result<(), Error> {
        if self.build_dir.exists() && self.build_dir.is_dir() {
            if let Err(e) = fs::remove_dir_all(&self.build_dir) {
                Err(BuildError::IOError(format!("Error deleting build directory at {:?}: {}", self.build_dir, e)))?
            }
        }
        if self.build_dir.exists() && self.build_dir.is_file() {
            if let Err(e) = fs::remove_file(&self.build_dir) {
                Err(BuildError::IOError(format!("Error deleting build directory (which as actually a file) at {:?}: {}", self.build_dir, e)))?
            }
        }
        if let Err(e) = fs::create_dir_all(&self.build_dir) {
            Err(BuildError::IOError(format!("Error creating build directory at {:?}: {}", self.build_dir, e)))?
        }
        Ok(())
    }

    fn write_build_scripts(&self) -> Result<(), Error> {
        let mut conanfile = String::from(include_str!("scripts/conan/conanfile.py"));
        let mut cmakelists = String::from(include_str!("scripts/conan/CMakeLists.txt"));

        // This could be much more efficient but it isnt a bottleneck.
        for (k,v) in &self.config {
            // This ends up replacing ("${key}" with the value)
            let p = format!("${}{}{}", "{", k, "}");
            debug!("Replacing: {}", p);
            conanfile = conanfile.replace(p.as_str(), v);
        }
        for (k,v) in &self.config {
            cmakelists = cmakelists.replace(format!("${}{}{}", "{", k, "}").as_str(), v);
        }

        // Ensure path is a directory.
        {
            let p = self.build_dir.as_path();
            if !p.exists() {
                if let Err(e) = fs::create_dir_all(p) { Err(BuildError::IOError(e.to_string()))? }
            } else if !p.is_dir() {
                Err(BuildError::IOError(format!("Build directory {} is not a directory.", p.display())))?
            }
        }

        // Create module configs.
        {
            // Use the specified conan file if specified.
            let conanfile_path = path_util::PathBuilder::from(&self.build_dir).push("conanfile.py").build();
            if self.conanfile.exists() {
                debug!("Copying: {:?} from {:?}", conanfile_path, self.conanfile);
                if let Err(e) = fs::copy(&self.conanfile, conanfile_path) { Err(BuildError::IOError(format!("Unable to write {:?}: {}", self.conanfile, e)))? };
            } else {
                debug!("Writing: {:?}", conanfile_path);
                if let Err(e) = fs::write(&conanfile_path, conanfile) { Err(BuildError::IOError(format!("Unable to write {:?}: {}", conanfile_path, e)))? }
            }

            let cmakelists_path = path_util::PathBuilder::from(&self.build_dir).push("CMakeLists.txt").build();
            debug!("Writing: {:?}", cmakelists_path);
            if let Err(e) = fs::write(cmakelists_path, cmakelists) { Err(BuildError::IOError(e.to_string()))? }
        }

        Ok(())
    }
}

impl Task for BuildTask {

    fn needs_execution(&self) -> bool {
        use walkdir::WalkDir;

        let timestamp_path = self.get_timestamp_file_path();
        trace!("Looking for timestamp file in: {:?}", timestamp_path);

        if !timestamp_path.exists() {
            debug!("No timestamp found at {}. {} needs rebuild.", self.name, timestamp_path.display());
            return true;
        }

        let metadata = fs::metadata(timestamp_path.clone());
        if let Err(e) = metadata {
            warn!("Error accessing metadata for {}: {}", timestamp_path.display(), e);
            return true
        }

        let modified = metadata.unwrap().modified();
        if let Err(e) = modified {
            warn!("Error accessing modified time for {}: {}", timestamp_path.display(), e);
            return true
        }

        let timestamp = modified.unwrap();

        for entry in WalkDir::new(&self.module_path).into_iter().filter_entry(
            |e| {
                !(e.depth() == 1 && e.file_name() == CONAN_BUILD_DIRECTORY) && // Ignore the build directory.
                !(e.depth() == 2 && e.file_name() == CONAN_BUILD_DIRECTORY && e.path().parent().unwrap().file_name().unwrap() == CONAN_TEST_DIRECTORY) // Ignore the test build directory.
            }
        ) {
            if let Err(e) = entry {
                warn!("Error accessing path: {}", e);
                continue
            }

            trace!("Examining: {}", entry.as_ref().unwrap().path().display());

            let metadata = fs::metadata(entry.as_ref().unwrap().path());
            if let Err(e) = metadata {
                warn!("Error accessing metadata for {}: {}", entry.unwrap().path().display(), e);
                continue
            }

            let modified = metadata.unwrap().modified();
            if let Err(e) = modified {
                warn!("Error accessing modified time for {}: {}", entry.unwrap().path().display(), e);
                continue
            }

            let modified = modified.unwrap();
            if let Ok(t) = modified.duration_since(timestamp) {
                debug!(
                    "File {} was modified on {}, {} since timestamp {}.",
                    entry.unwrap().path().display(),
                    DateTime::<chrono::offset::Local>::from(modified),
                    timeago::Formatter::new().convert(t),
                    DateTime::<chrono::offset::Local>::from(timestamp)
                );
                return true;
            }

        }

        false
    }

    fn execute(&self) -> Result<(), Error> {
        self.ensure_clean_build_dir()?;
        self.write_build_scripts()?;

        let artifact_type = &self.artifact_type;
        let user = &self.user;
        let channel = &self.channel;

        debug!("Computed Configuration: {:#?}", self.config);

        let wd = match dunce::canonicalize(&self.module_path) {
            Ok(dir) => dir,
            Err(e) => {
                Err(BuildError::IOError(format!("Error canonicalizing path {}: {}", self.module_path.display(), e.to_string())))?
            }
        };

        if artifact_type == CONAN_ARTIFACT_TYPE_LIB {

            let config = ConanConfig::Install(InstallConfig {
                wd: wd.clone(),
                conanfile: self.conanfile.clone(),
                install_folder: self.build_dir.clone()
            });
            conan(config)?;

            let config = ConanConfig::Create(CreateConfig {
                wd: wd,
                conanfile: self.conanfile.clone(),
                user: user.clone(),
                channel: channel.clone()
            });
            conan(config)?;
            self.set_timestamp_file()

        } else if artifact_type == CONAN_ARTIFACT_TYPE_BIN {

            let config = ConanConfig::Install(InstallConfig {
                wd: wd.clone(),
                conanfile: self.conanfile.clone(),
                install_folder: self.build_dir.clone()
            });
            conan(config)?;

            let config = ConanConfig::Build(BuildConfig {
                wd: wd.clone(),
                conanfile: self.conanfile.clone(),
                source_folder: self.build_dir.clone(),
                install_folder: self.build_dir.clone(),
                build_folder: self.build_dir.clone()
            });
            conan(config)?;
            self.set_timestamp_file()

        } else {
            Err(BuildError::IOError(format!("Unknown artifact type {}.", artifact_type)))?
        }
    }
}

fn conan(config: ConanConfig) -> Result<(), Error> {

    debug!("Config: {:#?}", config);

    let mut cmd = Command::new("conan");
    match config {
        ConanConfig::Create(config) => {
            cmd
                .arg("create")
                .arg(config.conanfile)
                .arg(format!("{}/{}", config.user, config.channel))
                .current_dir(config.wd)
        },
        ConanConfig::Install(config) => {
            cmd
                .arg("install")
                .arg(config.conanfile)
                .arg("--build=missing")
                .arg(format!("--install-folder={}", config.install_folder.display()))
                .current_dir(config.wd)
        },
        ConanConfig::Build(config) => {
            cmd
                .arg("build")
                .arg(config.conanfile)
                .arg(format!("--source-folder={}", config.source_folder.display()))
                .arg(format!("--install-folder={}", config.install_folder.display()))
                .arg(format!("--build-folder={}", config.build_folder.display()))
                .current_dir(config.wd)
        }
    };
    
    debug!("Command: {:?}", cmd);
    
    let mut p = match cmd.spawn() {
        Ok(p) => p,
        Err(msg) => Err(BuildError::ExecutionError(msg.to_string()))?
    };

    match p.wait() {
        Ok(e) => {
            debug!("Status: {:?}", e);
            if !e.success() {
                Err(BuildError::ExecutionError(format!("Process failed: {}", e)))?
            }
            return Ok(())
        },
        Err(e) => {
            error!("Error: {:?}", e);
            Err(BuildError::ExecutionError(format!("Error executing process: {}", e)))?
        }
    }
}