use serde::Deserialize;
use std::{
    collections::HashSet,
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Deserialize)]
struct Config {
    import: Option<Vec<String>>,
    session: Option<Vec<Session>>,
}

#[derive(Debug, Deserialize, Clone)]
struct Session {
    name: String,
    path: Option<String>,
    startup_command: String,
}

fn sesh_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(".config")
            .join("sesh")
            .join("sesh.toml")
    })
}

fn print_help() {
    println!(
        "\n\x1b[1mdsesh v1.1\x1b[0m  06/02/2026  SBDJ\n\
         \ndsesh is a terminal session manager designed to be compatible with Sesh TOML configurations.\n\
         \nUSAGE:\n  \x1b[32mdsesh [command]\x1b[0m\n\
         \nCOMMANDS:\n  \x1b[33mconnect\x1b[0m    Connect to the given session\n  \x1b[33mlist\x1b[0m       List sessions\n"
    );
}

fn expand_tilde(path: &str) -> PathBuf {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return PathBuf::from(path),
    };

    match path {
        "~" => home,
        p if p.starts_with("~/") => home.join(&p[2..]),
        _ => PathBuf::from(path),
    }
}

fn load_config_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> anyhow::Result<Vec<Session>> {
    let path = fs::canonicalize(path)?;

    // Prevent circular imports
    if !visited.insert(path.clone()) {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;

    let mut sessions = Vec::new();

    // Load imports first
    if let Some(imports) = config.import {
        for import in imports {
            let import_path = expand_tilde(&import);
            let import_path = if import_path.is_relative() {
                path.parent()
                    .unwrap()
                    .join(import_path)
            } else {
                import_path
            };
            sessions.extend(load_config_recursive(&import_path, visited)?);
        }
    }

    // Add local sessions
    if let Some(mut local_sessions) = config.session {
        sessions.append(&mut local_sessions);
    }

    Ok(sessions)
}

fn load_all_sessions(path: impl AsRef<Path>) -> anyhow::Result<Vec<Session>> {
    let mut visited = HashSet::new();
    load_config_recursive(path.as_ref(), &mut visited)
}

fn list_sessions(sessions: &[Session], filter: Option<&str>) {
    for s in sessions {
        if let Some(f) = filter {
            if !s.name.to_lowercase().contains(&f.to_lowercase()) {
                continue;
            }
        }
        println!("{}", s.name);
    }
}

fn find_session<'a>(sessions: &'a [Session], name: &str) -> Option<&'a Session> {
    let name = name.trim();
    sessions.iter().find(|s| s.name.trim() == name)
}

fn connect_session(session: &Session) -> anyhow::Result<()> {
    eprintln!("→ {}", session.name);

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&session.startup_command);

    if let Some(ref path) = session.path {
        cmd.current_dir(expand_tilde(path));
    } else {
        // default to current working directory
        cmd.current_dir(env::current_dir()?);
    }

    cmd.status()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let path = sesh_config_path()
        .ok_or_else(|| anyhow::anyhow!("Could not locate sesh config"))?;

    let sessions = load_all_sessions(&path)?;

    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("list") => {
            let filter = args.next();
            list_sessions(&sessions, filter.as_deref());
        }
        Some("connect") => {
            let name = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("connect requires a session name"))?;

            if name == "" {
                return Ok(())
            }

            let session = find_session(&sessions, &name)
                .ok_or_else(|| anyhow::anyhow!("No such session: {}", name))?;

            connect_session(session)?;
        }
        None => {
            print_help();
        }
        Some(cmd) => {
            anyhow::bail!("Unknown command: {}", cmd);
        }
    }

    Ok(())
}
