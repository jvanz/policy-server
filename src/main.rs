mod cli;

use std::fs;
use std::io::prelude::*;

use ::tracing::info;
use anyhow::anyhow;
use anyhow::Result;
use opentelemetry::global::shutdown_tracer_provider;
use policy_server::metrics::setup_metrics;
use policy_server::tracing::setup_tracing;
use policy_server::PolicyServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Starting from rustls 0.22, each application must set its default crypto provider.
    let crypto_provider = rustls::crypto::ring::default_provider();
    crypto_provider
        .install_default()
        .expect("Failed to install crypto provider");

    let matches = cli::build_cli().get_matches();
    match matches.subcommand_name() {
        Some("docs") => {
            if let Some(matches) = matches.subcommand_matches("docs") {
                let output = matches.get_one::<String>("output").unwrap();
                let mut file = std::fs::File::create(output)
                    .map_err(|e| anyhow!("cannot create file {}: {}", output, e))?;
                let docs_content = clap_markdown::help_markdown_command(&cli::build_cli());
                file.write_all(docs_content.as_bytes())
                    .map_err(|e| anyhow!("cannot write to file {}: {}", output, e))?;
            }
            Ok(())
        }
        _ => {
            let config = policy_server::config::Config::from_args(&matches)?;

            setup_tracing(&config.log_level, &config.log_fmt, config.log_no_color)?;

            if config.metrics_enabled {
                setup_metrics()?;
            };

            if config.daemon {
                info!("Running instance as a daemon");

                let mut daemonize = daemonize::Daemonize::new().pid_file(&config.daemon_pid_file);
                if let Some(stdout_file) = config.daemon_stdout_file.clone() {
                    let file = fs::File::create(stdout_file)
                        .map_err(|e| anyhow!("Cannot create file for daemon stdout: {}", e))?;
                    daemonize = daemonize.stdout(file);
                }
                if let Some(stderr_file) = config.daemon_stderr_file.clone() {
                    let file = fs::File::create(stderr_file)
                        .map_err(|e| anyhow!("Cannot create file for daemon stderr: {}", e))?;
                    daemonize = daemonize.stderr(file);
                }

                daemonize
                    .start()
                    .map_err(|e| anyhow!("Cannot daemonize: {}", e))?;

                info!("Detached from shell, now running in background.");
            }

            let api_server = PolicyServer::new_from_config(config).await?;
            api_server.run().await?;

            shutdown_tracer_provider();

            Ok(())
        }
    }
}
