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
/// `is_failure` says whether a result is one `consume` will refuse; it only
/// lets the workers stop early, and `consume` remains the judge: it must
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
