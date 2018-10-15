use ::Context;
use ::Task;
use failure::Error;
use std::collections::HashMap;

pub struct Executor<'ctx> {
    context: &'ctx Context,
    state: HashMap<String, ExecutionItem<'ctx>>
}

struct ExecutionItem<'ctx> {
    task: &'ctx Task,
    state: ExecutionState
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExecutionState {
    NotExecuted,
    Done
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionBehavior {
    Skipped,
    Executed
}

#[derive(Debug, Fail)]
enum ExecutionError {
    #[fail(display = "dependency not found: {}", name)]
    UnknownDependency{ name: String }
}

impl<'ctx> Executor<'ctx> {
    pub fn new(ctx: &Context) -> Executor {
        Executor {
            context: ctx,
            state: HashMap::new()
        }
    }

    fn get_execution(&mut self, task: &str) -> Result<&mut ExecutionItem<'ctx>, Error> {
        let task = task.to_string();
        if !self.state.contains_key(&task) {
            self.state.insert(
                task.to_string(), 
                ExecutionItem {
                    task: self.context.get_task(&task)?,
                    state: ExecutionState::NotExecuted
                }
            );
        }
        match self.state.get_mut(&task) {
            Some(t) => Ok(t),
            None => Err(ExecutionError::UnknownDependency{name: task})?
        }
    }

    fn execute_helper(&mut self, task: &str) -> Result<ExecutionBehavior, Error> {
        let mut dep_behavior = ExecutionBehavior::Skipped;
        let mut ret_behavior = ExecutionBehavior::Skipped;

        let deps = self.context.get_task_deps(task)?.clone();
        let task = task.to_string();

        debug!("Considering: {} with dependencies: {:?}", task, deps);

        for dep in deps {
            let behavior = self.execute_helper(&dep)?;

            // If any of the dependencies execute, then we consider the behavior to be "executed."
            if dep_behavior == ExecutionBehavior::Skipped {
                dep_behavior = behavior
            }
        }

        let exec_item = self.get_execution(&task)?;
        if exec_item.state() == ExecutionState::NotExecuted {
            // If any dependency executed or if we need execution.
            if dep_behavior == ExecutionBehavior::Executed || exec_item.task().needs_execution() {
                info!("Executing: {}", task);
                exec_item.task().execute()?;
                ret_behavior = ExecutionBehavior::Executed;
            } else {
                info!("Skipping: {}", task);
            }
            exec_item.set_done();
        }

        debug!("Considering: {}. Behavior: {:?}", task, ret_behavior);

        Ok(ret_behavior)
    }

    pub fn execute(&mut self, task: &str) -> Result<(), Error> {
        self.execute_helper(task)?;
        Ok(())
    }
}

impl<'a> ExecutionItem<'a> {

    fn task(&self) -> &Task {
        self.task
    }

    fn state(&self) -> ExecutionState {
        self.state
    }

    fn set_done(&mut self) {
        self.state = ExecutionState::Done
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::collections::HashMap;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::ops::DerefMut;
    use std::rc::Rc;

    struct TestTask {
        flag: Rc<RefCell<bool>>
    }

    impl TestTask {
        fn new(flag: Rc<RefCell<bool>>) -> TestTask {
            TestTask {
                flag: flag
            }
        }
    }

    impl Task for TestTask {
        fn needs_execution(&self) -> bool {
            true
        }
        fn execute(&self) -> Result<(), Error> {
            *self.flag.borrow_mut().deref_mut() = true;
            Ok(())
        }
    }

    fn flag() -> Rc<RefCell<bool>> {
        Rc::new(RefCell::new(false))
    }

    #[test]
    fn execution() {
        let flag0 = flag();
        let flag1 = flag();
        let flag2 = flag();
        let flag3 = flag();
        let flag4 = flag();

        let mut ctx = Context::new();
        ctx.add_task("task0", Box::new(TestTask::new(flag0.clone())));
        ctx.add_task("task1", Box::new(TestTask::new(flag1.clone())));
        ctx.add_task("task2", Box::new(TestTask::new(flag2.clone())));
        ctx.add_task("task3", Box::new(TestTask::new(flag3.clone())));
        ctx.add_task("task4", Box::new(TestTask::new(flag4.clone())));

        ctx.task_mut("task0").unwrap().depends_on("task1").unwrap();
        ctx.task_mut("task1").unwrap().depends_on("task2").unwrap();
        ctx.task_mut("task1").unwrap().depends_on("task3").unwrap();

        let mut executor = Executor::new(&ctx);
        executor.execute("task0").unwrap();

        assert_eq!(*(*flag0).borrow(), true);
        assert_eq!(*(*flag1).borrow(), true);
        assert_eq!(*(*flag2).borrow(), true);
        assert_eq!(*(*flag3).borrow(), true);
        assert_eq!(*(*flag4).borrow(), false);
    }
}