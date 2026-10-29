//! Explicit, per-user AppImage maintenance from Core's About window.
use eframe::egui;
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(5 * 60);
const MAX_OUTPUT: usize = 32 * 1024;

#[derive(Clone)]
struct Package {
    image: PathBuf,
    directory: PathBuf,
    launcher: PathBuf,
}

impl Package {
    fn from_paths(image: PathBuf, directory: PathBuf) -> Option<Self> {
        if !image.is_absolute() || !image.is_file() || !directory.is_absolute() {
            return None;
        }
        let image = image.canonicalize().ok()?;
        let directory = directory.canonicalize().ok()?;
        let launcher = directory.join("AppRun");
        if !directory.is_dir() || !launcher.is_file() {
            return None;
        }
        Some(Self {
            image,
            directory,
            launcher,
        })
    }

    fn owns_executable(&self, executable: &Path) -> bool {
        executable
            .canonicalize()
            .ok()
            .zip(
                self.directory
                    .join("usr/bin/observatory-core")
                    .canonicalize()
                    .ok(),
            )
            .is_some_and(|(executable, payload)| executable == payload)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Install,
    Check,
    Update,
}

impl Action {
    fn argument(self) -> &'static str {
        match self {
            Self::Install => "--install",
            Self::Check => "--check-for-update",
            Self::Update => "--update",
        }
    }

    fn progress(self) -> &'static str {
        match self {
            Self::Install => "Installing for this user…",
            Self::Check => "Checking for AppImage updates…",
            Self::Update => "Downloading and updating the AppImage…",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpdateState {
    Unchecked,
    Current,
    Available,
    RestartRequired,
}

struct Outcome {
    available: bool,
    message: String,
}

/// Transient state only: opening About never starts a process or network request.
pub struct Manager {
    package: Package,
    pending: Option<(Action, Receiver<Result<Outcome, String>>)>,
    update_state: UpdateState,
    installed_needs_relaunch: bool,
    message: String,
    failed: bool,
}

impl Manager {
    pub fn detect() -> Option<Self> {
        if !cfg!(target_os = "linux") {
            return None;
        }
        let package = Package::from_paths(
            std::env::var_os("APPIMAGE")?.into(),
            std::env::var_os("APPDIR")?.into(),
        )?;
        if !package.owns_executable(&std::env::current_exe().ok()?) {
            return None;
        }
        Some(Self::new(package))
    }

    fn new(package: Package) -> Self {
        Self {
            package,
            pending: None,
            update_state: UpdateState::Unchecked,
            installed_needs_relaunch: false,
            message:
                "Install adds a launcher to your application menu. Updates have not been checked."
                    .into(),
            failed: false,
        }
    }

    fn start(&mut self, action: Action) {
        if self.pending.is_some()
            || self.installed_needs_relaunch && matches!(action, Action::Check | Action::Update)
            || action == Action::Update && self.update_state != UpdateState::Available
        {
            return;
        }
        let package = self.package.clone();
        let (sender, receiver) = mpsc::channel();
        match thread::Builder::new()
            .name("appimage-maintenance".into())
            .spawn(move || {
                let _ = sender.send(run(&package, action, TIMEOUT));
            }) {
            Ok(_) => {
                self.pending = Some((action, receiver));
                if action == Action::Check {
                    self.update_state = UpdateState::Unchecked;
                }
                self.message = action.progress().into();
                self.failed = false;
            }
            Err(error) => {
                self.message = format!("Could not start AppImage maintenance: {error}");
                self.failed = true;
            }
        }
    }

    fn poll(&mut self) {
        let Some((action, receiver)) = &self.pending else {
            return;
        };
        let action = *action;
        let result = match receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                Err("AppImage maintenance stopped without reporting a result.".into())
            }
        };
        self.pending = None;
        match result {
            Ok(outcome) => {
                match action {
                    Action::Install => self.installed_needs_relaunch = true,
                    Action::Check => {
                        self.update_state = if outcome.available {
                            UpdateState::Available
                        } else {
                            UpdateState::Current
                        };
                    }
                    Action::Update => self.update_state = UpdateState::RestartRequired,
                }
                self.message = outcome.message;
                self.failed = false;
            }
            Err(message) => {
                self.message = message;
                self.failed = true;
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll();
        ui.separator();
        ui.strong("AppImage")
            .on_hover_text(self.package.image.display().to_string());
        let busy = self.pending.is_some();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!busy, egui::Button::new("Install for this user"))
                .clicked()
            {
                self.start(Action::Install);
            }
            if ui
                .add_enabled(
                    !busy
                        && !self.installed_needs_relaunch
                        && self.update_state != UpdateState::RestartRequired,
                    egui::Button::new("Check for Updates"),
                )
                .clicked()
            {
                self.start(Action::Check);
            }
            if !self.installed_needs_relaunch
                && self.update_state == UpdateState::Available
                && ui.add_enabled(!busy, egui::Button::new("Update")).clicked()
            {
                self.start(Action::Update);
            }
            if self.pending.is_some() {
                ui.spinner();
            }
        });
        if self.installed_needs_relaunch {
            ui.label("Installed. Close this copy and launch Observatory from your application menu to manage its updates.");
        } else if self.update_state == UpdateState::RestartRequired {
            ui.label("Update complete. Close Observatory and launch the AppImage again to use it.");
        }
        egui::ScrollArea::vertical()
            .id_salt("AppImage maintenance status")
            .max_height(70.0)
            .show(ui, |ui| {
                let text = egui::RichText::new(&self.message);
                ui.add(
                    egui::Label::new(if self.failed {
                        text.color(ui.visuals().error_fg_color)
                    } else {
                        text
                    })
                    .wrap()
                    .selectable(true),
                );
            });
        if self.pending.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }
}

struct Captured {
    bytes: Vec<u8>,
    truncated: bool,
    error: Option<String>,
}

fn capture(mut input: impl Read) -> Captured {
    let mut captured = Captured {
        bytes: Vec::new(),
        truncated: false,
        error: None,
    };
    let mut buffer = [0; 4096];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => return captured,
            Ok(count) => {
                let keep = count.min(MAX_OUTPUT - captured.bytes.len());
                captured.bytes.extend_from_slice(&buffer[..keep]);
                captured.truncated |= keep < count;
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => {
                captured.error = Some(error.to_string());
                return captured;
            }
        }
    }
}

impl Captured {
    fn text(&self) -> String {
        let mut text = String::from_utf8_lossy(&self.bytes).trim().to_owned();
        if self.truncated {
            text.push_str("\n[output truncated]");
        }
        text
    }
}

fn stop(child: &mut std::process::Child) {
    // AppRun may have started an updater: terminate the complete worker group.
    #[cfg(target_os = "linux")]
    unsafe {
        libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn run(package: &Package, action: Action, timeout: Duration) -> Result<Outcome, String> {
    let mut command = Command::new(&package.launcher);
    command
        .arg(action.argument())
        .env("APPIMAGE", &package.image)
        .env("APPDIR", &package.directory)
        .current_dir(&package.directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not run AppImage {}: {error}", action.argument()))?;
    let (sender, receiver) = mpsc::channel();
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let stderr_sender = sender.clone();
    thread::spawn(move || {
        let _ = sender.send((false, capture(stdout)));
    });
    thread::spawn(move || {
        let _ = stderr_sender.send((true, capture(stderr)));
    });
    let deadline = Instant::now() + timeout;
    let mut status = None;
    let mut stdout = None;
    let mut stderr = None;
    loop {
        while let Ok((is_stderr, output)) = receiver.try_recv() {
            if is_stderr {
                stderr = Some(output);
            } else {
                stdout = Some(output);
            }
        }
        if status.is_none() {
            status = match child.try_wait() {
                Ok(status) => status,
                Err(error) => {
                    stop(&mut child);
                    return Err(format!("Could not wait for AppImage maintenance: {error}"));
                }
            };
        }
        if let (Some(status), Some(stdout), Some(stderr)) = (status, &stdout, &stderr) {
            return interpret(action, status, stdout, stderr);
        }
        if Instant::now() >= deadline {
            stop(&mut child);
            return Err(format!(
                "AppImage {} timed out after {} seconds. The maintenance process was stopped.",
                action.argument(),
                timeout.as_secs_f32()
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn interpret(
    action: Action,
    status: ExitStatus,
    stdout: &Captured,
    stderr: &Captured,
) -> Result<Outcome, String> {
    let output = [stdout.text(), stderr.text()]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if let Some(error) = stdout.error.as_ref().or(stderr.error.as_ref()) {
        return Err(format!(
            "Could not read AppImage maintenance output: {error}\n{output}"
        ));
    }
    let available = action == Action::Check && status.code() == Some(1);
    if !status.success() && !available {
        return Err(format!(
            "AppImage {} failed ({status}).{}",
            action.argument(),
            if output.is_empty() {
                String::new()
            } else {
                format!("\n{output}")
            }
        ));
    }
    let message = match action {
        Action::Install => "Installed for this user.",
        Action::Check if available => {
            "An AppImage update is available. Select Update to install it."
        }
        Action::Check => "The AppImage is up to date.",
        Action::Update => "The AppImage was updated successfully.",
    };
    Ok(Outcome {
        available,
        message: if output.is_empty() {
            message.into()
        } else {
            format!("{message}\n{output}")
        },
    })
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    fn package(script: &str) -> (tempfile::TempDir, Package) {
        let root = tempfile::tempdir().unwrap();
        let image = root.path().join("Observatory $(literal) image.AppImage");
        let directory = root.path().join("App Dir");
        fs::write(&image, "AppImage fixture").unwrap();
        fs::create_dir(&directory).unwrap();
        let launcher = directory.join("AppRun");
        fs::write(&launcher, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
        (root, Package::from_paths(image, directory).unwrap())
    }

    #[test]
    fn detection_requires_absolute_image_file_and_apprun() {
        let (_root, package) = package("exit 0");
        assert!(
            Package::from_paths("relative.AppImage".into(), package.directory.clone()).is_none()
        );
        assert!(
            Package::from_paths(package.directory.clone(), package.directory.clone()).is_none()
        );
        assert!(Package::from_paths(package.image.clone(), "relative".into()).is_none());
        fs::remove_file(&package.launcher).unwrap();
        assert!(Package::from_paths(package.image, package.directory).is_none());
    }

    #[test]
    fn inherited_appimage_environment_cannot_target_another_app() {
        let (root, package) = package("exit 0");
        let payload = package.directory.join("usr/bin/observatory-core");
        fs::create_dir_all(payload.parent().unwrap()).unwrap();
        fs::write(&payload, "payload Core").unwrap();
        let foreign = root.path().join("standalone-observatory-core");
        fs::write(&foreign, "another Core").unwrap();
        assert!(package.owns_executable(&payload));
        assert!(!package.owns_executable(&foreign));
        fs::remove_file(payload).unwrap();
        assert!(!package.owns_executable(&foreign));
    }

    #[test]
    fn maintenance_passes_literal_paths_and_only_requested_argument() {
        let (_root, package) =
            package("printf '%s\\n%s\\n%s\\n%s\\n' \"$APPIMAGE\" \"$APPDIR\" \"$1\" \"$#\"");
        let result = run(&package, Action::Install, Duration::from_secs(2)).unwrap();
        assert!(result
            .message
            .contains(&package.image.display().to_string()));
        assert!(result
            .message
            .contains(&package.directory.display().to_string()));
        assert!(result.message.ends_with("--install\n1"));
    }

    #[test]
    fn check_distinguishes_current_available_and_error() {
        for (code, available) in [(0, false), (1, true)] {
            let (_root, package) = package(&format!("printf 'check details'; exit {code}"));
            let result = run(&package, Action::Check, Duration::from_secs(2)).unwrap();
            assert_eq!(result.available, available);
            assert!(result.message.contains("check details"));
        }
        let (_root, package) = package("printf 'download failure' >&2; exit 2");
        let error = run(&package, Action::Check, Duration::from_secs(2))
            .err()
            .unwrap();
        assert!(error.contains("exit status: 2"));
        assert!(error.contains("download failure"));
    }

    #[test]
    fn install_and_update_failures_are_never_treated_as_available() {
        let (_root, package) = package("printf 'failed' >&2; exit 1");
        for action in [Action::Install, Action::Update] {
            let error = run(&package, action, Duration::from_secs(2)).err().unwrap();
            assert!(error.contains("failed"));
            assert!(error.contains("exit status: 1"));
        }
    }

    #[test]
    fn missing_launcher_reports_spawn_error() {
        let (_root, package) = package("exit 0");
        fs::remove_file(&package.launcher).unwrap();
        let error = run(&package, Action::Install, Duration::from_secs(2))
            .err()
            .unwrap();
        assert!(error.contains("Could not run AppImage --install"));
    }

    #[test]
    fn timeout_includes_children_with_inherited_output() {
        let (_root, package) = package("sleep 10 &\nexit 0");
        let started = Instant::now();
        let error = run(&package, Action::Update, Duration::from_millis(80))
            .err()
            .unwrap();
        assert!(error.contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn output_is_capped_while_pipe_is_fully_drained() {
        let (_root, package) = package("head -c 262144 /dev/zero | tr '\\000' x");
        let result = run(&package, Action::Install, Duration::from_secs(2)).unwrap();
        assert!(result.message.contains("[output truncated]"));
        assert!(result.message.len() < MAX_OUTPUT + 200);
    }

    fn finish(manager: &mut Manager) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while manager.pending.is_some() && Instant::now() < deadline {
            manager.poll();
            thread::sleep(Duration::from_millis(5));
        }
        assert!(manager.pending.is_none());
    }

    #[test]
    fn update_requires_check_then_explicit_action_and_preserves_restart_notice() {
        let (_root, package) =
            package("case \"$1\" in --check-for-update) exit 1;; *) exit 0;; esac");
        let mut manager = Manager::new(package);
        manager.start(Action::Update);
        assert!(manager.pending.is_none());
        manager.start(Action::Check);
        finish(&mut manager);
        assert_eq!(manager.update_state, UpdateState::Available);
        assert!(manager.pending.is_none());
        manager.start(Action::Update);
        finish(&mut manager);
        assert_eq!(manager.update_state, UpdateState::RestartRequired);
        manager.start(Action::Install);
        finish(&mut manager);
        assert_eq!(manager.update_state, UpdateState::RestartRequired);
    }

    #[test]
    fn installation_requires_relaunch_before_checking_or_updating_the_managed_copy() {
        let (_root, package) = package(
            "case \"$1\" in --check-for-update) touch checked-portable-copy; exit 1;; --update) touch updated-portable-copy;; esac",
        );
        let directory = package.directory.clone();
        let mut manager = Manager::new(package);
        manager.update_state = UpdateState::Available;
        manager.start(Action::Install);
        finish(&mut manager);
        assert!(manager.installed_needs_relaunch);
        for action in [Action::Check, Action::Update] {
            manager.start(action);
            assert!(manager.pending.is_none());
        }
        assert!(!directory.join("checked-portable-copy").exists());
        assert!(!directory.join("updated-portable-copy").exists());
    }

    #[test]
    fn idle_about_does_not_start_maintenance() {
        let (root, package) = package("touch maintenance-was-run");
        let mut manager = Manager::new(package);
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| manager.show(ui));
        });
        assert!(manager.pending.is_none());
        assert!(!root.path().join("App Dir/maintenance-was-run").exists());
    }
}
