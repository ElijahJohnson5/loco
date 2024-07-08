//! # Task Management Module
//!
//! This module defines the task management framework used to manage and execute
//! tasks in a web server application.
use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::{app::AppContext, errors::Error, Result};

/// Struct representing a collection of task arguments.
#[derive(Default, Debug)]
pub struct Vars {
    /// A list of cli arguments.
    pub cli: BTreeMap<String, String>,
}

impl Vars {
    /// Create [`Vars`] instance from cli arguments.
    ///
    /// # Arguments
    ///
    /// * `key` - A string representing the key.
    /// * `value` - A string representing the value.
    ///
    /// # Example
    ///
    /// ```
    /// use loco_rs::task::Vars;
    ///
    /// let args = vec![("key1".to_string(), "value".to_string())];
    /// let vars = Vars::from_cli_args(args);
    /// ```
    #[must_use]
    pub fn from_cli_args(args: Vec<(String, String)>) -> Self {
        Self {
            cli: args.into_iter().collect(),
        }
    }

    /// Retrieves the value associated with the given key from the `cli` list.
    ///
    /// # Errors
    ///
    /// Returns an error if the key does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use loco_rs::task::Vars;
    ///
    /// let args = vec![("key1".to_string(), "value".to_string())];
    /// let vars = Vars::from_cli_args(args);
    ///
    /// assert!(vars.cli_arg("key1").is_ok());
    /// assert!(vars.cli_arg("not-exists").is_err());
    /// ```
    pub fn cli_arg(&self, key: &str) -> Result<&String> {
        self.cli
            .get(key)
            .ok_or(Error::Message(format!("the argument {key} does not exist")))
    }
}

/// Information about a task, including its name and details.
#[allow(clippy::module_name_repetitions)]
pub struct TaskInfo {
    pub name: String,
    pub detail: String,
}

/// A trait defining the behavior of a task.
#[async_trait]
pub trait Task<T: Send + Sync + Clone>: Send + Sync {
    /// Get information about the task.
    fn task(&self) -> TaskInfo;
    /// Execute the task with the provided application context and variables.
    async fn run(&self, app_context: &AppContext<T>, vars: &Vars) -> Result<()>;
}

/// Managing and running tasks.
pub struct Tasks<T: Send + Sync + Clone> {
    registry: BTreeMap<String, Box<dyn Task<T>>>,
}

impl<T: Send + Sync + Clone> Default for Tasks<T> {
    fn default() -> Self {
        Self {
            registry: Default::default(),
        }
    }
}

impl<T: Send + Sync + Clone> Tasks<T> {
    /// List all registered tasks with their information.
    #[must_use]
    pub fn list(&self) -> Vec<TaskInfo> {
        self.registry.values().map(|t| t.task()).collect::<Vec<_>>()
    }

    /// Run a registered task by name with provided variables.
    ///
    /// # Errors
    ///
    /// Returns a [`Result`] if an task finished with error. mostly if the given
    /// task is not found or an error to run the task.s
    pub async fn run(&self, app_context: &AppContext<T>, task: &str, vars: &Vars) -> Result<()> {
        let task = self
            .registry
            .get(task)
            .ok_or_else(|| Error::TaskNotFound(task.to_string()))?;
        task.run(app_context, vars).await?;
        Ok(())
    }

    /// Register a new task to the registry.
    pub fn register(&mut self, task: impl Task<T> + 'static) {
        let name = task.task().name;
        self.registry.insert(name, Box::new(task));
    }
}
