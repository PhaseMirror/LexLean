//! A host that embeds the library ends the children the library started
//! (SPEC.md §17.17, §24.6): one public function ends every live child
//! process group, reports each as ended, already gone, or failing, installs
//! no handler, never signals the host, and refuses a later child start until
//! the host re-enables it. The case is its own test binary because it
//! replaces `PATH`, takes real signals, and ends the process-wide live list,
//! so its cases share one lock and restore the shutdown state.

mod support;

/// The tests share the process-wide child state (the live list and the
/// shutdown state), so they run one at a time.
fn serial() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Re-enable starting children whatever way the case ends: the shutdown
/// state is process-wide and the next case starts children.
struct Reenable;

impl Drop for Reenable {
    fn drop(&mut self) {
        lexlean::verify::allow_children();
    }
}

/// Restore the `PATH` a case replaced.
#[cfg(unix)]
struct RestorePath(std::ffi::OsString);

#[cfg(unix)]
impl Drop for RestorePath {
    fn drop(&mut self) {
        std::env::set_var("PATH", &self.0);
    }
}

/// A `sleep` that leads a process group of its own, as every child of the
/// library does, registered the way `verify::child::run` registers one.
#[cfg(unix)]
fn spawn_sleep(seconds: u32) -> (std::process::Child, lexlean::verify::lifeline::Guard) {
    use std::os::unix::process::CommandExt;
    let mut command = std::process::Command::new("sleep");
    command
        .arg(format!("{seconds}.{}", std::process::id()))
        .process_group(0)
        .stdin(std::process::Stdio::null());
    lexlean::verify::lifeline::spawn(&mut command).expect("a child starts")
}

/// This process's own process-group number: `/proc` where the host has one
/// and `ps` where it does not (macOS), so the case is not vacuous there.
#[cfg(unix)]
fn own_pgid() -> i32 {
    if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
        let mut fields = stat.rsplit_once(')').expect("stat").1.split_whitespace();
        let _state = fields.next();
        let _parent = fields.next();
        return fields.next().expect("pgrp").parse().expect("a number");
    }
    let ps = ["/bin/ps", "/usr/bin/ps"]
        .into_iter()
        .find(|path| std::path::Path::new(path).is_file())
        .expect("the host has neither /proc nor ps, so the process table cannot be read");
    let output = std::process::Command::new(ps)
        .args(["-o", "pgid=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps runs");
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("a group number")
}

/// An embedding host runs the compiler on a thread, ends the live children
/// while a grandchild runs, and every group is reported ended, nothing of
/// the group is left, and the run returns a diagnostic rather than hanging.
#[test]
fn an_embedding_host_ends_every_live_group() {
    let _serial = serial();
    #[cfg(unix)]
    scenario_embedding();
}

#[cfg(unix)]
fn scenario_embedding() {
    use std::time::{Duration, Instant};

    let Some(root) = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(std::path::Path::to_path_buf)
        .filter(|root| root.join("examples/nat-add-zero/lexlean.toml").is_file())
    else {
        eprintln!("the crate is being tested outside its repository; the host did not run");
        return;
    };
    let _reenable = Reenable;
    let directory = tempfile::tempdir().expect("tempdir");
    let project = directory.path().join("project");
    copy_tree(&root.join("examples/nat-add-zero"), &project);
    // A git source that must be acquired: `lock --allow-network` runs `git`.
    let manifest = project.join("lexlean.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "\n[limits]",
            "\n[[lexicon_source]]\npackage = \"test.remote\"\nkind = \"git\"\nurl = \"https://example.invalid/repo.git\"\nrevision = \"0123456789abcdef0123456789abcdef01234567\"\nsubdirectory = \"lexicon\"\n\n[limits]",
        ),
    )
    .expect("write");
    let bin = directory.path().join("bin");
    std::fs::create_dir(&bin).expect("mkdir");
    // The `git` starts a long-running grandchild, as `lake` starts `lean`,
    // and never finishes. The decimal part is this process, so no other run
    // is mistaken for it.
    let marker = format!("sleep 10000.{}", std::process::id());
    support::script(
        &bin.join("git"),
        &format!("#!/bin/sh\nsleep 10000.{} &\nwait\n", std::process::id()),
    );
    let _path = RestorePath(std::env::var_os("PATH").unwrap_or_default());
    std::env::set_var(
        "PATH",
        format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        ),
    );

    let arguments: Vec<String> = ["lexlean", "lock", "--allow-network"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let working = camino::Utf8PathBuf::from_path_buf(project).expect("utf-8");
    let (code_tx, code_rx) = std::sync::mpsc::channel();
    let run = std::thread::spawn(move || {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = lexlean::cli::run(&arguments, &working, &mut out, &mut err);
        let _ = code_tx.send(code);
    });
    let give_up = Instant::now() + Duration::from_secs(60);
    while support::running(&marker).is_empty() {
        assert!(
            Instant::now() < give_up,
            "the run never started the grandchild"
        );
        std::thread::sleep(Duration::from_millis(20));
    }

    let (pid_before, pgid_before) = (std::process::id(), own_pgid());
    let outcome = lexlean::verify::end_live_children();
    assert!(!outcome.is_empty(), "a group was live");
    assert!(
        outcome
            .iter()
            .all(|row| row.outcome == lexlean::verify::GroupEnd::Ended),
        "every live group is reported ended: {outcome:?}"
    );
    assert!(
        outcome
            .iter()
            .all(|row| row.group != u32::try_from(pgid_before).unwrap_or(0)),
        "the host's own group is never among the ended: {outcome:?}"
    );
    let give_up = Instant::now() + Duration::from_secs(10);
    let mut left = support::running(&marker);
    while !left.is_empty() && Instant::now() < give_up {
        std::thread::sleep(Duration::from_millis(20));
        left = support::running(&marker);
    }
    support::end_all(&left);
    assert!(left.is_empty(), "the end left running: {left:?}");
    let code = code_rx
        .recv_timeout(Duration::from_secs(60))
        .expect("the run returns rather than hanging once its children are gone");
    run.join().expect("the run thread");
    assert_ne!(code, 0, "the run returns a diagnostic, not a success");
    assert_eq!(pid_before, std::process::id(), "the host is untouched");
    assert_eq!(pgid_before, own_pgid(), "the host's group is untouched");
}

#[cfg(unix)]
fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy");
        }
    }
}

/// A real SIGINT and a real SIGTERM reach a host that registered its own
/// handling for them: the host ends the children from that handling's
/// thread, survives to run its own cleanup, and the library installed and
/// changed nothing.
#[test]
fn the_host_ends_children_from_its_own_signal_thread() {
    let _serial = serial();
    #[cfg(unix)]
    scenario_signals();
}

#[cfg(unix)]
fn scenario_signals() {
    use std::time::{Duration, Instant};

    let _reenable = Reenable;
    let marker = format!("sleep 10000.{}", std::process::id());
    // A service manager starts a host as the leader of its own process
    // group; lead it here where the host can, so the assertions below cover
    // that case (a host that already leads a session cannot).
    let leader = rustix::process::setsid().is_ok();
    // The host's own handling of SIGINT and SIGTERM: an ordinary thread fed
    // by the host's own registration, as a service handles shutdown.
    let mut signals = signal_hook::iterator::Signals::new([
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
    ])
    .expect("the host registers its handler");
    let (outcomes_tx, outcomes_rx) = std::sync::mpsc::channel();
    let handler = std::thread::spawn(move || {
        let mut outcomes = Vec::new();
        for signal in signals.forever().take(2) {
            outcomes.push((signal, lexlean::verify::end_live_children()));
        }
        let _ = outcomes_tx.send(outcomes);
    });

    let (pid_before, pgid_before) = (std::process::id(), own_pgid());
    if leader {
        assert_eq!(
            pgid_before,
            i32::try_from(pid_before).expect("a process id"),
            "the host leads its own process group, so it can be signalled by mistake"
        );
    }
    let (mut first_child, first_guard) = spawn_sleep(10000);
    let first_pid = first_child.id();
    rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::INT)
        .expect("the host signals itself");
    // The host survives its own SIGINT: had this crate installed a handler
    // or emulated the default action, the process would end here.
    let give_up = Instant::now() + Duration::from_secs(10);
    let mut left = support::running(&marker);
    while !left.is_empty() && Instant::now() < give_up {
        std::thread::sleep(Duration::from_millis(20));
        left = support::running(&marker);
    }
    support::end_all(&left);
    assert!(left.is_empty(), "SIGINT left running: {left:?}");
    let _ = first_child.wait();
    drop(first_guard);

    // The host's own cleanup: re-enable, start another child, and end it
    // from the same handling on SIGTERM.
    lexlean::verify::allow_children();
    let (mut second_child, second_guard) = spawn_sleep(10000);
    let second_pid = second_child.id();
    rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::TERM)
        .expect("the host signals itself");
    let give_up = Instant::now() + Duration::from_secs(10);
    let mut left = support::running(&marker);
    while !left.is_empty() && Instant::now() < give_up {
        std::thread::sleep(Duration::from_millis(20));
        left = support::running(&marker);
    }
    support::end_all(&left);
    assert!(left.is_empty(), "SIGTERM left running: {left:?}");
    let _ = second_child.wait();
    drop(second_guard);

    let outcomes = outcomes_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("both signals reached the host's own handling");
    handler.join().expect("the handling thread");
    assert_eq!(outcomes.len(), 2, "SIGINT and SIGTERM, one end each");
    let mut reported = Vec::new();
    for (signal, outcome) in &outcomes {
        assert!(
            outcome
                .iter()
                .all(|row| row.outcome == lexlean::verify::GroupEnd::Ended),
            "signal {signal}: every live group ended: {outcome:?}"
        );
        reported.extend(outcome.iter().map(|row| row.group));
    }
    assert!(
        reported.contains(&first_pid) && reported.contains(&second_pid),
        "both children were ended and reported: {reported:?}"
    );
    assert_eq!(pid_before, std::process::id(), "the host is untouched");
    assert_eq!(pgid_before, own_pgid(), "the host's group is untouched");
}

/// A call with no live child is a no-op, and a second call is idempotent.
#[test]
fn nothing_live_is_a_no_op_and_a_second_call_is_idempotent() {
    let _serial = serial();
    let _reenable = Reenable;
    let first = lexlean::verify::end_live_children();
    assert!(first.is_empty(), "nothing live is nothing ended: {first:?}");
    let second = lexlean::verify::end_live_children();
    assert!(
        second.is_empty(),
        "the second call ends nothing either: {second:?}"
    );
}

/// A child that already exited is reported as already gone, not as a
/// failure: it is waited and reaped, so its group has nothing left to
/// signal.
#[test]
fn an_exited_child_is_reported_gone_not_failed() {
    let _serial = serial();
    #[cfg(unix)]
    scenario_gone();
}

#[cfg(unix)]
fn scenario_gone() {
    use std::os::unix::process::CommandExt;

    let _reenable = Reenable;
    let mut command = std::process::Command::new("true");
    command.process_group(0).stdin(std::process::Stdio::null());
    let (mut child, guard) = lexlean::verify::lifeline::spawn(&mut command).expect("true starts");
    let pid = child.id();
    let status = child.wait().expect("the child is waited and reaped");
    assert!(status.success());
    let outcome = lexlean::verify::end_live_children();
    assert_eq!(
        outcome,
        vec![lexlean::verify::ChildGroupEnd {
            group: pid,
            outcome: lexlean::verify::GroupEnd::AlreadyGone,
        }],
        "an exited child is gone, not a failure"
    );
    drop(guard);
}

/// After the end, starting a child is refused with `LLV7017` until the host
/// re-enables it; then a child starts again.
#[test]
fn a_child_started_after_the_end_is_refused() {
    let _serial = serial();
    #[cfg(unix)]
    scenario_refused();
}

#[cfg(unix)]
fn scenario_refused() {
    let _reenable = Reenable;
    let _ = lexlean::verify::end_live_children();
    match spawn_sleep_or_refusal() {
        Err(lexlean::verify::lifeline::CannotStart::Shutdown) => {}
        Err(other) => panic!("the refusal is the shutdown state: {other:?}"),
        Ok(_) => panic!("a child started after the end must be refused"),
    }
    // Through the child pipeline the refusal is the registered diagnostic.
    let directory = tempfile::tempdir().expect("tempdir");
    let script = directory.path().join("lake");
    support::script(&script, "#!/bin/sh\nexit 0\n");
    let program = camino::Utf8Path::from_path(&script).expect("utf-8");
    let cwd = camino::Utf8Path::from_path(directory.path()).expect("utf-8");
    let spec = lexlean::verify::child::ChildSpec {
        tool: "lake",
        module: None,
        program,
        executable_sha256: lexlean::Sha256Digest::of(b"script"),
        argv: Vec::new(),
        cwd,
        extra_env: Vec::new(),
        home: lexlean::verify::child::ChildHome::Isolated { home: cwd },
    };
    let error = lexlean::verify::child::run(
        &spec,
        &limits(1000),
        &lexlean::verify::child::Normalizer::default(),
    )
    .expect_err("a child started after the end is refused");
    assert_eq!(error.code.as_str(), "LLV7017", "{error:?}");
    // The host re-enables explicitly, and a child starts again.
    lexlean::verify::allow_children();
    let mut command = std::process::Command::new("true");
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0).stdin(std::process::Stdio::null());
    }
    let (mut child, guard) =
        lexlean::verify::lifeline::spawn(&mut command).expect("a child starts once re-enabled");
    let _ = child.wait();
    drop(guard);
}

/// A `sleep` or the refusal to start it.
#[cfg(unix)]
fn spawn_sleep_or_refusal() -> Result<
    (std::process::Child, lexlean::verify::lifeline::Guard),
    lexlean::verify::lifeline::CannotStart,
> {
    use std::os::unix::process::CommandExt;
    let mut command = std::process::Command::new("sleep");
    command
        .arg(format!("10000.{}", std::process::id()))
        .process_group(0)
        .stdin(std::process::Stdio::null());
    lexlean::verify::lifeline::spawn(&mut command)
}

/// Children started from several threads while another thread calls the
/// function: each is ended or refused, none survives. The first registered
/// group is in the call's outcome, every started child is accounted ended,
/// and a later call finds nothing, because the shutdown state is set under
/// the same lock that registers a child.
#[test]
fn concurrent_starts_are_ended_or_refused() {
    let _serial = serial();
    #[cfg(unix)]
    scenario_concurrent();
}

#[cfg(unix)]
fn scenario_concurrent() {
    use std::collections::BTreeSet;
    use std::sync::{Arc, Barrier};
    use std::time::Duration;

    let _reenable = Reenable;
    let marker = format!("sleep 300.{}", std::process::id());
    let (registered_tx, registered_rx) = std::sync::mpsc::channel::<u32>();
    let barrier = Arc::new(Barrier::new(5));
    let mut joins = Vec::new();
    for _ in 0..4 {
        let registered_tx = registered_tx.clone();
        let barrier = Arc::clone(&barrier);
        joins.push(std::thread::spawn(move || {
            let mut mine = Vec::new();
            barrier.wait();
            for _ in 0..25 {
                match spawn_sleep_or_refusal_300() {
                    Ok((child, guard)) => {
                        let _ = registered_tx.send(child.id());
                        mine.push((child, guard));
                    }
                    Err(lexlean::verify::lifeline::CannotStart::Shutdown) => {}
                    Err(other) => panic!("the spawn works or is refused: {other:?}"),
                }
            }
            mine
        }));
    }
    barrier.wait();
    let first = registered_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("at least one child registers");
    let outcome = lexlean::verify::end_live_children();
    let mut spawned: Vec<(std::process::Child, lexlean::verify::lifeline::Guard)> = Vec::new();
    for join in joins {
        spawned.extend(join.join().expect("the spawn thread"));
    }
    let sweep = lexlean::verify::end_live_children();
    assert!(
        sweep.is_empty(),
        "the shutdown state refuses every later start, so nothing is left to end: {sweep:?}"
    );
    assert!(
        outcome
            .iter()
            .any(|row| row.group == first && row.outcome == lexlean::verify::GroupEnd::Ended),
        "the first registered group is ended by the call: {outcome:?}"
    );
    assert!(
        outcome
            .iter()
            .all(|row| row.outcome == lexlean::verify::GroupEnd::Ended),
        "every registered group was live and is ended: {outcome:?}"
    );
    let ended: BTreeSet<u32> = outcome.iter().map(|row| row.group).collect();
    for (child, _) in &spawned {
        assert!(
            ended.contains(&child.id()),
            "every started child is ended or was refused: {} is neither",
            child.id()
        );
    }
    for (mut child, guard) in spawned {
        let _ = child.wait();
        drop(guard);
    }
    let left = support::running(&marker);
    support::end_all(&left);
    assert!(left.is_empty(), "a concurrent child survived: {left:?}");
}

/// A `sleep 300` or the refusal to start it.
#[cfg(unix)]
fn spawn_sleep_or_refusal_300() -> Result<
    (std::process::Child, lexlean::verify::lifeline::Guard),
    lexlean::verify::lifeline::CannotStart,
> {
    use std::os::unix::process::CommandExt;
    let mut command = std::process::Command::new("sleep");
    command
        .arg(format!("300.{}", std::process::id()))
        .process_group(0)
        .stdin(std::process::Stdio::null());
    lexlean::verify::lifeline::spawn(&mut command)
}

/// The limits of an ordinary project with the given child timeout.
#[cfg(unix)]
fn limits(timeout_ms: u64) -> lexlean::config::Limits {
    lexlean::config::Limits {
        max_file_bytes: 4_194_304,
        max_total_source_bytes: 67_108_864,
        max_primitive_atoms: 2_000_000,
        max_token_lattice_edges: 4_000_000,
        max_parse_states: 4_000_000,
        max_ir_nodes: 2_000_000,
        max_scope_depth: 1024,
        max_import_depth: 128,
        max_diagnostics: 256,
        max_child_output_bytes: 16_777_216,
        child_timeout_ms: timeout_ms,
    }
}
