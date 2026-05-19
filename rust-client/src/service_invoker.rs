use anyhow::{Ok, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout, Command as TokioCommand};

#[derive(Debug)]
pub struct ServiceInvoker {
    service: tokio::process::Child,
}

impl ServiceInvoker {
    fn transmit_browser_err(stderr: ChildStderr) {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();

            while let std::result::Result::Ok(Some(line)) = reader.next_line().await {
                eprintln!("[browser-api stderr] {}", line);
            }
        });
    }

    async fn wait_for_browser(stdout: ChildStdout) -> Result<()> {
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await? {
            println!("[browser-api] {}", line);

            if line.contains("READY") {
                break;
            }
        }
        return Ok(());
    }

    pub async fn new(port: usize) -> Result<Self> {
        let mut service = TokioCommand::new("node")
            .arg("service.js")
            .arg(port.to_string())
            .current_dir("../playwright-service")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = service.stderr.take().unwrap();
        let stdout = service.stdout.take().unwrap();

        Self::transmit_browser_err(stderr);
        Self::wait_for_browser(stdout).await?;

        return Ok(Self { service });
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.service.kill().await?;
        return Ok(());
    }
}
