#![allow(dead_code)]

use crate::tasks_builder_error::TasksBuilderError;
use moon_args::split_args;
use moon_common::{color, Id};
use moon_config::{
    InheritedTasksConfig, InputPath, PlatformType, ProjectConfig,
    ProjectWorkspaceInheritedTasksConfig, TaskCommandArgs, TaskConfig, TaskMergeStrategy,
    TaskOutputStyle, TaskType, ToolchainConfig,
};
use moon_target::Target;
use moon_task::{Task, TaskOptions};
use rustc_hash::{FxHashMap, FxHashSet};
use starbase_events::{Emitter, Event};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::path::Path;
use tracing::trace;

struct ConfigChain<'proj> {
    config: &'proj TaskConfig,
    inherited: bool,
}

// This is a standalone function as recursive closures are not possible!
fn legacy_extract_config<'builder, 'proj>(
    configs: &'builder mut Vec<ConfigChain<'proj>>,
    task_id: &'builder Id,
    tasks: &'builder FxHashMap<&'proj Id, &'proj TaskConfig>,
    extendable_tasks: &'builder FxHashMap<&'proj Id, &'proj TaskConfig>,
    inherited: bool,
    extended: bool,
) -> miette::Result<()> {
    let config = if extended {
        extendable_tasks.get(task_id)
    } else {
        tasks.get(task_id)
    };

    if let Some(config) = config {
        if let Some(extend_task_id) = &config.extends {
            if !extendable_tasks.contains_key(extend_task_id) {
                return Err(TasksBuilderError::UnknownExtendsSource {
                    source_id: task_id.to_owned(),
                    target_id: extend_task_id.to_owned(),
                }
                .into());
            }

            legacy_extract_config(
                configs,
                extend_task_id,
                tasks,
                extendable_tasks,
                inherited,
                true,
            )?;
        }

        configs.push(ConfigChain { config, inherited });
    }

    Ok(())
}

fn extract_config<'builder, 'proj>(
    task_id: &'builder Id,
    local_tasks: &'builder FxHashMap<&'proj Id, &'proj TaskConfig>,
    global_tasks: &'builder FxHashMap<&'proj Id, &'proj TaskConfig>,
) -> miette::Result<Vec<ConfigChain<'proj>>> {
    let mut stack = vec![];

    let mut extract = |tasks: &'builder FxHashMap<&'proj Id, &'proj TaskConfig>,
                       inherited: bool|
     -> miette::Result<()> {
        if let Some(config) = tasks.get(task_id) {
            stack.push(ConfigChain { config, inherited });

            if let Some(extend_task_id) = &config.extends {
                let extended_stack = extract_config(extend_task_id, local_tasks, global_tasks)?;

                if extended_stack.is_empty() {
                    return Err(TasksBuilderError::UnknownExtendsSource {
                        source_id: task_id.to_owned(),
                        target_id: extend_task_id.to_owned(),
                    }
                    .into());
                } else {
                    stack.extend(extended_stack);
                }
            }
        }

        Ok(())
    };

    extract(local_tasks, false)?;
    extract(global_tasks, true)?;

    Ok(stack)
}

#[derive(Debug)]
pub struct DetectPlatformEvent {
    pub enabled_platforms: Vec<PlatformType>,
    pub task_command: String,
}

impl Event for DetectPlatformEvent {
    type Data = PlatformType;
}

pub struct TasksBuilderContext<'proj> {
    pub detect_platform: &'proj Emitter<DetectPlatformEvent>,
    pub legacy_task_inheritance: bool,
    pub toolchain_config: &'proj ToolchainConfig,
    pub workspace_root: &'proj Path,
}

pub struct TasksBuilder<'proj> {
    context: TasksBuilderContext<'proj>,

    project_id: &'proj str,
    project_env: FxHashMap<&'proj str, &'proj str>,
    project_platform: &'proj PlatformType,
    project_source: &'proj str,

    // Global settings for tasks to inherit
    implicit_deps: Vec<&'proj Target>,
    implicit_inputs: Vec<&'proj InputPath>,

    // Tasks to merge and build
    task_ids: FxHashSet<&'proj Id>,
    global_tasks: FxHashMap<&'proj Id, &'proj TaskConfig>,
    local_tasks: FxHashMap<&'proj Id, &'proj TaskConfig>,
    filters: Option<&'proj ProjectWorkspaceInheritedTasksConfig>,
}

impl<'proj> TasksBuilder<'proj> {
    pub fn new(
        project_id: &'proj str,
        project_source: &'proj str,
        project_platform: &'proj PlatformType,
        context: TasksBuilderContext<'proj>,
    ) -> Self {
        Self {
            context,
            project_id,
            project_env: FxHashMap::default(),
            project_platform,
            project_source,
            implicit_deps: vec![],
            implicit_inputs: vec![],
            task_ids: FxHashSet::default(),
            global_tasks: FxHashMap::default(),
            local_tasks: FxHashMap::default(),
            filters: None,
        }
    }

    pub fn inherit_global_tasks(
        &mut self,
        global_config: &'proj InheritedTasksConfig,
        global_filters: Option<&'proj ProjectWorkspaceInheritedTasksConfig>,
    ) -> &mut Self {
        let mut include_all = true;
        let mut include_set = FxHashSet::default();
        let mut exclude = vec![];
        let mut rename = FxHashMap::default();

        if let Some(filters) = global_filters {
            exclude.extend(&filters.exclude);
            rename.extend(&filters.rename);

            if let Some(include_config) = &filters.include {
                include_all = false;
                include_set.extend(include_config);
            }
        }

        trace!(
            id = self.project_id,
            tasks = ?global_config.tasks.keys().map(|k| k.as_str()).collect::<Vec<_>>(),
            "Filtering global tasks",
        );

        for (task_id, task_config) in &global_config.tasks {
            let target = Target::new(self.project_id, task_id).unwrap();

            // None = Include all
            // [] = Include none
            // ["a"] = Include "a"
            if !include_all {
                if include_set.is_empty() {
                    trace!(
                        target = target.as_str(),
                        "Not inheriting any global tasks, empty include filter",
                    );

                    break;
                } else if !include_set.contains(task_id) {
                    trace!(
                        target = target.as_str(),
                        "Not inheriting global task {}, not included",
                        color::id(task_id)
                    );

                    continue;
                }
            }

            // None, [] = Exclude none
            // ["a"] = Exclude "a"
            if !exclude.is_empty() && exclude.contains(&task_id) {
                trace!(
                    target = target.as_str(),
                    "Not inheriting global task {}, excluded",
                    color::id(task_id)
                );

                continue;
            }

            let task_key = if let Some(renamed_task_id) = rename.get(task_id) {
                trace!(
                    target = target.as_str(),
                    "Inheriting global task {} and renaming to {}",
                    color::id(task_id),
                    color::id(renamed_task_id)
                );

                renamed_task_id
            } else {
                trace!(
                    target = target.as_str(),
                    "Inheriting global task {}",
                    color::id(task_id),
                );

                task_id
            };

            self.global_tasks.insert(task_key, task_config);
            self.task_ids.insert(task_key);
        }

        self.filters = global_filters;
        self.implicit_deps.extend(&global_config.implicit_deps);
        self.implicit_inputs.extend(&global_config.implicit_inputs);
        self
    }

    pub fn load_local_tasks(&mut self, local_config: &'proj ProjectConfig) -> &mut Self {
        for (key, value) in &local_config.env {
            self.project_env.insert(key, value);
        }

        trace!(
            id = self.project_id,
            tasks = ?local_config.tasks.keys().map(|k| k.as_str()).collect::<Vec<_>>(),
            "Loading local tasks",
        );

        self.local_tasks.extend(&local_config.tasks);

        for id in local_config.tasks.keys() {
            self.task_ids.insert(id);
        }

        self
    }

    #[tracing::instrument(name = "task", skip_all)]
    pub async fn build(self) -> miette::Result<BTreeMap<Id, Task>> {
        let mut tasks = BTreeMap::new();

        for id in &self.task_ids {
            tasks.insert((*id).to_owned(), self.build_task(id).await?);
        }

        Ok(tasks)
    }

    async fn build_task(&self, id: &Id) -> miette::Result<Task> {
        let target = Target::new(self.project_id, id)?;

        trace!(
            target = target.as_str(),
            "Building task {}",
            color::id(id.as_str())
        );

        let mut task = Task::default();
        let chain = self.get_config_inherit_chain(id, true)?;

        // Determine command and args before building options and the task,
        // as we need to figure out if we're running in local mode or not.
        let mut is_local = id == "dev" || id == "serve" || id == "start";
        let mut args_sets = vec![];

        for link in &chain {
            let (command, base_args) = self.get_command_and_args(link.config)?;

            if let Some(command) = command {
                task.command = command;
            }

            // Add to task later after we have a merge strategy
            args_sets.push(base_args);

            if let Some(local) = link.config.local {
                is_local = local;
            }
        }

        if is_local {
            trace!(target = target.as_str(), "Marking task as local");
        }

        task.options = self.build_task_options(id, is_local)?;
        task.flags.local = is_local;

        // Aggregate all values that are inherited from the global task configs,
        // and should always be included in the task, regardless of merge strategy.
        let global_deps = self.build_global_deps(&target)?;
        let mut global_inputs = self.build_global_inputs(&target, &task.options)?;

        // Aggregate all values that that are inherited from the project,
        // and should be set on the task first, so that merge strategies can be applied.
        for args in args_sets {
            task.args = self.merge_vec(task.args, args, task.options.merge_args, false);
        }

        task.env = self.build_env(&target)?;

        // Finally build the task itself, while applying our complex merge logic!
        let mut configured_inputs = 0;
        let mut has_configured_inputs = false;

        for link in &chain {
            let config = link.config;

            task.deps = self.merge_vec(
                task.deps,
                if link.inherited {
                    self.apply_filters_to_deps(config.deps.to_owned())
                } else {
                    config.deps.to_owned()
                },
                task.options.merge_deps,
                true,
            );

            task.env = self.merge_map(task.env, config.env.to_owned(), task.options.merge_env);

            // Inherit global inputs as normal inputs, but do not consider them a configured input
            if !config.global_inputs.is_empty() {
                global_inputs.extend(config.global_inputs.to_owned());
            }

            // Inherit local inputs, which are user configured, and keep track of the total
            if let Some(inputs) = &config.inputs {
                has_configured_inputs = true;

                if inputs.is_empty()
                    && matches!(task.options.merge_inputs, TaskMergeStrategy::Replace)
                {
                    configured_inputs = 0;
                } else {
                    configured_inputs += inputs.len();
                }

                task.inputs = self.merge_vec(
                    task.inputs,
                    inputs.to_owned(),
                    task.options.merge_inputs,
                    true,
                );
            }

            if let Some(outputs) = &config.outputs {
                task.outputs = self.merge_vec(
                    task.outputs,
                    outputs.to_owned(),
                    task.options.merge_outputs,
                    true,
                );
            }

            if !config.platform.is_unknown() {
                task.platform = config.platform;
            }
        }

        // Inputs are tricky, as they come from many sources. We need to ensure that user configured
        // inputs are handled explicitly, while globally inherited sources are handled implicitly.
        if configured_inputs == 0 {
            if has_configured_inputs {
                trace!(
                    target = target.as_str(),
                    "Task has explicitly disabled inputs",
                );

                task.flags.empty_inputs = true;
            } else {
                trace!(
                    target = target.as_str(),
                    "No inputs configured, defaulting to {} (from project)",
                    color::file("**/*"),
                );

                task.inputs.push(InputPath::ProjectGlob("**/*".into()));
            }
        }

        // And lastly, before we return the task and options, we should finalize
        // all necessary fields and populate/calculate with values.
        if task.command.is_empty() {
            task.command = "noop".into();
        }

        if !global_deps.is_empty() {
            task.deps = self.merge_vec(task.deps, global_deps, TaskMergeStrategy::Append, true);
        }

        task.id = id.to_owned();

        if !global_inputs.is_empty() {
            task.inputs =
                self.merge_vec(task.inputs, global_inputs, TaskMergeStrategy::Append, true);
        }

        if task.platform.is_unknown() {
            let platform = self
                .context
                .detect_platform
                .emit(DetectPlatformEvent {
                    enabled_platforms: self.context.toolchain_config.get_enabled_platforms(),
                    task_command: task.command.clone(),
                })
                .await?;

            task.platform = if platform.is_unknown() {
                if self.project_platform.is_unknown() {
                    PlatformType::System
                } else {
                    self.project_platform.to_owned()
                }
            } else {
                platform
            };
        }

        task.target = target;

        task.type_of = if !task.outputs.is_empty() {
            TaskType::Build
        } else if is_local {
            TaskType::Run
        } else {
            TaskType::Test
        };

        if task.options.shell.is_none() {
            // Windows requires a shell for path resolution to work correctly
            if cfg!(windows) || task.platform.is_system() {
                task.options.shell = Some(true);
            }
        }

        Ok(task)
    }

    fn build_task_options(&self, id: &Id, is_local: bool) -> miette::Result<TaskOptions> {
        let mut options = TaskOptions {
            cache: !is_local,
            interactive: false,
            output_style: is_local.then_some(TaskOutputStyle::Stream),
            persistent: is_local,
            run_in_ci: !is_local,
            ..TaskOptions::default()
        };

        let configs = self
            .get_config_inherit_chain(id, false)?
            .iter()
            .map(|link| &link.config.options)
            .collect::<Vec<_>>();

        for config in configs {
            if let Some(affected_files) = &config.affected_files {
                options.affected_files = Some(affected_files.to_owned());
            }

            if let Some(allow_failure) = &config.allow_failure {
                options.allow_failure = *allow_failure;
            }

            if let Some(cache) = &config.cache {
                options.cache = *cache;
            }

            if let Some(env_file) = &config.env_file {
                options.env_file = env_file.to_input_path();
            }

            if let Some(interactive) = &config.interactive {
                options.interactive = *interactive;
            }

            if let Some(merge_args) = &config.merge_args {
                options.merge_args = *merge_args;
            }

            if let Some(merge_deps) = &config.merge_deps {
                options.merge_deps = *merge_deps;
            }

            if let Some(merge_env) = &config.merge_env {
                options.merge_env = *merge_env;
            }

            if let Some(merge_inputs) = &config.merge_inputs {
                options.merge_inputs = *merge_inputs;
            }

            if let Some(merge_outputs) = &config.merge_outputs {
                options.merge_outputs = *merge_outputs;
            }

            if let Some(output_style) = &config.output_style {
                options.output_style = Some(*output_style);
            }

            if let Some(persistent) = &config.persistent {
                options.persistent = *persistent;
            }

            if let Some(retry_count) = &config.retry_count {
                options.retry_count = *retry_count;
            }

            if let Some(run_deps_in_parallel) = &config.run_deps_in_parallel {
                options.run_deps_in_parallel = *run_deps_in_parallel;
            }

            if let Some(run_in_ci) = &config.run_in_ci {
                options.run_in_ci = *run_in_ci;
            }

            if let Some(run_from_workspace_root) = &config.run_from_workspace_root {
                options.run_from_workspace_root = *run_from_workspace_root;
            }

            if let Some(shell) = &config.shell {
                options.shell = Some(*shell);
            }
        }

        if options.interactive {
            options.cache = false;
            options.output_style = Some(TaskOutputStyle::Stream);
            options.persistent = false;
            options.run_in_ci = false;
        }

        Ok(options)
    }

    fn build_global_deps(&self, target: &Target) -> miette::Result<Vec<Target>> {
        let global_deps = self
            .implicit_deps
            .iter()
            .map(|d| (*d).to_owned())
            .collect::<Vec<_>>();

        if !global_deps.is_empty() {
            trace!(
                target = target.as_str(),
                deps = ?global_deps.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
                "Inheriting global implicit deps",
            );
        }

        Ok(global_deps)
    }

    fn build_global_inputs(
        &self,
        target: &Target,
        options: &TaskOptions,
    ) -> miette::Result<Vec<InputPath>> {
        let mut global_inputs = self
            .implicit_inputs
            .iter()
            .map(|d| (*d).to_owned())
            .collect::<Vec<_>>();

        global_inputs.push(InputPath::WorkspaceGlob(".moon/*.yml".into()));

        if let Some(env_file) = &options.env_file {
            global_inputs.push(env_file.to_owned());
        }

        if !global_inputs.is_empty() {
            trace!(
                target = target.as_str(),
                inputs = ?global_inputs.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
                "Inheriting global implicit inputs",
            );
        }

        Ok(global_inputs)
    }

    fn build_env(&self, target: &Target) -> miette::Result<FxHashMap<String, String>> {
        let env = self
            .project_env
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect::<FxHashMap<_, _>>();

        if !env.is_empty() {
            trace!(
                target = target.as_str(),
                env_vars = ?self.project_env,
                "Inheriting project env vars",
            );
        }

        Ok(env)
    }

    fn get_command_and_args(
        &self,
        config: &TaskConfig,
    ) -> miette::Result<(Option<String>, Vec<String>)> {
        let mut command = None;
        let mut args = vec![];

        let mut cmd_list = match &config.command {
            TaskCommandArgs::None => vec![],
            TaskCommandArgs::String(cmd_string) => split_args(cmd_string)?,
            TaskCommandArgs::List(cmd_args) => cmd_args.to_owned(),
        };

        if !cmd_list.is_empty() {
            command = Some(cmd_list.remove(0));
            args.extend(cmd_list);
        }

        match &config.args {
            TaskCommandArgs::None => {}
            TaskCommandArgs::String(args_string) => args.extend(split_args(args_string)?),
            TaskCommandArgs::List(args_list) => args.extend(args_list.to_owned()),
        };

        Ok((command, args))
    }

    fn get_config_inherit_chain(
        &self,
        id: &Id,
        _inspect: bool,
    ) -> miette::Result<Vec<ConfigChain>> {
        let mut configs = vec![];

        if self.context.legacy_task_inheritance {
            let mut extendable_tasks = FxHashMap::default();
            extendable_tasks.extend(&self.global_tasks);
            extendable_tasks.extend(&self.local_tasks);

            // Inherit all global first
            legacy_extract_config(
                &mut configs,
                id,
                &self.global_tasks,
                &extendable_tasks,
                true,
                false,
            )?;

            // Then all local
            legacy_extract_config(
                &mut configs,
                id,
                &self.local_tasks,
                &extendable_tasks,
                false,
                false,
            )?;
        } else {
            let mut stack = extract_config(id, &self.local_tasks, &self.global_tasks)?;
            stack.reverse();

            configs.extend(stack);
        }

        Ok(configs)
    }

    fn apply_filters_to_deps(&self, deps: Vec<Target>) -> Vec<Target> {
        let Some(filters) = &self.filters else {
            return deps;
        };

        deps.into_iter()
            .filter(|dep| !filters.exclude.contains(&dep.task_id))
            .map(|mut dep| {
                if let Some(new_task_id) = filters.rename.get(&dep.task_id) {
                    dep.id = Target::format(&dep.scope, new_task_id);
                    dep.task_id = new_task_id.to_owned();
                }

                dep
            })
            .collect()
    }

    fn merge_map<K, V>(
        &self,
        base: FxHashMap<K, V>,
        next: FxHashMap<K, V>,
        strategy: TaskMergeStrategy,
    ) -> FxHashMap<K, V>
    where
        K: Eq + Hash,
    {
        match strategy {
            TaskMergeStrategy::Append => {
                if next.is_empty() {
                    return base;
                }

                let mut map = FxHashMap::default();
                map.extend(base);
                map.extend(next);
                map
            }
            TaskMergeStrategy::Prepend => {
                if next.is_empty() {
                    return base;
                }

                let mut map = FxHashMap::default();
                map.extend(next);
                map.extend(base);
                map
            }
            TaskMergeStrategy::Replace => next,
        }
    }

    fn merge_vec<T: Eq>(
        &self,
        base: Vec<T>,
        next: Vec<T>,
        strategy: TaskMergeStrategy,
        dedupe: bool,
    ) -> Vec<T> {
        let mut list: Vec<T> = vec![];

        // Dedupe while merging vectors. We can't use a set here because
        // we need to preserve the insertion order. Revisit if this is costly!
        let mut append = |items: Vec<T>, force: bool| {
            for item in items {
                #[allow(clippy::nonminimal_bool)]
                if force || !dedupe || (dedupe && !list.contains(&item)) {
                    list.push(item);
                }
            }
        };

        match strategy {
            TaskMergeStrategy::Append => {
                if next.is_empty() {
                    return base;
                }

                append(base, true);
                append(next, false);
            }
            TaskMergeStrategy::Prepend => {
                if next.is_empty() {
                    return base;
                }

                append(next, true);
                append(base, false);
            }
            TaskMergeStrategy::Replace => {
                list.extend(next);
            }
        }

        list
    }
}
