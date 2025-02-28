use crate::language_platform::PlatformType;
use crate::project::{PartialTaskOptionsConfig, TaskOptionsConfig};
use crate::shapes::{InputPath, OutputPath};
use moon_common::{cacheable, color, Id};
use moon_target::{Target, TargetScope};
use rustc_hash::FxHashMap;
use schematic::{
    derive_enum, merge, Config, ConfigEnum, ConfigLoader, Format, PathSegment, ValidateError,
};

fn validate_command<D, C>(
    command: &PartialTaskCommandArgs,
    _task: &D,
    _ctx: &C,
) -> Result<(), ValidateError> {
    let invalid = match command {
        PartialTaskCommandArgs::None => false,
        PartialTaskCommandArgs::String(args) => {
            let mut parts = args.split(' ');
            let cmd = parts.next();
            cmd.is_none() || cmd.unwrap().is_empty()
        }
        PartialTaskCommandArgs::List(args) => args.is_empty() || args[0].is_empty(),
    };

    if invalid {
        return Err(ValidateError::new(
            "a command is required; use \"noop\" otherwise",
        ));
    }

    Ok(())
}

pub fn validate_deps<D, C>(deps: &[Target], _task: &D, _context: &C) -> Result<(), ValidateError> {
    for (i, dep) in deps.iter().enumerate() {
        if matches!(dep.scope, TargetScope::All) {
            return Err(ValidateError::with_segment(
                "target scope not supported as a task dependency",
                PathSegment::Index(i),
            ));
        }
    }

    Ok(())
}

derive_enum!(
    #[derive(ConfigEnum, Copy, Default)]
    pub enum TaskType {
        Build,
        Run,
        #[default]
        Test,
    }
);

cacheable!(
    #[derive(Clone, Config, Debug, Eq, PartialEq)]
    #[serde(untagged, expecting = "expected a string or a list of strings")]
    pub enum TaskCommandArgs {
        #[setting(default, null)]
        None,
        String(String),
        List(Vec<String>),
    }
);

cacheable!(
    #[derive(Clone, Config, Debug, Eq, PartialEq)]
    pub struct TaskConfig {
        pub extends: Option<Id>,

        #[setting(nested, validate = validate_command)]
        pub command: TaskCommandArgs,

        #[setting(nested)]
        pub args: TaskCommandArgs,

        #[setting(validate = validate_deps)]
        pub deps: Vec<Target>,

        pub env: FxHashMap<String, String>,

        #[setting(skip, merge = merge::append_vec)]
        pub global_inputs: Vec<InputPath>,

        // None = All inputs (**/*)
        // [] = No inputs
        // [...] = Specific inputs
        pub inputs: Option<Vec<InputPath>>,

        pub local: Option<bool>,

        pub outputs: Option<Vec<OutputPath>>,

        #[setting(nested)]
        pub options: TaskOptionsConfig,

        pub platform: PlatformType,

        #[serde(rename = "type")]
        pub type_of: Option<TaskType>,
    }
);

impl TaskConfig {
    pub fn parse<T: AsRef<str>>(code: T) -> miette::Result<TaskConfig> {
        let result = ConfigLoader::<TaskConfig>::new()
            .set_help(color::muted_light("https://moonrepo.dev/docs/config/tasks"))
            .code(code.as_ref(), Format::Yaml)?
            .load()?;

        Ok(result.config)
    }
}
