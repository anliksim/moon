use crate::create_input_paths;
use moon_config::{
    InputPath, NodePackageManager, PartialBunConfig, PartialBunpmConfig,
    PartialInheritedTasksConfig, PartialNodeConfig, PartialNpmConfig, PartialPnpmConfig,
    PartialTaskCommandArgs, PartialTaskConfig, PartialToolchainConfig, PartialTypeScriptConfig,
    PartialWorkspaceConfig, PartialWorkspaceProjects, PartialWorkspaceProjectsConfig,
    PartialYarnConfig, UnresolvedVersionSpec,
};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::str::FromStr;

// Turn everything off by default
pub fn get_default_toolchain() -> PartialToolchainConfig {
    PartialToolchainConfig {
        node: Some(PartialNodeConfig {
            version: Some(UnresolvedVersionSpec::parse("18.0.0").unwrap()),
            add_engines_constraint: Some(false),
            dedupe_on_lockfile_change: Some(false),
            infer_tasks_from_scripts: Some(false),
            sync_project_workspace_dependencies: Some(false),
            npm: Some(PartialNpmConfig {
                version: Some(UnresolvedVersionSpec::parse("8.19.0").unwrap()),
                ..PartialNpmConfig::default()
            }),
            ..PartialNodeConfig::default()
        }),
        typescript: Some(PartialTypeScriptConfig {
            create_missing_config: Some(false),
            route_out_dir_to_cache: Some(false),
            sync_project_references: Some(false),
            sync_project_references_to_paths: Some(false),
            ..PartialTypeScriptConfig::default()
        }),
        ..PartialToolchainConfig::default()
    }
}

pub fn get_cases_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let workspace_config = PartialWorkspaceConfig {
        projects: Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
            ("root".into(), ".".to_owned()),
            ("affected".into(), "affected".to_owned()),
            ("base".into(), "base".to_owned()),
            ("noop".into(), "noop".to_owned()),
            ("files".into(), "files".to_owned()),
            ("states".into(), "states".to_owned()),
            // Runner
            ("interactive".into(), "interactive".to_owned()),
            ("passthroughArgs".into(), "passthrough-args".to_owned()),
            // Project/task deps
            ("depsA".into(), "deps-a".to_owned()),
            ("depsB".into(), "deps-b".to_owned()),
            ("depsC".into(), "deps-c".to_owned()),
            ("dependsOn".into(), "depends-on".to_owned()),
            // Target scopes
            ("targetScopeA".into(), "target-scope-a".to_owned()),
            ("targetScopeB".into(), "target-scope-b".to_owned()),
            ("targetScopeC".into(), "target-scope-c".to_owned()),
            // Outputs
            ("outputs".into(), "outputs".to_owned()),
            ("outputsFiltering".into(), "outputs-filtering".to_owned()),
            ("outputStyles".into(), "output-styles".to_owned()),
        ]))),
        ..PartialWorkspaceConfig::default()
    };

    let toolchain_config = get_default_toolchain();

    let tasks_config = PartialInheritedTasksConfig {
        tasks: Some(BTreeMap::from_iter([(
            "noop".into(),
            PartialTaskConfig {
                command: Some(PartialTaskCommandArgs::String("noop".into())),
                ..PartialTaskConfig::default()
            },
        )])),
        ..PartialInheritedTasksConfig::default()
    };

    (workspace_config, toolchain_config, tasks_config)
}

pub fn get_projects_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let workspace_config = PartialWorkspaceConfig {
        projects: Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
            ("advanced".into(), "advanced".to_owned()),
            ("basic".into(), "basic".to_owned()),
            ("emptyConfig".into(), "empty-config".to_owned()),
            ("noConfig".into(), "no-config".to_owned()),
            ("tasks".into(), "tasks".to_owned()),
            ("platforms".into(), "platforms".to_owned()),
            // Deps
            ("foo".into(), "deps/foo".to_owned()),
            ("bar".into(), "deps/bar".to_owned()),
            ("baz".into(), "deps/baz".to_owned()),
        ]))),
        ..PartialWorkspaceConfig::default()
    };

    let toolchain_config = get_default_toolchain();

    let tasks_config = PartialInheritedTasksConfig {
        file_groups: Some(FxHashMap::from_iter([
            (
                "sources".into(),
                create_input_paths(["src/**/*", "types/**/*"]),
            ),
            ("tests".into(), create_input_paths(["tests/**/*"])),
        ])),
        ..PartialInheritedTasksConfig::default()
    };

    (workspace_config, toolchain_config, tasks_config)
}

pub fn get_project_graph_aliases_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let workspace_config = PartialWorkspaceConfig {
        projects: Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
            ("explicit".into(), "explicit".to_owned()),
            (
                "explicitAndImplicit".into(),
                "explicit-and-implicit".to_owned(),
            ),
            ("implicit".into(), "implicit".to_owned()),
            ("noLang".into(), "no-lang".to_owned()),
            // Node.js
            ("node".into(), "node".to_owned()),
            ("nodeNameOnly".into(), "node-name-only".to_owned()),
            ("nodeNameScope".into(), "node-name-scope".to_owned()),
        ]))),
        ..PartialWorkspaceConfig::default()
    };

    let toolchain_config = PartialToolchainConfig {
        node: Some(PartialNodeConfig {
            version: Some(UnresolvedVersionSpec::parse("18.0.0").unwrap()),
            add_engines_constraint: Some(false),
            dedupe_on_lockfile_change: Some(false),
            npm: Some(PartialNpmConfig {
                version: Some(UnresolvedVersionSpec::parse("8.19.0").unwrap()),
                ..PartialNpmConfig::default()
            }),
            ..PartialNodeConfig::default()
        }),
        ..PartialToolchainConfig::default()
    };

    let tasks_config = PartialInheritedTasksConfig::default();

    (workspace_config, toolchain_config, tasks_config)
}

pub fn get_tasks_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let workspace_config = PartialWorkspaceConfig {
        projects: Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
            ("basic".into(), "basic".to_owned()),
            ("buildA".into(), "build-a".to_owned()),
            ("buildB".into(), "build-b".to_owned()),
            ("buildC".into(), "build-c".to_owned()),
            ("chain".into(), "chain".to_owned()),
            ("cycle".into(), "cycle".to_owned()),
            ("inheritTags".into(), "inherit-tags".to_owned()),
            ("inputA".into(), "input-a".to_owned()),
            ("inputB".into(), "input-b".to_owned()),
            ("inputC".into(), "input-c".to_owned()),
            ("inputs".into(), "inputs".to_owned()),
            (
                "mergeAllStrategies".into(),
                "merge-all-strategies".to_owned(),
            ),
            ("mergeAppend".into(), "merge-append".to_owned()),
            ("mergePrepend".into(), "merge-prepend".to_owned()),
            ("mergeReplace".into(), "merge-replace".to_owned()),
            ("noTasks".into(), "no-tasks".to_owned()),
            ("persistent".into(), "persistent".to_owned()),
            ("scopeAll".into(), "scope-all".to_owned()),
            ("scopeDeps".into(), "scope-deps".to_owned()),
            ("scopeSelf".into(), "scope-self".to_owned()),
            ("tokens".into(), "tokens".to_owned()),
            ("expandEnv".into(), "expand-env".to_owned()),
            ("expandEnvProject".into(), "expand-env-project".to_owned()),
            ("expandOutputs".into(), "expand-outputs".to_owned()),
            ("fileGroups".into(), "file-groups".to_owned()),
        ]))),
        ..PartialWorkspaceConfig::default()
    };

    let toolchain_config = get_default_toolchain();

    let tasks_config = PartialInheritedTasksConfig {
        file_groups: Some(FxHashMap::from_iter([
            (
                "static".into(),
                create_input_paths([
                    "file.ts",
                    "dir",
                    "dir/other.tsx",
                    "dir/subdir",
                    "dir/subdir/another.ts",
                ]),
            ),
            ("dirs_glob".into(), create_input_paths(["**/*"])),
            ("files_glob".into(), create_input_paths(["**/*.{ts,tsx}"])),
            (
                "globs".into(),
                create_input_paths(["**/*.{ts,tsx}", "*.js"]),
            ),
            ("no_globs".into(), create_input_paths(["config.js"])),
        ])),
        tasks: Some(BTreeMap::from_iter([
            (
                "standard".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("cmd".into())),
                    ..PartialTaskConfig::default()
                },
            ),
            (
                "withArgs".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("cmd".into())),
                    args: Some(PartialTaskCommandArgs::List(vec![
                        "--foo".into(),
                        "--bar".into(),
                        "baz".into(),
                    ])),
                    ..PartialTaskConfig::default()
                },
            ),
            (
                "withInputs".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("cmd".into())),
                    inputs: Some(vec![
                        InputPath::from_str("rel/file.*").unwrap(),
                        InputPath::from_str("/root.*").unwrap(),
                    ]),
                    ..PartialTaskConfig::default()
                },
            ),
            (
                "withOutputs".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("cmd".into())),
                    inputs: Some(vec![
                        InputPath::from_str("lib").unwrap(),
                        InputPath::from_str("/build").unwrap(),
                    ]),
                    ..PartialTaskConfig::default()
                },
            ),
        ])),
        ..PartialInheritedTasksConfig::default()
    };

    (workspace_config, toolchain_config, tasks_config)
}

// JAVASCRIPT

pub fn get_bun_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let workspace_config = PartialWorkspaceConfig {
        projects: Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
            ("bun".into(), "base".to_owned()),
            ("packageManager".into(), "package-manager".to_owned()),
            ("versionOverride".into(), "version-override".to_owned()),
        ]))),
        ..PartialWorkspaceConfig::default()
    };

    let mut toolchain_config = get_default_toolchain();
    toolchain_config.node = None;
    toolchain_config.bun = Some(PartialBunConfig {
        version: Some(UnresolvedVersionSpec::parse("1.0.0").unwrap()),
        ..PartialBunConfig::default()
    });

    let tasks_config = PartialInheritedTasksConfig {
        tasks: Some(BTreeMap::from_iter([
            (
                "version".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("bun".into())),
                    args: Some(PartialTaskCommandArgs::String("--version".into())),
                    ..PartialTaskConfig::default()
                },
            ),
            (
                "noop".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("noop".into())),
                    ..PartialTaskConfig::default()
                },
            ),
        ])),
        ..PartialInheritedTasksConfig::default()
    };

    (workspace_config, toolchain_config, tasks_config)
}

pub fn get_node_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let workspace_config = PartialWorkspaceConfig {
        projects: Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
            ("node".into(), "base".to_owned()),
            ("lifecycles".into(), "lifecycles".to_owned()),
            ("postinstall".into(), "postinstall".to_owned()),
            (
                "postinstallRecursion".into(),
                "postinstall-recursion".to_owned(),
            ),
            ("versionOverride".into(), "version-override".to_owned()),
            // Binaries
            ("esbuild".into(), "esbuild".to_owned()),
            ("swc".into(), "swc".to_owned()),
            // Project/task deps
            ("depsA".into(), "deps-a".to_owned()),
            ("depsB".into(), "deps-b".to_owned()),
            ("depsC".into(), "deps-c".to_owned()),
            ("depsD".into(), "deps-d".to_owned()),
            ("dependsOn".into(), "depends-on".to_owned()),
            ("dependsOnScopes".into(), "depends-on-scopes".to_owned()),
        ]))),
        ..PartialWorkspaceConfig::default()
    };

    let toolchain_config = get_default_toolchain();

    let tasks_config = PartialInheritedTasksConfig {
        tasks: Some(BTreeMap::from_iter([
            (
                "version".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("node".into())),
                    args: Some(PartialTaskCommandArgs::String("--version".into())),
                    ..PartialTaskConfig::default()
                },
            ),
            (
                "noop".into(),
                PartialTaskConfig {
                    command: Some(PartialTaskCommandArgs::String("noop".into())),
                    ..PartialTaskConfig::default()
                },
            ),
        ])),
        ..PartialInheritedTasksConfig::default()
    };

    (workspace_config, toolchain_config, tasks_config)
}

pub fn get_node_depman_fixture_configs(
    depman: &str,
) -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let (mut workspace_config, mut toolchain_config, tasks_config) = get_node_fixture_configs();

    workspace_config.projects = Some(PartialWorkspaceProjects::Sources(FxHashMap::from_iter([
        (depman.into(), "base".to_owned()),
        ("other".into(), "other".to_owned()),
        ("notInWorkspace".into(), "not-in-workspace".to_owned()),
    ])));

    if let Some(node_config) = &mut toolchain_config.node {
        match depman {
            "bun" => {
                node_config.package_manager = Some(NodePackageManager::Bun);
                node_config.bun = Some(PartialBunpmConfig {
                    version: Some(UnresolvedVersionSpec::parse("1.0.0").unwrap()),
                    ..PartialBunpmConfig::default()
                });
            }
            "npm" => {
                node_config.package_manager = Some(NodePackageManager::Npm);
                node_config.npm = Some(PartialNpmConfig {
                    version: Some(UnresolvedVersionSpec::parse("8.0.0").unwrap()),
                    ..PartialNpmConfig::default()
                });
            }
            "pnpm" => {
                node_config.package_manager = Some(NodePackageManager::Pnpm);
                node_config.pnpm = Some(PartialPnpmConfig {
                    version: Some(UnresolvedVersionSpec::parse("7.5.0").unwrap()),
                    ..PartialPnpmConfig::default()
                });
            }
            "yarn" => {
                node_config.package_manager = Some(NodePackageManager::Yarn);
                node_config.yarn = Some(PartialYarnConfig {
                    version: Some(UnresolvedVersionSpec::parse("3.3.0").unwrap()),
                    plugins: Some(vec!["workspace-tools".into()]),
                    ..PartialYarnConfig::default()
                });
            }
            "yarn1" => {
                node_config.package_manager = Some(NodePackageManager::Yarn);
                node_config.yarn = Some(PartialYarnConfig {
                    version: Some(UnresolvedVersionSpec::parse("1.22.0").unwrap()),
                    plugins: Some(vec![]),
                    ..PartialYarnConfig::default()
                });
            }
            _ => {}
        }
    }

    (workspace_config, toolchain_config, tasks_config)
}

pub fn get_typescript_fixture_configs() -> (
    PartialWorkspaceConfig,
    PartialToolchainConfig,
    PartialInheritedTasksConfig,
) {
    let (mut workspace_config, mut toolchain_config, tasks_config) = get_node_fixture_configs();

    workspace_config.projects = Some(PartialWorkspaceProjects::Both(
        PartialWorkspaceProjectsConfig {
            globs: Some(vec!["*".into()]),
            sources: Some(FxHashMap::from_iter([("root".into(), ".".into())])),
        },
    ));

    if let Some(ts_config) = &mut toolchain_config.typescript {
        ts_config.create_missing_config = Some(true);
        ts_config.sync_project_references = Some(true);
    }

    (workspace_config, toolchain_config, tasks_config)
}
