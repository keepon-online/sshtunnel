use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use sshtunnel_core::ssh_launch::{CommandSpec, LaunchPlan};

pub struct ManagedProcess {
    inner: ManagedProcessInner,
    logs: Arc<Mutex<Vec<String>>>,
}

enum ManagedProcessInner {
    Native(Child),
    Prompted(PromptedProcess),
}

struct PromptedProcess {
    child: Box<dyn portable_pty::Child + Send>,
    reader_thread: Option<JoinHandle<()>>,
}

impl ManagedProcess {
    pub fn spawn(plan: LaunchPlan) -> Result<Self, String> {
        match plan {
            LaunchPlan::Native(command) => Self::spawn_native(command),
            LaunchPlan::PromptedPassword {
                command,
                password,
                prompt,
            } => Self::spawn_prompted(command, &password, &prompt),
        }
    }

    pub fn pid(&self) -> Option<u32> {
        match &self.inner {
            ManagedProcessInner::Native(child) => Some(child.id()),
            ManagedProcessInner::Prompted(child) => child.child.process_id(),
        }
    }

    pub fn try_wait(&mut self) -> Result<Option<String>, String> {
        match &mut self.inner {
            ManagedProcessInner::Native(child) => child
                .try_wait()
                .map(|status| status.map(|item| format!("{item}")))
                .map_err(|error| error.to_string()),
            ManagedProcessInner::Prompted(child) => child
                .child
                .try_wait()
                .map(|status| status.map(|item| format!("{item:?}")))
                .map_err(|error| error.to_string()),
        }
    }

    pub fn kill(&mut self) -> Result<(), String> {
        match &mut self.inner {
            ManagedProcessInner::Native(child) => {
                child.kill().map_err(|error| error.to_string())?;
                let _ = child.wait();
            }
            ManagedProcessInner::Prompted(child) => {
                child.child.kill().map_err(|error| error.to_string())?;
                let _ = child.child.wait();
                if let Some(handle) = child.reader_thread.take() {
                    let _ = handle.join();
                }
            }
        }

        Ok(())
    }

    pub fn take_logs(&self) -> Vec<String> {
        let mut logs = self.logs.lock().expect("managed-process logs poisoned");
        std::mem::take(&mut *logs)
    }

    fn spawn_native(command: CommandSpec) -> Result<Self, String> {
        let mut child = Command::new(&command.program);
        child.args(command.args);
        child.stdin(Stdio::null());
        child.stdout(Stdio::null());
        child.stderr(Stdio::null());

        let child = child.spawn().map_err(|error| error.to_string())?;

        Ok(Self {
            inner: ManagedProcessInner::Native(child),
            logs: Arc::new(Mutex::new(Vec::new())),
        })
    }

    fn spawn_prompted(command: CommandSpec, password: &str, prompt: &str) -> Result<Self, String> {
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| error.to_string())?;

        let mut builder = CommandBuilder::new(command.program);
        for arg in command.args {
            builder.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(builder)
            .map_err(|error| error.to_string())?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| error.to_string())?;
        let mut writer = pair.master.take_writer().map_err(|error| error.to_string())?;

        let logs = Arc::new(Mutex::new(Vec::new()));
        let password = password.to_string();
        let prompt = prompt.to_ascii_lowercase();
        let log_sink = Arc::clone(&logs);

        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 1024];
            let mut transcript = String::new();
            let mut sent_password = false;

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        if !transcript.trim().is_empty() {
                            push_log(&log_sink, transcript.trim());
                        }
                        break;
                    }
                    Ok(read) => {
                        let chunk = String::from_utf8_lossy(&buf[..read]).to_string();
                        transcript.push_str(&chunk);
                        for line in chunk.lines() {
                            if !line.trim().is_empty() {
                                push_log(&log_sink, line.trim());
                            }
                        }

                        if !sent_password
                            && transcript.to_ascii_lowercase().contains(&prompt)
                            && writer.write_all(password.as_bytes()).is_ok()
                            && writer.write_all(b"\n").is_ok()
                        {
                            let _ = writer.flush();
                            sent_password = true;
                            push_log(&log_sink, "password sent to interactive ssh session");
                        }
                    }
                    Err(error) => {
                        push_log(&log_sink, &format!("pty reader error: {error}"));
                        break;
                    }
                }
            }
        });

        Ok(Self {
            inner: ManagedProcessInner::Prompted(PromptedProcess {
                child,
                reader_thread: Some(reader_thread),
            }),
            logs,
        })
    }
}

fn push_log(logs: &Arc<Mutex<Vec<String>>>, line: &str) {
    let mut guard = logs.lock().expect("managed-process logs poisoned");
    guard.push(line.to_string());
    if guard.len() > 24 {
        let trim = guard.len() - 24;
        guard.drain(0..trim);
    }
}
