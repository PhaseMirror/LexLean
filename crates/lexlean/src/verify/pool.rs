//! The scheduler of the independent proof processes of one verification stage
//! (SPEC.md §22.11).
//!
//! The processes of a frontier of module elaboration, of the replays, and of
//! the audit members do not depend on each other, so up to `width` of them run
//! at once. What the caller sees does not depend on how many did: results are
//! handed to it one at a time, in the order of the items, whatever order the
//! processes finished in. That is the whole of the contract, and it is why a
//! verification at width 4 publishes the bytes a verification at width 1
//! does — nothing downstream of this function can observe a completion order.
//!
//! A failure is the first one in item order, too. Items are claimed in order,
//! so when item `f` fails every earlier item has been claimed and is
//! finished before this returns; a later item that is already running is
//! finished and its result dropped; no item after a known failure is started.
//! The failure reported is therefore the failure width 1 would have stopped at.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

use crate::code;
use crate::diagnostic::Diagnostic;
use crate::error::LexLeanError;
use crate::verify::telemetry;

fn internal(message: &str) -> LexLeanError {
    LexLeanError::from_diagnostic(Diagnostic::new(
        code!("LLI9001"),
        format!("phase verify: {message}"),
    ))
}

/// Run `job` on each of `items`, at most `width` at a time, and hand each
/// result to `consume` in the order of `items`.
///
/// `is_failure` says whether a result is one `consume` will refuse; at width
/// above 1 it lets the workers stop early, and `consume` remains the judge: it must
/// return an error for every result `is_failure` accepts, and the first error
/// it returns in item order is the one returned here.
///
/// # Errors
///
/// The first error `consume` returns, or `LLI9001` when a worker panicked or
/// stopped without a failure having been consumed.
pub fn run_ordered<R, C>(
    width: usize,
    items: &[usize],
    job: &(impl Fn(usize) -> R + Sync),
    is_failure: &(impl Fn(&R) -> bool + Sync),
    mut consume: C,
) -> Result<(), LexLeanError>
where
    R: Send,
    C: FnMut(usize, R) -> Result<(), LexLeanError>,
{
    if items.is_empty() {
        return Ok(());
    }
    // Width 1 is the sequence the verifier ran before there was a width: item
    // `i + 1` starts only after item `i` has been consumed, on the calling
    // thread, so a failure only `consume` can see (a missing `.olean`, a
    // staging error) stops the stage before another process is started.
    if width == 1 {
        for (index, &item) in items.iter().enumerate() {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(item)))
                .map_err(|_| internal("verification process worker panicked"))?;
            telemetry::completed(item, index + 1);
            let failed = is_failure(&result);
            consume(item, result)?;
            if failed {
                return Err(internal(
                    "verification stage consumed a failed process without refusing it",
                ));
            }
        }
        return Ok(());
    }
    let workers = width.clamp(1, items.len());
    let next = AtomicUsize::new(0);
    // The least position known to have failed. Positions after it are not
    // started; positions before it still are, because the failure to report
    // is the least one and they may hold it.
    let stop_after = AtomicUsize::new(usize::MAX);
    let (sender, receiver) = mpsc::channel::<(usize, R)>();
    let context = telemetry::current();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                let sender = sender.clone();
                let context = context.clone();
                let (next, stop_after) = (&next, &stop_after);
                scope.spawn(move || {
                    let _installed = telemetry::install(context);
                    loop {
                        let position = next.fetch_add(1, Ordering::SeqCst);
                        if position >= items.len() || position > stop_after.load(Ordering::SeqCst) {
                            break;
                        }
                        let result = job(items[position]);
                        if is_failure(&result) {
                            stop_after.fetch_min(position, Ordering::SeqCst);
                        }
                        if sender.send((position, result)).is_err() {
                            break;
                        }
                    }
                })
            })
            .collect();
        drop(sender);

        let mut arrived: BTreeMap<usize, R> = BTreeMap::new();
        let mut consumed = 0usize;
        let mut outcome: Result<(), LexLeanError> = Ok(());
        let mut sequence = 0usize;
        for (position, result) in receiver {
            sequence += 1;
            telemetry::completed(items[position], sequence);
            arrived.insert(position, result);
            while outcome.is_ok() {
                let Some(ready) = arrived.remove(&consumed) else {
                    break;
                };
                if let Err(error) = consume(items[consumed], ready) {
                    stop_after.fetch_min(consumed, Ordering::SeqCst);
                    outcome = Err(error);
                }
                consumed += 1;
            }
        }
        // Every worker has dropped its sender, so each has finished (or
        // panicked) and nothing started here outlives this call.
        let panicked = handles
            .into_iter()
            .map(std::thread::ScopedJoinHandle::join)
            .any(|joined| joined.is_err());
        if outcome.is_ok() && panicked {
            return Err(internal("verification process worker panicked"));
        }
        if outcome.is_ok() && consumed != items.len() {
            return Err(internal(
                "verification stage stopped before every process was consumed",
            ));
        }
        outcome
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{mpsc, Mutex};
    use std::time::Duration;

    use super::run_ordered;
    use crate::diagnostic::Diagnostic;
    use crate::error::LexLeanError;

    fn refusal(text: &str) -> LexLeanError {
        LexLeanError::from_diagnostic(Diagnostic::new(crate::code!("LLI9001"), text))
    }

    /// Items 0..n finish in reverse order: item `i` waits for item `i + 1`.
    /// The results are still consumed in item order.
    #[test]
    fn results_are_consumed_in_item_order_whatever_order_they_finish_in() {
        let items: Vec<usize> = (0..4).collect();
        let senders: Vec<_> = (0..items.len()).map(|_| mpsc::channel::<()>()).collect();
        let (txs, rxs): (Vec<_>, Vec<_>) = senders.into_iter().unzip();
        let rxs: Vec<Mutex<mpsc::Receiver<()>>> = rxs.into_iter().map(Mutex::new).collect();
        let finished = Mutex::new(Vec::new());
        let mut consumed = Vec::new();
        run_ordered(
            4,
            &items,
            &|item: usize| {
                if item + 1 < 4 {
                    rxs[item]
                        .lock()
                        .expect("receiver lock")
                        .recv_timeout(Duration::from_secs(5))
                        .expect("the next item finishes first");
                }
                finished.lock().expect("finished lock").push(item);
                if item > 0 {
                    txs[item - 1].send(()).expect("wake the previous item");
                }
                item * 10
            },
            &|_: &usize| false,
            |item, value| {
                assert_eq!(value, item * 10);
                consumed.push(item);
                Ok(())
            },
        )
        .expect("all four succeed");
        assert_eq!(
            *finished.lock().expect("finished lock"),
            vec![3, 2, 1, 0],
            "the jobs really did finish in reverse order"
        );
        assert_eq!(
            consumed,
            vec![0, 1, 2, 3],
            "and were consumed in item order"
        );
    }

    /// Two items fail; the one with the least position is the one reported,
    /// though the later one finishes first.
    #[test]
    fn the_first_failure_in_item_order_is_the_one_reported() {
        let items: Vec<usize> = (0..4).collect();
        let (tx, rx) = mpsc::channel::<()>();
        let rx = Mutex::new(rx);
        let outcome = run_ordered(
            4,
            &items,
            &|item: usize| {
                match item {
                    1 => {
                        rx.lock()
                            .expect("receiver lock")
                            .recv_timeout(Duration::from_secs(5))
                            .expect("item 3 fails first");
                    }
                    3 => tx.send(()).expect("wake item 1"),
                    _ => {}
                }
                item
            },
            &|value: &usize| *value == 1 || *value == 3,
            |item, value| {
                if value == 1 || value == 3 {
                    Err(refusal(&format!("item {item} failed")))
                } else {
                    Ok(())
                }
            },
        );
        let error = outcome.expect_err("two items fail");
        assert!(
            error.to_string().contains("item 1 failed"),
            "the first failure in item order is reported, not the first to finish: {error}"
        );
    }

    /// At width 1 an item after a failure is never started, and at any width
    /// the number of items started after the failed one stays below `width`.
    #[test]
    fn nothing_is_started_after_a_known_failure_at_width_one() {
        let items: Vec<usize> = (0..6).collect();
        let started = AtomicUsize::new(0);
        let outcome = run_ordered(
            1,
            &items,
            &|item: usize| {
                started.fetch_add(1, Ordering::SeqCst);
                item
            },
            &|value: &usize| *value == 2,
            |_, value| {
                if value == 2 {
                    Err(refusal("item 2 failed"))
                } else {
                    Ok(())
                }
            },
        );
        assert!(outcome.is_err());
        assert_eq!(started.load(Ordering::SeqCst), 3, "items 0, 1 and 2 only");
    }

    /// At width 1 the stage is strictly sequential: a job starts only after
    /// the previous result has been consumed.
    #[test]
    fn width_one_alternates_job_and_consume() {
        let items: Vec<usize> = (0..4).collect();
        let log = Mutex::new(Vec::new());
        let mut order = Vec::new();
        run_ordered(
            1,
            &items,
            &|item: usize| {
                log.lock().expect("lock").push(format!("job {item}"));
                item
            },
            &|_: &usize| false,
            |item, _| {
                log.lock().expect("lock").push(format!("consume {item}"));
                order.push(item);
                Ok(())
            },
        )
        .expect("four jobs");
        assert_eq!(
            *log.lock().expect("lock"),
            [
                "job 0",
                "consume 0",
                "job 1",
                "consume 1",
                "job 2",
                "consume 2",
                "job 3",
                "consume 3"
            ]
        );
        assert_eq!(order, [0, 1, 2, 3]);
        // A failure only `consume` can see ends the stage before the next job.
        let started = AtomicUsize::new(0);
        let outcome = run_ordered(
            1,
            &items,
            &|item: usize| {
                started.fetch_add(1, Ordering::SeqCst);
                item
            },
            &|_: &usize| false,
            |item, _| {
                if item == 0 {
                    Err(refusal("only consume sees this"))
                } else {
                    Ok(())
                }
            },
        );
        assert!(outcome.is_err());
        assert_eq!(
            started.load(Ordering::SeqCst),
            1,
            "no process after the refusal"
        );
    }

    /// Of two failures known out of item order, the earlier position bounds the
    /// items that may start: with failures at 1 and then at 3 (both already
    /// running), none of the eight items after them is started, and the failure
    /// reported is item 1's. (`stop_after` takes the minimum; that a later,
    /// larger position cannot reopen the stage is also a consequence of items
    /// being claimed in order, so this test fixes the behaviour and no mutant
    /// of the store survives that this could tell apart.)
    #[test]
    fn a_later_failure_does_not_reopen_the_stage() {
        let items: Vec<usize> = (0..12).collect();
        let started = Mutex::new(Vec::new());
        let (first_failed, wait_for_first) = mpsc::channel::<()>();
        let wait_for_first = Mutex::new(wait_for_first);
        let third_finished = std::sync::atomic::AtomicBool::new(false);
        let outcome = run_ordered(
            4,
            &items,
            &|item: usize| {
                started.lock().expect("lock").push(item);
                match item {
                    // Item 1 fails first, item 3 after it; items 0 and 2 stay
                    // busy until both have, so no worker is free to claim more
                    // before the failures are known.
                    1 => {
                        // Not before item 3 is running: it fails after this.
                        let give_up = std::time::Instant::now() + Duration::from_secs(10);
                        while started.lock().expect("lock").len() < 4
                            && std::time::Instant::now() < give_up
                        {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        first_failed.send(()).expect("announce");
                    }
                    3 => {
                        wait_for_first
                            .lock()
                            .expect("lock")
                            .recv_timeout(Duration::from_secs(10))
                            .expect("item 1 failed first");
                        std::thread::sleep(Duration::from_millis(50));
                        third_finished.store(true, Ordering::SeqCst);
                    }
                    0 | 2 => {
                        let give_up = std::time::Instant::now() + Duration::from_secs(10);
                        while !third_finished.load(Ordering::SeqCst)
                            && std::time::Instant::now() < give_up
                        {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    _ => {}
                }
                item
            },
            &|item: &usize| *item == 1 || *item == 3,
            |item, value| {
                if value == 1 || value == 3 {
                    Err(refusal(&format!("item {item} failed")))
                } else {
                    Ok(())
                }
            },
        );
        let error = outcome.expect_err("two items fail");
        assert!(error.to_string().contains("item 1 failed"), "{error}");
        let mut seen = started.lock().expect("lock").clone();
        seen.sort_unstable();
        assert_eq!(
            seen,
            [0, 1, 2, 3],
            "nothing after the failure at item 1 started"
        );
    }

    #[test]
    fn a_panicking_worker_is_an_internal_error_not_a_hang() {
        let items: Vec<usize> = (0..3).collect();
        let outcome = run_ordered(
            2,
            &items,
            &|item: usize| {
                assert!(item != 1, "a job panics");
                item
            },
            &|_: &usize| false,
            |_, _| Ok(()),
        );
        let error = outcome.expect_err("the panic is reported");
        assert!(error.to_string().contains("LLI9001"), "{error}");
    }

    #[test]
    fn no_items_is_nothing_to_do() {
        run_ordered(3, &[], &|item: usize| item, &|_: &usize| false, |_, _| {
            Ok(())
        })
        .expect("empty");
    }
}
