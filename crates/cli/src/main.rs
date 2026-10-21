use heart::{
    dashboard,
    session::{self, Action, Session},
};
use heart_core::{Engine, load_profiles, ollama::Ollama};
use heart_websearch::WebSearch;
use std::{
    env,
    io::{self, Write},
    time::Duration,
};
fn main() {
    // Environment wins; .env.local is optional and never printed.
    if std::path::Path::new(".env.local").exists() && dotenvy::from_filename(".env.local").is_err()
    {
        eprintln!("Could not load .env.local; check its syntax.");
        return;
    }
    let mut model = env::var("HEART_MODEL").unwrap_or_else(|_| "qwen3.5:2b-q4_K_M".into());
    let mut endpoint = env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
    let mut profile_id = None;
    let mut dashboard = false;
    let mut animations = true;
    let mut profiles_dir = std::path::PathBuf::from("profiles");
    let mut experimental_nl = false;
    let mut diagnostics = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dashboard" => dashboard = true,
            "--no-animation" => animations = false,
            "--diagnostics" => diagnostics = true,
            "--model" => {
                let Some(value) = args.next() else {
                    return usage();
                };
                model = value;
            }
            "--endpoint" => {
                let Some(value) = args.next() else {
                    return usage();
                };
                endpoint = value;
            }
            "--personality" => {
                let Some(value) = args.next() else {
                    return usage();
                };
                profile_id = Some(value);
            }
            "--profiles-dir" => {
                let Some(value) = args.next() else {
                    return usage();
                };
                profiles_dir = value.into();
            }
            "--experimental-nl" => experimental_nl = true,
            "--help" | "-h" => return usage(),
            _ => return usage(),
        }
    }
    let profile_id = profile_id.unwrap_or_else(|| if dashboard { "cat" } else { "butler" }.into());
    if dashboard
        && (!std::io::IsTerminal::is_terminal(&io::stdin())
            || !std::io::IsTerminal::is_terminal(&io::stdout()))
    {
        eprintln!("Dashboard requires an interactive terminal (TTY) on stdin and stdout.");
        return;
    }
    let seconds = env::var("HEART_IDLE_SECONDS")
        .unwrap_or_else(|_| "300".into())
        .parse::<u64>();
    let seconds = match seconds {
        Ok(n) if n <= 86400 => n,
        _ => {
            eprintln!("HEART_IDLE_SECONDS must be between 0 and 86400.");
            return;
        }
    };
    let inference = match Ollama::new(
        &endpoint,
        &model,
        Duration::from_secs(30),
        Duration::from_secs(120),
    ) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let mut engine = Engine::new(inference, Duration::from_secs(seconds));
    let module = match WebSearch::new(env::var("TAVILY_API_KEY").ok()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    if let Err(e) = engine.register(Box::new(module)) {
        eprintln!("{e}");
        return;
    }
    for module in [
        Box::new(heart_system::DiskSpace) as Box<dyn heart_contract::Module>,
        Box::new(heart_system::SystemInfo),
        Box::new(heart_system::Uptime),
    ] {
        if let Err(e) = engine.register(module) {
            eprintln!("{e}");
            return;
        }
    }
    let profiles = match load_profiles(&profiles_dir, engine.commands().into_iter().map(|c| c.name))
    {
        Ok(profiles) => profiles,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };
    if !profiles.contains_key(&profile_id) {
        eprintln!("Unknown initial profile: {profile_id}");
        return;
    }
    let help = format!(
        "{}\n/profile ID · /ID · /help · /exit\nProfiles: {}",
        engine.command_help().join("\n"),
        profiles.keys().cloned().collect::<Vec<_>>().join(", ")
    );
    let mut session = Session {
        profiles,
        selected: profile_id,
        help,
        experimental_nl,
        diagnostics,
    };
    if dashboard {
        if let Err(error) = dashboard::run(engine, session, animations) {
            eprintln!("Dashboard: {error}");
        }
        return;
    }
    println!(
        "HEART · {} · {model}\n{}\nUse /help for commands.",
        session.profile().name(),
        session.profile().greeting()
    );
    loop {
        print!("> ");
        if io::stdout().flush().is_err() {
            break;
        }
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            _ => {}
        }
        match session.classify(&line) {
            Action::Empty => {}
            Action::Exit => break,
            Action::Help => println!("{}", session.help),
            Action::Profile(text) | Action::Message(text) => println!("{text}"),
            Action::Request(request) => {
                match session::execute(&mut engine, &request, session.profile(), |text| {
                    println!("{text}")
                }) {
                    Ok(reply) => {
                        println!("{}", reply.text);
                        if diagnostics {
                            eprintln!("{}", session::diagnostic(&reply));
                        }
                        if let Some(warning) = reply.warning {
                            eprintln!("{warning}");
                        }
                        for (i, source) in reply.sources.iter().enumerate() {
                            println!("[{}] {} — {}", i + 1, source.title, source.url);
                        }
                    }
                    Err(error) => println!("{error}"),
                }
            }
        }
    }
}

fn usage() {
    println!(
        "heart [--dashboard] [--no-animation] [--model NAME] [--endpoint URL] [--personality ID] [--profiles-dir PATH] [--experimental-nl] [--diagnostics]\nEnvironment: HEART_MODEL, OLLAMA_HOST, HEART_IDLE_SECONDS (default 300), TAVILY_API_KEY\nOptional project .env.local; model installation is manual."
    );
}
