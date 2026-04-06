use crate::Result;
use crate::config::miserc;
use crate::env_diff::{EnvDiff, EnvDiffOperation, EnvDiffPatches, EnvMap};
use crate::file::replace_path;
use crate::shell::ShellType;
use crate::{cli::args::ToolArg, file::display_path};
use eyre::Context;
use indexmap::IndexSet;
use itertools::Itertools;
use log::LevelFilter;
pub use std::env::*;
use std::sync::LazyLock as Lazy;
use std::sync::RwLock;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    sync::Mutex,
};
use std::{path, process};
use std::{path::Path, string::ToString};
use std::{path::PathBuf, sync::atomic::AtomicBool};

pub static ARGS: RwLock<Vec<String>> = RwLock::new(vec![]);
pub static TOOL_ARGS: RwLock<Vec<ToolArg>> = RwLock::new(vec![]);
#[cfg(unix)]
pub static SHELL: Lazy<String> = Lazy::new(|| var("SHELL").unwrap_or_else(|_| "sh".into()));
#[cfg(windows)]
pub static SHELL: Lazy<String> = Lazy::new(|| var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()));
pub static MISE_SHELL: Lazy<Option<ShellType>> = Lazy::new(|| {
    var("MISE_SHELL")
        .unwrap_or_else(|_| SHELL.clone())
        .parse()
        .ok()
});
#[cfg(unix)]
pub static SHELL_COMMAND_FLAG: &str = "-c";
#[cfg(windows)]
pub static SHELL_COMMAND_FLAG: &str = "/c";

// paths and directories
#[cfg(test)]
pub static HOME: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test"));
#[cfg(not(test))]
pub static HOME: Lazy<PathBuf> = Lazy::new(|| {
    homedir::my_home()
        .ok()
        .flatten()
        .unwrap_or_else(|| PathBuf::from("/"))
});

pub static EDITOR: Lazy<String> =
    Lazy::new(|| var("VISUAL").unwrap_or_else(|_| var("EDITOR").unwrap_or_else(|_| "nano".into())));

#[cfg(macos)]
pub static XDG_CACHE_HOME: Lazy<PathBuf> =
    Lazy::new(|| var_path("XDG_CACHE_HOME").unwrap_or_else(|| HOME.join("Library/Caches")));
#[cfg(windows)]
pub static XDG_CACHE_HOME: Lazy<PathBuf> = Lazy::new(|| {
    var_path("XDG_CACHE_HOME")
        .or_else(|| var_path("TEMP"))
        .unwrap_or_else(temp_dir)
});
#[cfg(all(not(windows), not(macos)))]
pub static XDG_CACHE_HOME: Lazy<PathBuf> =
    Lazy::new(|| var_path("XDG_CACHE_HOME").unwrap_or_else(|| HOME.join(".cache")));
pub static XDG_CONFIG_HOME: Lazy<PathBuf> =
    Lazy::new(|| var_path("XDG_CONFIG_HOME").unwrap_or_else(|| HOME.join(".config")));
#[cfg(unix)]
pub static XDG_DATA_HOME: Lazy<PathBuf> =
    Lazy::new(|| var_path("XDG_DATA_HOME").unwrap_or_else(|| HOME.join(".local").join("share")));
#[cfg(windows)]
pub static XDG_DATA_HOME: Lazy<PathBuf> = Lazy::new(|| {
    var_path("XDG_DATA_HOME")
        .or(var_path("LOCALAPPDATA"))
        .unwrap_or_else(|| HOME.join("AppData/Local"))
});
pub static XDG_STATE_HOME: Lazy<PathBuf> =
    Lazy::new(|| var_path("XDG_STATE_HOME").unwrap_or_else(|| HOME.join(".local").join("state")));

/// control display of "friendly" errors - defaults to release mode behavior unless overridden
pub static MISE_FRIENDLY_ERROR: Lazy<bool> = Lazy::new(|| {
    if var_is_true("MISE_FRIENDLY_ERROR") {
        true
    } else if var_is_false("MISE_FRIENDLY_ERROR") {
        false
    } else {
        // default behavior: friendly in release mode unless debug logging
        !cfg!(debug_assertions) && log::max_level() < log::LevelFilter::Debug
    }
});
pub static MISE_TOOL_STUB: Lazy<bool> =
    Lazy::new(|| ARGS.read().unwrap().get(1).map(|s| s.as_str()) == Some("tool-stub"));
pub static MISE_NO_CONFIG: Lazy<bool> = Lazy::new(|| var_is_true("MISE_NO_CONFIG"));
pub static MISE_NO_ENV: Lazy<bool> = Lazy::new(|| var_is_true("MISE_NO_ENV"));
pub static MISE_NO_HOOKS: Lazy<bool> = Lazy::new(|| var_is_true("MISE_NO_HOOKS"));
pub static MISE_PROGRESS_TRACE: Lazy<bool> = Lazy::new(|| var_is_true("MISE_PROGRESS_TRACE"));
pub static MISE_CACHE_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_CACHE_DIR").unwrap_or_else(|| XDG_CACHE_HOME.join("mise")));
pub static MISE_CONFIG_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_CONFIG_DIR").unwrap_or_else(|| XDG_CONFIG_HOME.join("mise")));
/// The default config directory location (XDG_CONFIG_HOME/mise), used to filter out
/// configs from this location when MISE_CONFIG_DIR is set to a different path
pub static MISE_DEFAULT_CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| XDG_CONFIG_HOME.join("mise"));
/// True if MISE_CONFIG_DIR was explicitly set to a non-default location
pub static MISE_CONFIG_DIR_OVERRIDDEN: Lazy<bool> = Lazy::new(|| {
    var_path("MISE_CONFIG_DIR").is_some() && *MISE_CONFIG_DIR != *MISE_DEFAULT_CONFIG_DIR
});
pub static MISE_DATA_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_DATA_DIR").unwrap_or_else(|| XDG_DATA_HOME.join("mise")));
pub static MISE_STATE_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_STATE_DIR").unwrap_or_else(|| XDG_STATE_HOME.join("mise")));
pub static MISE_TMP_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_TMP_DIR").unwrap_or_else(|| temp_dir().join("mise")));
/// Raw system config path. **Prefer [`system_config_dir()`] in application code** — it applies
/// the Windows trust gate automatically. Only use this static directly when you need the raw
/// path regardless of trust (e.g. display/reporting or cleanup commands).
#[cfg(not(windows))]
pub static MISE_SYSTEM_CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    var_path("MISE_SYSTEM_CONFIG_DIR")
        .or_else(|| var_path("MISE_SYSTEM_DIR"))
        .unwrap_or_else(|| PathBuf::from("/etc/mise"))
});
/// Raw system config path. **Prefer [`system_config_dir()`] in application code** — it applies
/// the Windows trust gate automatically. Only use this static directly when you need the raw
/// path regardless of trust (e.g. display/reporting or cleanup commands).
#[cfg(windows)]
pub static MISE_SYSTEM_CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    var_path("MISE_SYSTEM_CONFIG_DIR")
        .or_else(|| var_path("MISE_SYSTEM_DIR"))
        .unwrap_or_else(windows_programdata_mise)
});

// data subdirs
pub static MISE_INSTALLS_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_INSTALLS_DIR").unwrap_or_else(|| MISE_DATA_DIR.join("installs")));
pub static MISE_DOWNLOADS_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_DOWNLOADS_DIR").unwrap_or_else(|| MISE_DATA_DIR.join("downloads")));
pub static MISE_PLUGINS_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_PLUGINS_DIR").unwrap_or_else(|| MISE_DATA_DIR.join("plugins")));
pub static MISE_SHIMS_DIR: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_SHIMS_DIR").unwrap_or_else(|| MISE_DATA_DIR.join("shims")));
/// Raw system data path. **Prefer [`system_data_dir()`] in application code** — it applies
/// the Windows trust gate automatically. Only use this static directly when you need the raw
/// path regardless of trust (e.g. display/reporting or `mise install --system`).
#[cfg(not(windows))]
pub static MISE_SYSTEM_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    var_path("MISE_SYSTEM_DATA_DIR").unwrap_or_else(|| PathBuf::from("/usr/local/share/mise"))
});
/// Raw system data path. **Prefer [`system_data_dir()`] in application code** — it applies
/// the Windows trust gate automatically. Only use this static directly when you need the raw
/// path regardless of trust (e.g. display/reporting or `mise install --system`).
#[cfg(windows)]
pub static MISE_SYSTEM_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    var_path("MISE_SYSTEM_DATA_DIR").unwrap_or_else(windows_programdata_mise)
});
/// System-level installs directory, derived from MISE_SYSTEM_DATA_DIR.
pub static MISE_SYSTEM_INSTALLS_DIR: Lazy<PathBuf> =
    Lazy::new(|| MISE_SYSTEM_DATA_DIR.join("installs"));

/// Extra shared install directories parsed from the environment variable.
/// This is the early/fallback source; prefer `shared_install_dirs()` which also
/// reads from Settings (config files) when available.
static MISE_SHARED_INSTALL_DIRS_ENV: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    var_os("MISE_SHARED_INSTALL_DIRS")
        .map(|v| {
            std::env::split_paths(&v)
                .filter(|p| !p.as_os_str().is_empty())
                .map(replace_path)
                .collect()
        })
        .unwrap_or_default()
});

/// Returns the list of shared install directories to search.
/// Includes the system installs dir (`MISE_SYSTEM_DATA_DIR/installs`) plus any
/// user-configured dirs from Settings (config files) or the environment variable.
/// The user's primary install dir is NOT included here — it is checked separately.
pub fn shared_install_dirs() -> Vec<PathBuf> {
    use crate::config::Settings;
    let user_dirs = if let std::result::Result::Ok(settings) = Settings::try_get()
        && let Some(ref dirs) = settings.shared_install_dirs
        && !dirs.is_empty()
    {
        dirs.clone()
    } else {
        MISE_SHARED_INSTALL_DIRS_ENV.clone()
    };
    let system = &*MISE_SYSTEM_INSTALLS_DIR;
    // System dir first (if it exists and isn't the user's own install dir),
    // then user-configured dirs.
    let mut result = Vec::new();
    #[cfg(windows)]
    let system_trusted = *WINDOWS_SYSTEM_DIR_TRUSTED;
    #[cfg(not(windows))]
    let system_trusted = true;
    if system_trusted && system.is_dir() && *system != *MISE_INSTALLS_DIR {
        result.push(system.clone());
    }
    result.extend(user_dirs);
    result
}

/// Early-boot variant used by install_state::init_tools() before Settings is loaded.
pub fn shared_install_dirs_early() -> Vec<PathBuf> {
    let system = &*MISE_SYSTEM_INSTALLS_DIR;
    let mut result = Vec::new();
    #[cfg(windows)]
    let system_trusted = *WINDOWS_SYSTEM_DIR_TRUSTED;
    #[cfg(not(windows))]
    let system_trusted = true;
    if system_trusted && system.is_dir() && *system != *MISE_INSTALLS_DIR {
        result.push(system.clone());
    }
    result.extend(MISE_SHARED_INSTALL_DIRS_ENV.iter().cloned());
    result
}

/// Categorize an install path as system, shared, or local.
pub fn install_path_category(path: &Path) -> InstallPathCategory {
    if path.starts_with(&*MISE_SYSTEM_INSTALLS_DIR) {
        InstallPathCategory::System
    } else if shared_install_dirs().iter().any(|d| path.starts_with(d)) {
        InstallPathCategory::Shared
    } else {
        InstallPathCategory::Local
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPathCategory {
    /// Primary user install dir
    Local,
    /// System-level (/usr/local/share/mise/installs)
    System,
    /// User-configured shared dir
    Shared,
}

/// Look up a tool version in shared install directories.
/// `tool_dir_name` should be the kebab-cased directory name (e.g. from `ba.installs_path`).
/// Returns the first shared path where `<shared_dir>/<tool_dir_name>/<pathname>` exists,
/// or `primary_path` if not found in any shared directory.
pub fn find_in_shared_installs(
    primary_path: PathBuf,
    tool_dir_name: &str,
    pathname: &str,
) -> PathBuf {
    if !primary_path.exists() {
        for shared_dir in shared_install_dirs() {
            let shared_path = shared_dir.join(tool_dir_name).join(pathname);
            if shared_path.exists() {
                return shared_path;
            }
        }
    }
    primary_path
}

pub static MISE_DEFAULT_TOOL_VERSIONS_FILENAME: Lazy<String> = Lazy::new(|| {
    var("MISE_DEFAULT_TOOL_VERSIONS_FILENAME")
        .ok()
        .or(MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES
            .as_ref()
            .and_then(|v| v.first().cloned()))
        .or(var("MISE_DEFAULT_TOOL_VERSIONS_FILENAME").ok())
        .unwrap_or_else(|| ".tool-versions".into())
});
pub static MISE_DEFAULT_CONFIG_FILENAME: Lazy<String> = Lazy::new(|| {
    var("MISE_DEFAULT_CONFIG_FILENAME")
        .ok()
        .or(MISE_OVERRIDE_CONFIG_FILENAMES.first().cloned())
        .unwrap_or_else(|| "mise.toml".into())
});
pub static MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES: Lazy<Option<IndexSet<String>>> =
    Lazy::new(|| match var("MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES") {
        Ok(v) if v == "none" => Some([].into()),
        Ok(v) => Some(v.split(':').map(|s| s.to_string()).collect()),
        Err(_) => {
            miserc::get_override_tool_versions_filenames().map(|v| v.iter().cloned().collect())
        }
    });
pub static MISE_OVERRIDE_CONFIG_FILENAMES: Lazy<IndexSet<String>> =
    Lazy::new(|| match var("MISE_OVERRIDE_CONFIG_FILENAMES") {
        Ok(v) => v.split(':').map(|s| s.to_string()).collect(),
        Err(_) => miserc::get_override_config_filenames()
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default(),
    });
pub static MISE_ENV: Lazy<Vec<String>> = Lazy::new(|| environment(&ARGS.read().unwrap()));
pub static MISE_GLOBAL_CONFIG_FILE: Lazy<Option<PathBuf>> =
    Lazy::new(|| var_path("MISE_GLOBAL_CONFIG_FILE").or_else(|| var_path("MISE_CONFIG_FILE")));
pub static MISE_GLOBAL_CONFIG_ROOT: Lazy<PathBuf> =
    Lazy::new(|| var_path("MISE_GLOBAL_CONFIG_ROOT").unwrap_or_else(|| HOME.to_path_buf()));
pub static MISE_SYSTEM_CONFIG_FILE: Lazy<Option<PathBuf>> =
    Lazy::new(|| var_path("MISE_SYSTEM_CONFIG_FILE"));
pub static MISE_IGNORED_CONFIG_PATHS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
    var("MISE_IGNORED_CONFIG_PATHS")
        .ok()
        .map(|v| {
            v.split(':')
                .filter(|p| !p.is_empty())
                .map(PathBuf::from)
                .map(replace_path)
                .collect()
        })
        .or_else(|| {
            miserc::get_ignored_config_paths()
                .map(|paths| paths.iter().cloned().map(replace_path).collect())
        })
        .unwrap_or_default()
});
pub static MISE_CEILING_PATHS: Lazy<HashSet<PathBuf>> = Lazy::new(|| {
    var("MISE_CEILING_PATHS")
        .ok()
        .map(|v| {
            split_paths(&v)
                .filter(|p| !p.as_os_str().is_empty())
                .map(replace_path)
                .collect()
        })
        .or_else(|| {
            miserc::get_ceiling_paths()
                .map(|paths| paths.iter().cloned().map(replace_path).collect())
        })
        .unwrap_or_default()
});
pub static MISE_USE_TOML: Lazy<bool> = Lazy::new(|| !var_is_false("MISE_USE_TOML"));
pub static MISE_LIST_ALL_VERSIONS: Lazy<bool> = Lazy::new(|| var_is_true("MISE_LIST_ALL_VERSIONS"));
pub static ARGV0: Lazy<String> = Lazy::new(|| ARGS.read().unwrap()[0].to_string());
pub static MISE_BIN_NAME: Lazy<&str> = Lazy::new(|| filename(&ARGV0));
pub static MISE_LOG_FILE: Lazy<Option<PathBuf>> = Lazy::new(|| var_path("MISE_LOG_FILE"));
pub static MISE_LOG_FILE_LEVEL: Lazy<Option<LevelFilter>> = Lazy::new(log_file_level);
fn find_in_tree(base: &Path, rels: &[&[&str]]) -> Option<PathBuf> {
    for rel in rels {
        let mut p = base.to_path_buf();
        for part in *rel {
            p = p.join(part);
        }
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn mise_install_base() -> Option<PathBuf> {
    std::fs::canonicalize(&*MISE_BIN)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

pub static MISE_SELF_UPDATE_INSTRUCTIONS: Lazy<Option<PathBuf>> = Lazy::new(|| {
    if let Some(p) = var_path("MISE_SELF_UPDATE_INSTRUCTIONS") {
        return Some(p);
    }
    let base = mise_install_base()?;
    // search lib/, lib/mise/, lib64/mise/
    find_in_tree(
        &base,
        &[
            &["lib", "mise-self-update-instructions.toml"],
            &["lib", "mise", "mise-self-update-instructions.toml"],
            &["lib64", "mise", "mise-self-update-instructions.toml"],
        ],
    )
});
#[cfg(feature = "self_update")]
pub static MISE_SELF_UPDATE_AVAILABLE: Lazy<Option<bool>> = Lazy::new(|| {
    if var_is_true("MISE_SELF_UPDATE_AVAILABLE") {
        Some(true)
    } else if var_is_false("MISE_SELF_UPDATE_AVAILABLE") {
        Some(false)
    } else {
        None
    }
});
#[cfg(feature = "self_update")]
pub static MISE_SELF_UPDATE_DISABLED_PATH: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let base = mise_install_base()?;
    find_in_tree(
        &base,
        &[
            &["lib", ".disable-self-update"],
            &["lib", "mise", ".disable-self-update"],
            &["lib64", "mise", ".disable-self-update"],
        ],
    )
});
pub static MISE_LOG_HTTP: Lazy<bool> = Lazy::new(|| var_is_true("MISE_LOG_HTTP"));

pub static __USAGE: Lazy<Option<String>> = Lazy::new(|| var("__USAGE").ok());

// true if running inside a shim
pub static __MISE_SHIM: Lazy<bool> = Lazy::new(|| var_is_true("__MISE_SHIM"));

// true if the current process is running as a shim (not direct mise invocation)
pub static IS_RUNNING_AS_SHIM: Lazy<bool> = Lazy::new(|| {
    // When running tests, always treat as direct mise invocation
    // to avoid interfering with test expectations
    if cfg!(test) {
        return false;
    }

    // Check if running as tool stub
    if *MISE_TOOL_STUB {
        return true;
    }

    let bin_name = *MISE_BIN_NAME;
    !is_mise_binary(bin_name)
});

/// Returns true if the given binary name refers to mise itself (not a shim).
/// Handles "mise", "mise.exe", "mise.bat", "mise.cmd", "mise-doctor", etc.
pub fn is_mise_binary(bin_name: &str) -> bool {
    bin_name == "mise" || bin_name.starts_with("mise.") || bin_name.starts_with("mise-")
}

#[cfg(test)]
pub static TERM_WIDTH: Lazy<usize> = Lazy::new(|| 80);

#[cfg(not(test))]
pub static TERM_WIDTH: Lazy<usize> = Lazy::new(|| {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(80)
        .max(80)
});

/// true if inside a script like bin/exec-env or bin/install
/// used to prevent infinite loops
pub static MISE_BIN: Lazy<PathBuf> = Lazy::new(|| {
    var_path("__MISE_BIN")
        .or_else(|| current_exe().ok())
        .unwrap_or_else(|| "mise".into())
});
pub static MISE_TIMINGS: Lazy<u8> = Lazy::new(|| var_u8("MISE_TIMINGS"));
pub static MISE_PID: Lazy<String> = Lazy::new(|| process::id().to_string());
pub static MISE_JOBS: Lazy<Option<usize>> =
    Lazy::new(|| var("MISE_JOBS").ok().and_then(|v| v.parse::<usize>().ok()));
pub static __MISE_SCRIPT: Lazy<bool> = Lazy::new(|| var_is_true("__MISE_SCRIPT"));
pub static __MISE_DIFF: Lazy<EnvDiff> = Lazy::new(get_env_diff);
pub static __MISE_ORIG_PATH: Lazy<Option<String>> = Lazy::new(|| var("__MISE_ORIG_PATH").ok());
pub static __MISE_ZSH_PRECMD_RUN: Lazy<bool> = Lazy::new(|| !var_is_false("__MISE_ZSH_PRECMD_RUN"));
pub static LINUX_DISTRO: Lazy<Option<String>> = Lazy::new(linux_distro);
/// Detected glibc version on Linux as (major, minor), e.g. (2, 17).
/// Returns None on non-Linux or if detection fails.
pub static LINUX_GLIBC_VERSION: Lazy<Option<(u32, u32)>> = Lazy::new(linux_glibc_version);
pub static PREFER_OFFLINE: Lazy<AtomicBool> =
    Lazy::new(|| prefer_offline(&ARGS.read().unwrap()).into());
pub static OFFLINE: Lazy<bool> = Lazy::new(|| offline(&ARGS.read().unwrap()));
pub static WARN_ON_MISSING_REQUIRED_ENV: Lazy<bool> =
    Lazy::new(|| warn_on_missing_required_env(&ARGS.read().unwrap()));
/// essentially, this is whether we show spinners or build output on runtime install
pub static PRISTINE_ENV: Lazy<EnvMap> =
    Lazy::new(|| get_pristine_env(&__MISE_DIFF, vars_safe().collect()));
pub static PATH_KEY: Lazy<String> = Lazy::new(|| {
    vars_safe()
        .map(|(k, _)| k)
        .find_or_first(|k| k.to_uppercase() == "PATH")
        .map(|k| k.to_string())
        .unwrap_or("PATH".into())
});
pub static PATH: Lazy<Vec<PathBuf>> = Lazy::new(|| match PRISTINE_ENV.get(&*PATH_KEY) {
    Some(path) => split_paths(path).collect(),
    None => vec![],
});
pub static PATH_NON_PRISTINE: Lazy<Vec<PathBuf>> = Lazy::new(|| match var(&*PATH_KEY) {
    Ok(ref path) => split_paths(path).collect(),
    Err(_) => vec![],
});
pub static DIRENV_DIFF: Lazy<Option<String>> = Lazy::new(|| var("DIRENV_DIFF").ok());

pub static GITHUB_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| get_token(&["MISE_GITHUB_TOKEN", "GITHUB_API_TOKEN", "GITHUB_TOKEN"]));
pub static MISE_GITHUB_ENTERPRISE_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| get_token(&["MISE_GITHUB_ENTERPRISE_TOKEN"]));
pub static GITLAB_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| get_token(&["MISE_GITLAB_TOKEN", "GITLAB_TOKEN"]));
pub static MISE_GITLAB_ENTERPRISE_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| get_token(&["MISE_GITLAB_ENTERPRISE_TOKEN"]));
pub static FORGEJO_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| get_token(&["MISE_FORGEJO_TOKEN", "FORGEJO_TOKEN"]));
pub static MISE_FORGEJO_ENTERPRISE_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| get_token(&["MISE_FORGEJO_ENTERPRISE_TOKEN"]));

pub static TEST_TRANCHE: Lazy<usize> = Lazy::new(|| var_u8("TEST_TRANCHE") as usize);
pub static TEST_TRANCHE_COUNT: Lazy<usize> = Lazy::new(|| var_u8("TEST_TRANCHE_COUNT") as usize);

pub static CLICOLOR_FORCE: Lazy<Option<bool>> =
    Lazy::new(|| var("CLICOLOR_FORCE").ok().map(|v| v != "0"));

pub static CLICOLOR: Lazy<Option<bool>> = Lazy::new(|| {
    if *CLICOLOR_FORCE == Some(true) {
        Some(true)
    } else if *NO_COLOR || var_is_false("MISE_COLOR") {
        Some(false)
    } else if let Ok(v) = var("CLICOLOR") {
        Some(v != "0")
    } else {
        None
    }
});

/// Disable color output - https://no-color.org/
pub static NO_COLOR: Lazy<bool> = Lazy::new(|| var("NO_COLOR").is_ok_and(|v| !v.is_empty()));

/// Force progress bars even in non-TTY (for debugging)
pub static MISE_FORCE_PROGRESS: Lazy<bool> = Lazy::new(|| var_is_true("MISE_FORCE_PROGRESS"));

// python
pub static PYENV_ROOT: Lazy<PathBuf> =
    Lazy::new(|| var_path("PYENV_ROOT").unwrap_or_else(|| HOME.join(".pyenv")));
pub static UV_PYTHON_INSTALL_DIR: Lazy<PathBuf> = Lazy::new(|| {
    var_path("UV_PYTHON_INSTALL_DIR").unwrap_or_else(|| XDG_DATA_HOME.join("uv").join("python"))
});

#[cfg(unix)]
pub const PATH_ENV_SEP: char = ':';
#[cfg(windows)]
pub const PATH_ENV_SEP: char = ';';

fn get_env_diff() -> EnvDiff {
    let env = vars_safe().collect::<HashMap<_, _>>();
    match env.get("__MISE_DIFF") {
        Some(raw) => EnvDiff::deserialize(raw).unwrap_or_else(|err| {
            warn!("Failed to deserialize __MISE_DIFF: {:#}", err);
            EnvDiff::default()
        }),
        None => EnvDiff::default(),
    }
}

fn var_u8(key: &str) -> u8 {
    var(key)
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or_default()
}

fn var_is_true(key: &str) -> bool {
    match var(key) {
        Ok(v) => {
            let v = v.to_lowercase();
            v == "y" || v == "yes" || v == "true" || v == "1" || v == "on"
        }
        Err(_) => false,
    }
}

fn var_is_false(key: &str) -> bool {
    match var(key) {
        Ok(v) => {
            let v = v.to_lowercase();
            v == "n" || v == "no" || v == "false" || v == "0" || v == "off"
        }
        Err(_) => false,
    }
}

pub fn in_home_dir() -> bool {
    current_dir().is_ok_and(|d| d == *HOME)
}

pub fn var_path(key: &str) -> Option<PathBuf> {
    var_os(key).map(PathBuf::from).map(replace_path)
}

/// this returns the environment as if __MISE_DIFF was reversed.
/// putting the shell back into a state before hook-env was run
fn get_pristine_env(mise_diff: &EnvDiff, orig_env: EnvMap) -> EnvMap {
    let patches = mise_diff.reverse().to_patches();
    let mut env = apply_patches(&orig_env, &patches);

    // get the current path as a vector
    let path = match env.get(&*PATH_KEY) {
        Some(path) => split_paths(path).collect(),
        None => vec![],
    };
    // get the paths that were removed by mise as a hashset
    let mut to_remove = mise_diff.path.iter().collect::<HashSet<_>>();

    // remove those paths that were added by mise, but only once (the first time)
    let path = path
        .into_iter()
        .filter(|p| !to_remove.remove(p))
        .collect_vec();

    // put the pristine PATH back into the environment
    env.insert(
        PATH_KEY.to_string(),
        join_paths(path).unwrap().to_string_lossy().to_string(),
    );
    env
}

fn apply_patches(env: &EnvMap, patches: &EnvDiffPatches) -> EnvMap {
    let mut new_env = env.clone();
    for patch in patches {
        match patch {
            EnvDiffOperation::Add(k, v) | EnvDiffOperation::Change(k, v) => {
                new_env.insert(k.into(), v.into());
            }
            EnvDiffOperation::Remove(k) => {
                new_env.remove(k);
            }
        }
    }

    new_env
}

fn offline(args: &[String]) -> bool {
    if var_is_true("MISE_OFFLINE") {
        return true;
    }

    args.iter()
        .take_while(|a| *a != "--")
        .any(|a| a == "--offline")
}

/// returns true if new runtime versions should not be fetched
fn prefer_offline(args: &[String]) -> bool {
    // First check if MISE_PREFER_OFFLINE is set
    if var_is_true("MISE_PREFER_OFFLINE") {
        return true;
    }

    // Otherwise fall back to the original command-based logic
    args.iter()
        .take_while(|a| *a != "--")
        .filter(|a| !a.starts_with('-') || *a == "--prefer-offline")
        .nth(1)
        .map(|a| {
            [
                "--prefer-offline",
                "activate",
                "current",
                "direnv",
                "env",
                "exec",
                "hook-env",
                "ls",
                "where",
                "x",
            ]
            .contains(&a.as_str())
        })
        .unwrap_or_default()
}

/// returns true if missing required env vars should produce warnings instead of errors
fn warn_on_missing_required_env(args: &[String]) -> bool {
    // Check if we're running in a command that should warn instead of error
    args.iter()
        .take_while(|a| *a != "--")
        .filter(|a| !a.starts_with('-'))
        .nth(1)
        .map(|a| {
            [
                "hook-env", // Shell activation should not break the shell
            ]
            .contains(&a.as_str())
        })
        .unwrap_or_default()
}

fn environment(args: &[String]) -> Vec<String> {
    let arg_defs = HashSet::from(["--profile", "-P", "--env", "-E"]);

    // Get environment value from args or env vars
    // Precedence: CLI args > env vars > .miserc.toml
    let from_args = if *IS_RUNNING_AS_SHIM {
        // When running as shim, ignore command line args and use env vars only
        vec![]
    } else {
        // Subcommands where positional args accept hyphen values, so -E after the
        // first positional would be a task arg, not a global flag.
        let run_subcommands: HashSet<&str> = HashSet::from(["run", "r"]);
        // Try to get from command line args first
        // Handles both `--env production` (separate args) and `--env=production` (joined with =)
        let mut values = Vec::new();
        let mut it = args.iter().take_while(|a| a.as_str() != "--");
        let mut in_run_subcommand = false;
        while let Some(arg) = it.next() {
            if let Some((flag, value)) = arg.split_once('=') {
                if arg_defs.contains(flag) {
                    values.push(value.to_string());
                }
            } else if arg_defs.contains(arg.as_str()) {
                if let Some(next) = it.next() {
                    values.push(next.to_string());
                }
            } else if !arg.starts_with('-') {
                // After `run`/`r`, the first positional is the task name — everything
                // after that belongs to the task, so stop scanning for env flags.
                if in_run_subcommand {
                    break;
                }
                if run_subcommands.contains(arg.as_str()) {
                    in_run_subcommand = true;
                }
            }
        }
        values
            .into_iter()
            .flat_map(|s| {
                s.split(',')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    if !from_args.is_empty() {
        return from_args;
    }
    var("MISE_ENV")
        .ok()
        .or_else(|| var("MISE_PROFILE").ok())
        .or_else(|| var("MISE_ENVIRONMENT").ok())
        .map(|s| {
            s.split(',')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
        .or_else(|| miserc::get_env().cloned())
        .unwrap_or_default()
}

fn log_file_level() -> Option<LevelFilter> {
    let log_level = var("MISE_LOG_FILE_LEVEL").unwrap_or_default();
    log_level.parse::<LevelFilter>().ok()
}

fn linux_distro() -> Option<String> {
    match sys_info::linux_os_release() {
        Ok(release) => release.id,
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn linux_glibc_version() -> Option<(u32, u32)> {
    let output = std::process::Command::new("ldd")
        .arg("--version")
        .output()
        .ok()?;
    // ldd --version prints to stdout on glibc, stderr on some systems
    let text = String::from_utf8_lossy(&output.stdout);
    let text = if text.is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        text
    };
    let first_line = text.lines().next()?;
    let version_str = first_line.rsplit(' ').next()?;
    let mut parts = version_str.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    debug!("detected glibc version: {}.{}", major, minor);
    Some((major, minor))
}

#[cfg(not(target_os = "linux"))]
fn linux_glibc_version() -> Option<(u32, u32)> {
    None
}

fn filename(path: &str) -> &str {
    path.rsplit_once(path::MAIN_SEPARATOR_STR)
        .map(|(_, file)| file)
        .unwrap_or(path)
}

fn get_token(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| var(key).ok())
        .and_then(|v| if v.trim().is_empty() { None } else { Some(v) })
}

pub fn is_activated() -> bool {
    var("__MISE_DIFF").is_ok()
}

pub fn set_var<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
    static MUTEX: Mutex<()> = Mutex::new(());
    let _mutex = MUTEX.lock().unwrap();
    unsafe {
        std::env::set_var(key, value);
    }
}

pub fn remove_var<K: AsRef<OsStr>>(key: K) {
    static MUTEX: Mutex<()> = Mutex::new(());
    let _mutex = MUTEX.lock().unwrap();
    unsafe {
        std::env::remove_var(key);
    }
}

/// Remove the env cache encryption key to force fresh env computation
pub fn reset_env_cache_key() {
    remove_var("__MISE_ENV_CACHE_KEY");
}

/// Safe wrapper around std::env::vars() that handles invalid UTF-8 gracefully.
/// This function uses vars_os() and converts OsString to String, skipping any
/// environment variables that contain invalid UTF-8 sequences.
pub fn vars_safe() -> impl Iterator<Item = (String, String)> {
    vars_os().filter_map(|(k, v)| {
        let k_str = k.to_str()?;
        let v_str = v.to_str()?;
        Some((k_str.to_string(), v_str.to_string()))
    })
}

pub fn set_current_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    trace!("cd {}", display_path(path));
    unsafe {
        std::env::set_current_dir(path).wrap_err_with(|| {
            format!("failed to set current directory to {}", display_path(path))
        })?;
        path_absolutize::update_cwd();
    }
    Ok(())
}

/// Returns the default Windows system directory: `%PROGRAMDATA%\mise`.
/// Used as the default for both `MISE_SYSTEM_CONFIG_DIR` and `MISE_SYSTEM_DATA_DIR` on Windows.
#[cfg(windows)]
pub(crate) fn windows_programdata_mise() -> PathBuf {
    var_path("PROGRAMDATA")
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
        .join("mise")
}

/// Checks whether `path` is both:
/// 1. Owned by `BUILTIN\Administrators` or `NT AUTHORITY\SYSTEM`, AND
/// 2. Not writable by the `BUILTIN\Users` group.
///
/// Returns `Ok(true)` if admin-controlled, `Ok(false)` if not, `Err` if check failed.
#[cfg(windows)]
fn windows_dir_admin_controlled(path: &Path) -> Result<bool> {
    use std::os::windows::ffi::OsStrExt;
    use winapi::shared::accctrl::{SE_FILE_OBJECT, TrusteeIsSid, TRUSTEE_W};
    use winapi::um::aclapi::{GetEffectiveRightsFromAclW, GetNamedSecurityInfoW};
    use winapi::um::securitybaseapi::{CreateWellKnownSid, IsWellKnownSid};
    use winapi::um::winbase::LocalFree;
    use winapi::um::winnt::{
        WinBuiltinAdministratorsSid, WinBuiltinUsersSid, WinLocalSystemSid,
        DACL_SECURITY_INFORMATION, DELETE, FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_WRITE_DATA,
        OWNER_SECURITY_INFORMATION, PACL, PSID, WRITE_DAC, WRITE_OWNER,
    };
    // ERROR_SUCCESS = 0; avoids adding a winerror feature dependency
    const ERROR_SUCCESS: u32 = 0;

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0u16))
        .collect();
    let mut owner_sid: PSID = std::ptr::null_mut();
    let mut dacl: PACL = std::ptr::null_mut();
    let mut sd = std::ptr::null_mut();

    let err = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner_sid,
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut sd,
        )
    };
    if err != ERROR_SUCCESS {
        if !sd.is_null() {
            unsafe { LocalFree(sd as _) };
        }
        return Err(eyre::eyre!(
            "GetNamedSecurityInfoW failed for {}: error {}",
            path.display(),
            err
        ));
    }

    // Check 1: owner must be Administrators or SYSTEM
    let owner_privileged = unsafe {
        IsWellKnownSid(owner_sid, WinBuiltinAdministratorsSid) != 0
            || IsWellKnownSid(owner_sid, WinLocalSystemSid) != 0
    };

    // Check 2: BUILTIN\Users must not have write access to the directory.
    // Build a Users SID for the trustee.
    let mut users_sid_buf = [0u8; 68]; // SECURITY_MAX_SID_SIZE
    let mut sid_size = users_sid_buf.len() as u32;
    let users_sid_ok = unsafe {
        CreateWellKnownSid(
            WinBuiltinUsersSid,
            std::ptr::null_mut(),
            users_sid_buf.as_mut_ptr() as PSID,
            &mut sid_size,
        ) != 0
    };

    let users_no_write = if users_sid_ok && !dacl.is_null() {
        let mut trustee: TRUSTEE_W = unsafe { std::mem::zeroed() };
        trustee.TrusteeForm = TrusteeIsSid; // u32 const: 0
        // TrusteeType is ignored by GetEffectiveRightsFromAclW; zeroed = TRUSTEE_IS_UNKNOWN
        trustee.ptstrName = users_sid_buf.as_mut_ptr() as *mut _;
        let mut access_rights: u32 = 0;
        let err2 =
            unsafe { GetEffectiveRightsFromAclW(dacl, &mut trustee, &mut access_rights) };
        // GENERIC_WRITE is never stored directly in ACEs; check specific write rights only
        let write_mask =
            FILE_WRITE_DATA | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY | WRITE_DAC | WRITE_OWNER | DELETE;
        err2 == ERROR_SUCCESS && (access_rights & write_mask) == 0
    } else {
        false // Could not verify — treat as untrusted
    };

    unsafe { LocalFree(sd as _) };
    Ok(owner_privileged && users_no_write)
}

/// True when the default Windows system directory (`%PROGRAMDATA%\mise`) is safe to use.
///
/// Returns true when:
/// - Any system-dir env var is explicitly set (user opted in — no ownership check needed), OR
/// - The directory does not exist (admin hasn't set it up yet), OR
/// - The directory passes both ownership and write-restriction checks.
///
/// Returns false (and emits a warning) when the directory exists but fails the security check.
#[cfg(windows)]
pub(crate) static WINDOWS_SYSTEM_DIR_TRUSTED: Lazy<bool> = Lazy::new(|| {
    // If any system-dir env var is explicitly set, the user opted in — skip the check
    if var_path("MISE_SYSTEM_CONFIG_DIR")
        .or_else(|| var_path("MISE_SYSTEM_DIR"))
        .or_else(|| var_path("MISE_SYSTEM_DATA_DIR"))
        .is_some()
    {
        return true;
    }
    let dir = windows_programdata_mise();
    if !dir.is_dir() {
        return true; // Directory doesn't exist — no threat yet
    }
    match windows_dir_admin_controlled(&dir) {
        Ok(true) => true,
        Ok(false) => {
            warn!(
                "mise: ignoring {} as the system directory: it must be owned by \
                 BUILTIN\\Administrators or NT AUTHORITY\\SYSTEM and not be writable \
                 by standard users. Set MISE_SYSTEM_CONFIG_DIR to override.",
                dir.display()
            );
            false
        }
        Err(err) => {
            warn!(
                "mise: could not verify ownership of system directory {}: {err:#}. \
                 Ignoring as a precaution. Set MISE_SYSTEM_CONFIG_DIR to override.",
                dir.display()
            );
            false
        }
    }
});

/// Returns the system config directory if it should be trusted, or `None` if it should be
/// skipped (Windows untrusted dir). On non-Windows this always returns `Some`.
/// Use this instead of `MISE_SYSTEM_CONFIG_DIR` in any code that loads or executes content
/// from the system config directory.
pub fn system_config_dir() -> Option<&'static Path> {
    #[cfg(windows)]
    if !*WINDOWS_SYSTEM_DIR_TRUSTED {
        return None;
    }
    Some(&MISE_SYSTEM_CONFIG_DIR)
}

/// Returns the system data directory if it should be trusted, or `None` if it should be
/// skipped (Windows untrusted dir). On non-Windows this always returns `Some`.
/// Use this instead of `MISE_SYSTEM_DATA_DIR` in any code that reads shims or tool installs
/// from the system data directory.
pub fn system_data_dir() -> Option<&'static Path> {
    #[cfg(windows)]
    if !*WINDOWS_SYSTEM_DIR_TRUSTED {
        return None;
    }
    Some(&MISE_SYSTEM_DATA_DIR)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn test_apply_patches() {
        let _config = Config::get().await.unwrap();
        let mut env = EnvMap::new();
        env.insert("foo".into(), "bar".into());
        env.insert("baz".into(), "qux".into());
        let patches = vec![
            EnvDiffOperation::Add("foo".into(), "bar".into()),
            EnvDiffOperation::Change("baz".into(), "qux".into()),
            EnvDiffOperation::Remove("quux".into()),
        ];
        let new_env = apply_patches(&env, &patches);
        assert_eq!(new_env.len(), 2);
        assert_eq!(new_env.get("foo").unwrap(), "bar");
        assert_eq!(new_env.get("baz").unwrap(), "qux");
    }

    #[tokio::test]
    async fn test_var_path() {
        let _config = Config::get().await.unwrap();
        set_var("MISE_TEST_PATH", "/foo/bar");
        assert_eq!(
            var_path("MISE_TEST_PATH").unwrap(),
            PathBuf::from("/foo/bar")
        );
        remove_var("MISE_TEST_PATH");
    }

    #[test]
    fn test_token_overwrite() {
        // Clean up any existing environment variables that might interfere
        remove_var("MISE_GITHUB_TOKEN");
        remove_var("GITHUB_TOKEN");
        remove_var("GITHUB_API_TOKEN");

        set_var("MISE_GITHUB_TOKEN", "");
        set_var("GITHUB_TOKEN", "invalid_token");
        assert_eq!(
            get_token(&["MISE_GITHUB_TOKEN", "GITHUB_TOKEN"]),
            None,
            "Empty token should overwrite other tokens"
        );
        assert_eq!(
            get_token(&["GITHUB_API_TOKEN", "GITHUB_TOKEN"]),
            Some("invalid_token".into()),
            "Unset token should not overwrite other tokens"
        );
        remove_var("MISE_GITHUB_TOKEN");
        remove_var("GITHUB_TOKEN");
        remove_var("GITHUB_API_TOKEN");
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn test_nonexistent_dir_returns_err() {
        let result = windows_dir_admin_controlled(Path::new(r"C:\does_not_exist_mise_test_xyz"));
        assert!(result.is_err(), "non-existent path should return Err");
    }

    #[test]
    fn test_system32_is_admin_controlled() {
        // C:\Windows\System32 is always owned by TrustedInstaller or SYSTEM and not user-writable
        let result = windows_dir_admin_controlled(Path::new(r"C:\Windows\System32"));
        assert_eq!(
            result.unwrap_or(false),
            true,
            "System32 should be admin-controlled"
        );
    }

    #[test]
    fn test_user_created_temp_dir_is_not_admin_controlled() {
        // A dir created by the current non-admin user is owned by that user, writable by Users
        let tmp = std::env::temp_dir().join("mise_admin_check_test_xyz");
        let _ = std::fs::create_dir_all(&tmp);
        let result = windows_dir_admin_controlled(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);
        // Only valid when tests run without admin rights (normal in CI)
        assert_eq!(
            result.unwrap_or(true),
            false,
            "User-created temp dir should not be admin-controlled"
        );
    }
}
