use crate::get_node_env_paths;
use crate::node_tool::NodeTool;
use moon_config::NpmConfig;
use moon_logger::debug;
use moon_node_lang::{npm, LockfileDependencyVersions, NPM};
use moon_process::Command;
use moon_terminal::{print_checkpoint, Checkpoint};
use moon_tool::{
    async_trait, get_proto_env_vars, get_proto_version_env, load_tool_plugin, prepend_path_env_var,
    use_global_tool_on_path, DependencyManager, Tool,
};
use moon_utils::{get_workspace_root, is_ci};
use proto_core::{Id, ProtoEnvironment, Tool as ProtoTool, UnresolvedVersionSpec};
use rustc_hash::FxHashMap;
use starbase_utils::fs;
use std::env;
use std::path::Path;
use std::sync::Arc;

pub struct NpmTool {
    pub config: NpmConfig,

    pub global: bool,

    pub tool: ProtoTool,

    proto_env: Arc<ProtoEnvironment>,
}

impl NpmTool {
    pub async fn new(
        proto_env: Arc<ProtoEnvironment>,
        config: &NpmConfig,
    ) -> miette::Result<NpmTool> {
        Ok(NpmTool {
            global: use_global_tool_on_path() || config.version.is_none(),
            config: config.to_owned(),
            tool: load_tool_plugin(&Id::raw("npm"), &proto_env, config.plugin.as_ref().unwrap())
                .await?,
            proto_env,
        })
    }
}

#[async_trait]
impl Tool for NpmTool {
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }

    async fn setup(
        &mut self,
        last_versions: &mut FxHashMap<String, UnresolvedVersionSpec>,
    ) -> miette::Result<u8> {
        let mut count = 0;
        let version = self.config.version.as_ref();

        let Some(version) = version else {
            return Ok(count);
        };

        if self.global {
            debug!("Using global binary in PATH");

            return Ok(count);
        }

        if self.tool.is_setup(version).await? {
            self.tool.locate_globals_dir().await?;

            debug!("npm has already been setup");

            return Ok(count);
        }

        // When offline and the tool doesn't exist, fallback to the global binary
        if proto_core::is_offline() {
            debug!(
                "No internet connection and npm has not been setup, falling back to global binary in PATH"
            );

            self.global = true;

            return Ok(count);
        }

        if let Some(last) = last_versions.get("npm") {
            if last == version && self.tool.get_tool_dir().exists() {
                return Ok(count);
            }
        }

        print_checkpoint(format!("installing npm {version}"), Checkpoint::Setup);

        if self.tool.setup(version, false).await? {
            last_versions.insert("npm".into(), version.to_owned());
            count += 1;
        }

        self.tool.locate_globals_dir().await?;

        Ok(count)
    }

    async fn teardown(&mut self) -> miette::Result<()> {
        self.tool.teardown().await?;

        Ok(())
    }
}

#[async_trait]
impl DependencyManager<NodeTool> for NpmTool {
    fn create_command(&self, node: &NodeTool) -> miette::Result<Command> {
        let mut cmd = Command::new("npm");
        cmd.envs(get_proto_env_vars());

        if !self.global {
            cmd.env(
                "PATH",
                prepend_path_env_var(get_node_env_paths(&self.proto_env)),
            );
        }

        if let Some(version) = get_proto_version_env(&self.tool) {
            cmd.env("PROTO_NPM_VERSION", version);
        }

        if let Some(version) = get_proto_version_env(&node.tool) {
            cmd.env("PROTO_NODE_VERSION", version);
        }

        // Tell proto to resolve instead of failing
        cmd.env_if_missing("PROTO_NPM_VERSION", "*");
        cmd.env_if_missing("PROTO_NODE_VERSION", "*");

        Ok(cmd)
    }

    async fn dedupe_dependencies(
        &self,
        node: &NodeTool,
        working_dir: &Path,
        log: bool,
    ) -> miette::Result<()> {
        self.create_command(node)?
            .args(["dedupe"])
            .cwd(working_dir)
            .set_print_command(log)
            .create_async()
            .exec_capture_output()
            .await?;

        Ok(())
    }

    fn get_lock_filename(&self) -> String {
        String::from(NPM.lockfile)
    }

    fn get_manifest_filename(&self) -> String {
        String::from(NPM.manifest)
    }

    async fn get_resolved_dependencies(
        &self,
        project_root: &Path,
    ) -> miette::Result<LockfileDependencyVersions> {
        let Some(lockfile_path) =
            fs::find_upwards_until(NPM.lockfile, project_root, get_workspace_root())
        else {
            return Ok(FxHashMap::default());
        };

        Ok(npm::load_lockfile_dependencies(lockfile_path)?)
    }

    async fn install_dependencies(
        &self,
        node: &NodeTool,
        working_dir: &Path,
        log: bool,
    ) -> miette::Result<()> {
        let mut args = vec!["install"];

        if is_ci() {
            let lockfile = working_dir.join(self.get_lock_filename());

            // npm will error if using `ci` and a lockfile does not exist!
            if lockfile.exists() {
                args.clear();
                args.push("ci");
            }
        } else {
            args.push("--no-audit");
        }

        args.push("--no-fund");

        let mut cmd = self.create_command(node)?;

        cmd.args(args).cwd(working_dir).set_print_command(log);

        let mut cmd = cmd.create_async();

        if env::var("MOON_TEST_HIDE_INSTALL_OUTPUT").is_ok() {
            cmd.exec_capture_output().await?;
        } else {
            cmd.exec_stream_output().await?;
        }

        Ok(())
    }

    async fn install_focused_dependencies(
        &self,
        node: &NodeTool,
        package_names: &[String],
        production_only: bool,
    ) -> miette::Result<()> {
        let mut cmd = self.create_command(node)?;
        cmd.args(["install"]);

        if production_only {
            cmd.arg("--production");
        }

        for package_name in package_names {
            cmd.args(["--workspace", package_name]);
        }

        cmd.create_async().exec_stream_output().await?;

        Ok(())
    }
}
