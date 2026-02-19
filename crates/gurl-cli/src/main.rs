mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gurl", version, about = "The HTTP runtime for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    args: commands::HttpArgs,
}

#[derive(Subcommand)]
enum Commands {
    /// HTTP GET request
    Get(commands::HttpArgs),
    /// HTTP POST request
    Post(commands::HttpArgs),
    /// HTTP PUT request
    Put(commands::HttpArgs),
    /// HTTP PATCH request
    Patch(commands::HttpArgs),
    /// HTTP DELETE request
    Delete(commands::HttpArgs),
    /// HTTP HEAD request
    Head(commands::HttpArgs),
    /// HTTP OPTIONS request
    Options(commands::HttpArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Get(args)) => commands::execute("GET", args).await,
        Some(Commands::Post(args)) => commands::execute("POST", args).await,
        Some(Commands::Put(args)) => commands::execute("PUT", args).await,
        Some(Commands::Patch(args)) => commands::execute("PATCH", args).await,
        Some(Commands::Delete(args)) => commands::execute("DELETE", args).await,
        Some(Commands::Head(args)) => commands::execute("HEAD", args).await,
        Some(Commands::Options(args)) => commands::execute("OPTIONS", args).await,
        None => {
            if cli.args.url.is_empty() {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            } else {
                commands::execute("GET", cli.args).await
            }
        }
    }
}
