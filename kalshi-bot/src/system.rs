use crate::{datasource::name::DataSourceName, strategy::name::StrategyName};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::Args;
use colored::{Color, Colorize};
use futures::future::join_all;
use std::process::Stdio;
use std::time::Duration;
use strum::IntoEnumIterator;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    task::JoinHandle,
    time::sleep,
};

struct CommandSpec {
    cmd: String,
    args: Vec<String>,
    delay_secs: Option<u64>,
    color: Color,
    name: String,
}

fn run_in_subprocess(service: CommandSpec) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        if let Some(delay) = service.delay_secs {
            sleep(Duration::from_secs(delay)).await;
        }
        let prefix = service.name.color(service.color);
        let stdout_prefix = prefix.clone();
        let stderr_prefix = prefix.clone();

        let mut child = Command::new(&service.cmd)
            .args(&service.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start command `{}`", service.cmd))?;

        // Merge stdout and stderr
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        // Spawn tasks for both streams
        let stdout_task = tokio::spawn(async move {
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                println!("{} {}", stdout_prefix, line);
            }
        });

        let stderr_task = tokio::spawn(async move {
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                eprintln!("{} {}", stderr_prefix, line.red());
            }
        });

        let status = child
            .wait()
            .await
            .with_context(|| format!("Failed to wait for command `{}`", service.cmd))?;
        stdout_task.await?;
        stderr_task.await?;

        if !status.success() {
            anyhow::bail!(
                "Command `{}` exited with status {:?}",
                service.args[1],
                status.code()
            );
        }

        Ok(())
    })
}

#[derive(Debug, Clone, Args)]
pub struct SystemCommand {
    #[arg(short, long)]
    date: NaiveDate,
}

pub async fn start_system(command: &SystemCommand) -> Result<()> {
    // Get the current executable's path
    let exe = std::env::current_exe()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut services = vec![CommandSpec {
        cmd: "geckodriver".into(),
        args: vec!["--log".into(), "error".into()],
        delay_secs: None,
        color: Color::Blue,
        name: "geckodriver".into(),
    }];

    for data_source in DataSourceName::iter() {
        services.push(CommandSpec {
            cmd: exe.clone(),
            args: vec!["data-source".into(), data_source.to_string()],
            delay_secs: None,
            color: Color::Green,
            name: data_source.to_string(),
        })
    }

    for strategy in StrategyName::iter() {
        services.push(CommandSpec {
            cmd: exe.clone(),
            args: vec![
                "strategy".into(),
                strategy.to_string(),
                "--date".into(),
                command.date.to_string(),
            ],
            delay_secs: None,
            color: Color::Cyan,
            name: strategy.to_string(),
        })
    }
    let handles: Vec<_> = services.into_iter().map(run_in_subprocess).collect();

    let results = join_all(handles).await;
    for result in results {
        match result {
            Ok(Ok(())) => { /* success */ }
            Ok(Err(e)) => eprintln!("Service error: {:?}", e),
            Err(e) => eprintln!("Task panicked: {:?}", e),
        }
    }

    Ok(())
}
