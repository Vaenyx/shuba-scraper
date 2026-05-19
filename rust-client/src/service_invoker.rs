use anyhow::{Ok, Result};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout, Command as TokioCommand};
use tokio::sync::Notify;

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
    fn transmit_browser_out(stdout: ChildStdout, ready: Arc<Notify>, silence_browser: bool) {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();

            while let std::result::Result::Ok(Some(line)) = reader.next_line().await {
                if !silence_browser {
                    println!("[browser-api] {}", line);
                }

                if line.contains("READY") {
                    ready.notify_one();
                }
            }
        });
    }

    pub async fn new(port: u16, silence_browser: bool) -> Result<Self> {
        let mut service = TokioCommand::new("node")
            .arg("service.js")
            .arg(port.to_string())
            .current_dir("../playwright-service")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = service.stderr.take().unwrap();
        let stdout = service.stdout.take().unwrap();

        let ready = Arc::new(Notify::new());

        Self::transmit_browser_err(stderr);

        Self::transmit_browser_out(stdout, ready.clone(), silence_browser);

        ready.notified().await;

        return Ok(Self { service });
    }

    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.service.kill().await?;
        return Ok(());
    }
}
