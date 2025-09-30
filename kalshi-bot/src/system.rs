use std::time::Duration;

use anyhow::{Context, Result};
use colored::{Color, Colorize};
use futures::future::join_all;
use std::process::Stdio;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    task::JoinHandle,
    time::sleep,
};

struct CommandSpec {
    cmd: String,
    args: Option<Vec<String>>,
    delay_secs: Option<u64>,
    color: Color,
    name: &'static str,
}

fn run_in_subprocess(service: CommandSpec) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        if let Some(delay) = service.delay_secs {
            sleep(Duration::from_secs(delay)).await;
        }
        let prefix = service.name.color(service.color);
        let stdout_prefix = prefix.clone();
        let stderr_prefix = prefix.clone();

        let args: &[String] = service.args.as_deref().unwrap_or(&[]);
        let mut child = Command::new(&service.cmd)
            .args(args)
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
                args[1],
                status.code()
            );
        }

        Ok(())
    })
}

pub async fn start_system() -> Result<()> {
    // Get the current executable's path
    let exe = std::env::current_exe()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let services = vec![
        CommandSpec {
            cmd: "geckodriver".into(),
            args: None,
            delay_secs: None,
            color: Color::Blue,
            name: "geckodriver",
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["data-source".into(), "nws-daily-observations".into()]),
            delay_secs: None,
            color: Color::Green,
            name: "nws-daily-observations",
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["data-source".into(), "nws-hourly-observations".into()]),
            delay_secs: Some(2),
            color: Color::Yellow,
            name: "nws-hourly-observations",
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["data-source".into(), "weather-forecast".into()]),
            delay_secs: None,
            color: Color::Magenta,
            name: "weather-forecast",
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["strategy".into(), "dump-if-temp-higher".into()]),
            delay_secs: Some(4),
            color: Color::Cyan,
            name: "dump-if-temp-higher",
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["strategy".into(), "forecast-notifier".into()]),
            delay_secs: Some(4),
            color: Color::BrightYellow,
            name: "forecast-notifier",
        },
    ];
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
