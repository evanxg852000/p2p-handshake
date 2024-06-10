use std::time::Duration;

use anyhow::Result;
use clap::Parser;

use p2p_handshake::{handshake, Version};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct App {
    /// Url of the target node
    #[arg(short, long)]
    target: String,

    /// Name of the client node
    #[arg(short, long)]
    name: String,

    /// Version of the client node
    #[arg(short, long)]
    version: Option<Version>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::parse();
    let version = match app.version {
        Some(version) => version,
        None => Version([3, 3, 6]), // default version
    };

    // We could pool the future right away, but we want to wrap
    // in a timeout future.
    let task = handshake(&app.target, &app.name, version, |_stream, reply| {
        println!("Handshake Reply: {:?}", reply);

        // On can keep using the stream for further work ...

        Ok(())
    });

    // Because target node could be anything, we take care of timing-out after
    // a certain period.
    // The Ergo reference node implementation will timeout after 30s. We expect any good behaving
    // to follow this guideline. Anything taking longer than that period should
    // be avoided.
    tokio::time::timeout(Duration::from_secs(30), task).await??;

    Ok(())
}
