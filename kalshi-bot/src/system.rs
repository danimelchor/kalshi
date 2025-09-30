use std::time::Duration;

use anyhow::{Context, Result};
use futures::future::join_all;
use tokio::{process::Command, task::JoinHandle, time::sleep};

struct CommandSpec {
    cmd: String,
    args: Option<Vec<String>>,
    delay_secs: Option<u64>,
}

fn run_in_subprocess(service: CommandSpec) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        if let Some(delay) = service.delay_secs {
            sleep(Duration::from_secs(delay)).await;
        }

        let args: &[String] = service.args.as_deref().unwrap_or(&[]);
        let mut child = Command::new(&service.cmd)
            .args(args)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to start command `{}`", service.cmd))?;

        let status = child
            .wait()
            .await
            .with_context(|| format!("Failed to wait for command `{}`", service.cmd))?;

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
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["data-source".into(), "nws-daily-observations".into()]),
            delay_secs: None,
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["data-source".into(), "nws-hourly-observations".into()]),
            delay_secs: Some(2),
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["data-source".into(), "weather-forecast".into()]),
            delay_secs: None,
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["strategy".into(), "dump-if-temp-higher".into()]),
            delay_secs: Some(4),
        },
        CommandSpec {
            cmd: exe.clone(),
            args: Some(vec!["strategy".into(), "forecast-notifier".into()]),
            delay_secs: Some(4),
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
