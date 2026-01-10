//! Erold CLI - A learning development agent
//!
//! An AI coding assistant that:
//! - Gets smarter over time through saved learnings
//! - Always uses fresh knowledge with TTL-based expiration
//! - Fetches relevant context per-subtask
//! - Requires human approval before making changes

mod agent;
mod ui;

use clap::{Parser, Subcommand};
use erold_api::{EroldClient, Project};
use erold_config::{ConfigLoader, Credentials, ProjectLink};
use std::io::{self, Write};
use tracing::info;
use tracing_subscriber::EnvFilter;

use agent::Agent;
use ui::{AutoApproveUI, ConsoleUI};

/// Erold CLI - A learning development agent
#[derive(Parser)]
#[command(name = "erold")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Auto-approve plans without prompting
    #[arg(long, global = true)]
    auto_approve: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure API credentials
    Login,

    /// Link current directory to an Erold project
    Link {
        /// Project ID to link to
        project_id: Option<String>,
    },

    /// Show current configuration
    Config,

    /// Run a task
    Run {
        /// Task description
        task: String,

        /// Auto-approve the plan without prompting
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Unlink current directory from project
    Unlink,
}

fn print_banner() {
    println!(
        "
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ███████╗██████╗  ██████╗ ██╗     ██████╗                   ║
║   ██╔════╝██╔══██╗██╔═══██╗██║     ██╔══██╗                  ║
║   █████╗  ██████╔╝██║   ██║██║     ██║  ██║                  ║
║   ██╔══╝  ██╔══██╗██║   ██║██║     ██║  ██║                  ║
║   ███████╗██║  ██║╚██████╔╝███████╗██████╔╝                  ║
║   ╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═════╝                   ║
║                                                               ║
║   A Learning Development Agent                                ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
"
    );
}

/// Prompt for a value with optional default
fn prompt(message: &str, default: Option<&str>) -> io::Result<String> {
    let prompt_text = match default {
        Some(d) => format!("{message} [{d}]: "),
        None => format!("{message}: "),
    };
    print!("{prompt_text}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    Ok(if input.is_empty() {
        default.unwrap_or("").to_string()
    } else {
        input
    })
}

/// Prompt for a secret value (API key)
fn prompt_secret(message: &str) -> io::Result<String> {
    print!("{message}: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Ensure user is logged in, prompting for credentials if not
async fn ensure_credentials() -> anyhow::Result<Credentials> {
    if let Ok(creds) = Credentials::load() {
        if !creds.openai_api_key.is_empty() {
            return Ok(creds);
        }
    }

    println!("Welcome to Erold! Let's set up your credentials.\n");
    do_login().await
}

/// Ensure project is linked, prompting user to create or link if not
async fn ensure_project_link(creds: &Credentials) -> anyhow::Result<ProjectLink> {
    let cwd = std::env::current_dir()?;

    // Check if already linked
    if let Ok((_, link)) = ConfigLoader::find_project_link(&cwd) {
        return Ok(link);
    }

    println!("\nNo project linked to this directory.");
    println!("Would you like to:\n");
    println!("  [1] Link to an existing project");
    println!("  [2] Create a new project\n");

    let choice = prompt("Select option", Some("1"))?;

    let config = ConfigLoader::load()?;
    let client = EroldClient::new(&config.api.url, &creds.erold_api_key, &creds.tenant_id)?;

    match choice.as_str() {
        "2" => {
            // Create new project
            let project = create_new_project(&client).await?;
            let link = save_project_link(&creds.tenant_id, &project.id, &project.title)?;
            Ok(link)
        }
        _ => {
            // Link to existing
            let project = select_existing_project(&client).await?;
            let link = save_project_link(&creds.tenant_id, &project.id, &project.title)?;
            Ok(link)
        }
    }
}

/// Create a new project in Erold
async fn create_new_project(client: &EroldClient) -> anyhow::Result<Project> {
    println!("\nCreate a new project:\n");

    let name = prompt("Project name", None)?;
    if name.is_empty() {
        anyhow::bail!("Project name is required");
    }

    let description = prompt("Description (optional)", None)?;
    let desc = if description.is_empty() { None } else { Some(description.as_str()) };

    print!("\nCreating project... ");
    io::stdout().flush()?;

    let project = client.create_project(&name, desc).await?;
    println!("OK");

    Ok(project)
}

/// Truncate and clean text for display (strip markdown, limit length)
fn clean_description(text: &str, max_len: usize) -> String {
    // Take first line only, strip markdown headers and code blocks
    let first_line = text.lines()
        .find(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("```")
                && !trimmed.starts_with("**")
                && !trimmed.starts_with("- ")
                && !trimmed.starts_with("* ")
        })
        .unwrap_or("")
        .trim();

    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len])
    } else {
        first_line.to_string()
    }
}

/// Select an existing project from the list
async fn select_existing_project(client: &EroldClient) -> anyhow::Result<Project> {
    print!("Fetching projects... ");
    io::stdout().flush()?;
    let projects = client.list_projects().await?;
    println!("OK\n");

    if projects.is_empty() {
        anyhow::bail!("No projects found. Select option [2] to create one.");
    }

    println!("Available projects:\n");
    for (i, project) in projects.iter().enumerate() {
        let status = format!("{:?}", project.status).to_lowercase();
        let tasks = project.task_count.unwrap_or(0);

        // Format: [1] Project Name (status, N tasks)
        print!("  [{:2}] {}", i + 1, project.title);

        // Add status and task count in dimmer style
        println!("  ({}, {} tasks)", status, tasks);

        // Show cleaned description if available (max 60 chars)
        if let Some(ref desc) = project.description {
            let clean = clean_description(desc, 60);
            if !clean.is_empty() {
                println!("       {}", clean);
            }
        }
    }

    println!();
    let selection = prompt("Select project number", Some("1"))?;
    let index: usize = selection.parse().map_err(|_| anyhow::anyhow!("Invalid selection"))?;

    if index == 0 || index > projects.len() {
        anyhow::bail!("Invalid selection: {index}");
    }

    Ok(projects[index - 1].clone())
}

/// Full setup check - ensures credentials and project link
async fn ensure_setup() -> anyhow::Result<(Credentials, ProjectLink)> {
    let creds = ensure_credentials().await?;
    let link = ensure_project_link(&creds).await?;
    Ok((creds, link))
}

/// Perform the login flow and return credentials
async fn do_login() -> anyhow::Result<Credentials> {
    println!("Configure Erold API credentials\n");

    let erold_api_key = prompt_secret("Erold API key")?;
    if erold_api_key.is_empty() {
        anyhow::bail!("Erold API key is required");
    }

    let tenant_id = prompt("Tenant ID", None)?;
    if tenant_id.is_empty() {
        anyhow::bail!("Tenant ID is required");
    }

    let openai_api_key = prompt_secret("OpenAI API key (for GPT models)")?;
    if openai_api_key.is_empty() {
        anyhow::bail!("OpenAI API key is required");
    }

    // Validate credentials
    print!("\nValidating credentials... ");
    io::stdout().flush()?;

    let config = ConfigLoader::load()?;
    let client = EroldClient::new(&config.api.url, &erold_api_key, &tenant_id)?;

    match client.list_projects().await {
        Ok(projects) => {
            println!("OK");
            println!("Found {} project(s)", projects.len());

            let creds = Credentials {
                erold_api_key,
                tenant_id,
                openai_api_key,
            };
            creds.save()?;
            println!("\nCredentials saved to ~/.erold/credentials.toml");

            Ok(creds)
        }
        Err(e) => {
            println!("FAILED");
            anyhow::bail!("Failed to validate credentials: {}", e);
        }
    }
}

async fn cmd_login() -> anyhow::Result<()> {
    do_login().await?;
    Ok(())
}

async fn cmd_link(project_id: Option<String>) -> anyhow::Result<()> {
    // Ensure credentials first
    let creds = ensure_credentials().await?;

    let config = ConfigLoader::load()?;
    let client = EroldClient::new(&config.api.url, &creds.erold_api_key, &creds.tenant_id)?;

    // If project_id provided, link directly
    if let Some(pid) = project_id {
        let project = client.get_project(&pid).await?;
        save_project_link(&creds.tenant_id, &project.id, &project.title)?;
        return Ok(());
    }

    // Otherwise, select from list
    let project = select_existing_project(&client).await?;
    save_project_link(&creds.tenant_id, &project.id, &project.title)?;

    Ok(())
}

fn save_project_link(tenant_id: &str, project_id: &str, project_name: &str) -> anyhow::Result<ProjectLink> {
    let cwd = std::env::current_dir()?;
    let link = ProjectLink {
        project_id: project_id.to_string(),
        project_name: project_name.to_string(),
        tenant_id: tenant_id.to_string(),
        linked_at: chrono::Utc::now().to_rfc3339(),
    };

    ConfigLoader::save_project_link(&cwd, &link)?;

    println!("\nLinked to project: {project_name}");
    println!("Project ID: {project_id}");
    println!("Config saved to: .erold/project.json");

    Ok(link)
}

fn cmd_config() -> anyhow::Result<()> {
    println!("Erold Configuration\n");
    println!("===================\n");

    // Show global config
    match ConfigLoader::load() {
        Ok(config) => {
            println!("Global Config (~/.erold/config.toml):");
            println!("  API URL: {}", config.api.url);
            println!("  API Timeout: {}s", config.api.timeout_secs);
            println!();
            println!("Workflow:");
            println!("  Require Plan: {}", config.workflow.require_plan);
            println!("  Require Approval: {}", config.workflow.require_approval);
            println!("  Read Before Edit: {}", config.workflow.require_read_before_edit);
            println!("  Auto Enrich: {}", config.workflow.auto_enrich);
            println!();
            println!("LLM:");
            println!("  Model: {}", config.llm.model);
            println!("  Max Tokens: {}", config.llm.max_tokens);
            println!("  Temperature: {}", config.llm.temperature);
        }
        Err(e) => {
            println!("Config: Not configured ({})", e);
        }
    }

    println!();

    // Show credentials status
    if Credentials::exists() {
        println!("Credentials: Configured");
        if let Ok(creds) = Credentials::load() {
            println!("  Tenant ID: {}", creds.tenant_id);
            println!("  Erold API Key: {}...", &creds.erold_api_key[..8.min(creds.erold_api_key.len())]);
            println!("  OpenAI API Key: {}...", &creds.openai_api_key[..8.min(creds.openai_api_key.len())]);
        }
    } else {
        println!("Credentials: Not configured (run 'erold login')");
    }

    println!();

    // Show project link
    let cwd = std::env::current_dir()?;
    match ConfigLoader::find_project_link(&cwd) {
        Ok((path, link)) => {
            println!("Project Link:");
            println!("  Project: {} ({})", link.project_name, link.project_id);
            println!("  Tenant: {}", link.tenant_id);
            println!("  Config: {}/.erold/project.json", path.display());
        }
        Err(_) => {
            println!("Project: Not linked (run 'erold link')");
        }
    }

    Ok(())
}

async fn cmd_run(task: &str, auto_approve: bool) -> anyhow::Result<()> {
    info!("Running task: {}", task);

    // Ensure setup (credentials + project link)
    let (creds, project_link) = ensure_setup().await?;
    let cwd = std::env::current_dir()?;
    let config = ConfigLoader::load()?;

    // Create agent
    let agent = Agent::from_credentials(
        &creds,
        &config,
        &project_link.project_id,
        cwd,
    )?;

    // Set UI based on auto-approve flag
    let agent = if auto_approve {
        agent.with_ui(Box::new(AutoApproveUI::new()))
    } else {
        agent.with_ui(Box::new(ConsoleUI::new()))
    };

    // Run the task
    agent.run(task).await?;

    Ok(())
}

async fn cmd_interactive(auto_approve: bool) -> anyhow::Result<()> {
    print_banner();

    // Ensure setup (credentials + project link) - will prompt if needed
    let (creds, project_link) = ensure_setup().await?;
    let cwd = std::env::current_dir()?;
    let config = ConfigLoader::load()?;

    println!("Project: {}", project_link.project_name);
    println!("Model: {}", config.llm.model);
    println!();
    println!("Type a task and press Enter. Type 'quit' to exit.\n");

    loop {
        let input = prompt(">", None)?;

        if input.is_empty() {
            continue;
        }

        let input_lower = input.to_lowercase();
        if input_lower == "quit" || input_lower == "exit" || input_lower == "q" {
            println!("Goodbye!");
            break;
        }

        // Create fresh agent for each task
        let agent = Agent::from_credentials(
            &creds,
            &config,
            &project_link.project_id,
            cwd.clone(),
        )?;

        let agent = if auto_approve {
            agent.with_ui(Box::new(AutoApproveUI::new()))
        } else {
            agent.with_ui(Box::new(ConsoleUI::new()))
        };

        // Run the task
        if let Err(e) = agent.run(&input).await {
            eprintln!("\nError: {e}\n");
        }

        println!();
    }

    Ok(())
}

fn cmd_unlink() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let erold_dir = cwd.join(".erold");
    let project_file = erold_dir.join("project.json");

    if project_file.exists() {
        std::fs::remove_file(&project_file)?;
        println!("Project unlinked from this directory.");

        // Remove .erold dir if empty
        if erold_dir.read_dir()?.next().is_none() {
            std::fs::remove_dir(&erold_dir)?;
        }
    } else {
        println!("No project linked to this directory.");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Some(Commands::Login) => cmd_login().await,
        Some(Commands::Link { project_id }) => cmd_link(project_id).await,
        Some(Commands::Config) => cmd_config(),
        Some(Commands::Run { task, yes }) => cmd_run(&task, yes || cli.auto_approve).await,
        Some(Commands::Unlink) => cmd_unlink(),
        None => cmd_interactive(cli.auto_approve).await,
    }
}
