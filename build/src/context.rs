use ::Module;
use ::Task;
use deptree::DepTree;
use error::BuildError;
use failure::Error;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Context {
    modules: DepTree<Module>,
    tasks: DepTree<Task>
}

pub struct TaskRef<'a> {
    name: String,
    context: &'a mut Context
}

pub struct ModuleRef<'a> {
    name: String,
    context: &'a mut Context
}

impl Context {
    pub fn new() -> Context {
        Context {
            modules: DepTree::new(),
            tasks: DepTree::new()
        }
    }

    pub fn add_module(&mut self, name: &str, module: Module) {
        self.modules.insert(name, module)
    }

    pub fn add_task(&mut self, name: &str, task: Box<Task>) {
        self.tasks.insert_box(name, task)
    }

    pub fn get_module_deps(&self, name: &str) -> Result<&HashSet<String>, Error> {
        self.modules.get_deps(name)
    }

    pub fn get_task(&self, name: &str) -> Result<&Task, Error> {
        Ok(self.tasks.get_item(name).ok_or(BuildError::NoSuchTask(name.to_string()))?)
    }

    pub fn get_task_deps(&self, name: &str) -> Result<&HashSet<String>, Error> {
        self.tasks.get_deps(name)
    }

    pub fn modules_mut(&mut self) -> &HashMap<String, Box<Module>> {
        self.modules.items_mut()
    }

    pub fn module(&mut self, name: &str) -> Result<ModuleRef, Error> {
        self.modules.get_item(name).ok_or(BuildError::NoSuchModule(name.to_string()))?;
        let name = String::from(name);
        Ok(
            ModuleRef {
                name: name,
                context: self
            }
        )
    }

    pub fn task_mut(&mut self, name: &str) -> Result<TaskRef, Error> {
        self.tasks.get_item(name).ok_or(BuildError::NoSuchTask(name.to_string()))?;
        let name = String::from(name);
        Ok(
            TaskRef {
                name: name,
                context: self
            }
        )
    }
}

impl <'a> TaskRef<'a> {
    pub fn depends_on(&mut self, name: &str) -> Result<(), Error> {
        self.context.tasks.set_dependency(&self.name, name)
    }
}

impl <'a> ModuleRef<'a> {
    pub fn depends_on(&mut self, name: &str) -> Result<(), Error> {
        self.context.modules.set_dependency(&self.name, name)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    struct TestTask {}

    impl TestTask {
        fn new() -> TestTask {
            TestTask {}
        }
    }

    impl Task for TestTask {
        fn needs_execution(&self) -> bool {
            true
        }
        fn execute(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[test]
    fn task_dependency() {
        let mut ctx = Context::new();
        ctx.add_task("task0", Box::new(TestTask::new()));
        ctx.add_task("task1", Box::new(TestTask::new()));
        ctx.task_mut("task1").unwrap().depends_on("task0").unwrap();

        let mut expected = HashSet::<String>::new();
        expected.insert("task0".to_string());

        assert_eq!(ctx.get_task_deps("task1").unwrap(), &expected);
    }

    #[test]
    fn module_dependency() {
        let mut ctx = Context::new();
        ctx.add_module("m0", Module::new(
            &"m0".to_string(),
            PathBuf::new(),
            PathBuf::new(),
            HashSet::new(),
            HashMap::new()
        ));
        ctx.add_module("m1", Module::new(
            &"m0".to_string(),
            PathBuf::new(),
            PathBuf::new(),
            HashSet::new(),
            HashMap::new()
        ));
        ctx.module("m1").unwrap().depends_on("m0").unwrap();

        let mut expected = HashSet::<String>::new();
        expected.insert("m0".to_string());

        assert_eq!(ctx.get_module_deps("m1").unwrap(), &expected);
    }
}