use crate::synthesis::{SynthesisEngine, SynthesisCursor};
use crate::encoder::IncumbentManager;
use crate::checkpoint::ReplayCheckpoint;

/// A stateful scheduler task that can be yielded and resumed.
pub struct SynthesisTask {
    /// The target bytes to encode.
    pub target_bytes: Vec<u8>,
    /// The manager tracking the best found encoder graph.
    pub incumbent: IncumbentManager,
    /// The engine enumerating candidate graphs.
    pub engine: SynthesisEngine,
    /// The memory budget for evaluation.
    pub memory_limit: usize,
    /// The total logical steps taken.
    pub steps_taken: u64,
}

impl SynthesisTask {
    /// Bootstraps a new synthesis task from target bytes.
    pub fn new(target_bytes: Vec<u8>, memory_limit: usize) -> Self {
        let incumbent = IncumbentManager::new(target_bytes.clone());
        let engine = SynthesisEngine::new(SynthesisCursor::default());
        Self {
            target_bytes,
            incumbent,
            engine,
            memory_limit,
            steps_taken: 0,
        }
    }

    /// Resumes the task by deterministically replaying work up to the checkpoint.
    /// Enforces the requirement that no trusted state (like opaque cursors) is imported.
    /// Everything is reconstructed exactly from initial conditions.
    pub fn resume_from_checkpoint(target_bytes: Vec<u8>, memory_limit: usize, checkpoint: &ReplayCheckpoint) -> Self {
        let mut task = Self::new(target_bytes, memory_limit);
        task.step(checkpoint.replay_steps as usize);
        task
    }

    /// Generates a checkpoint for the current task state.
    pub fn create_checkpoint(&self, target_identity: [u8; 32], policy_identity: [u8; 32]) -> ReplayCheckpoint {
        ReplayCheckpoint::new(target_identity, policy_identity, self.steps_taken)
    }

    /// Performs one quantum of work, enumerating candidate graphs up to `quantum` steps.
    /// Returns true if work was performed, false if the synthesis space is exhausted.
    pub fn step(&mut self, quantum: usize) -> bool {
        let mut performed = false;
        for _ in 0..quantum {
            if let Some(candidate) = self.engine.next_candidate(self.target_bytes.len()) {
                self.incumbent.evaluate_candidate(candidate, self.memory_limit);
                self.steps_taken += 1;
                performed = true;
            } else {
                break;
            }
        }
        performed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_resume_invariance() {
        // We run a task for 2 steps.
        let target = b"hello".to_vec();
        let mut full_task = SynthesisTask::new(target.clone(), 1024);
        
        // Step 1
        full_task.step(1);
        let steps_after_1 = full_task.steps_taken;
        
        // Step 2
        full_task.step(1);
        let incumbent_after_2 = full_task.incumbent.best_graph.clone();

        // Now create a checkpoint exactly from step 1
        let ckpt = ReplayCheckpoint::new([0u8; 32], [0u8; 32], steps_after_1);

        // Resume a new task from the checkpoint and run 1 step.
        let mut resumed_task = SynthesisTask::resume_from_checkpoint(target.clone(), 1024, &ckpt);
        resumed_task.step(1);

        // The state should be exactly equivalent.
        assert_eq!(resumed_task.incumbent.best_graph, incumbent_after_2);
        assert_eq!(resumed_task.steps_taken, 2);
    }
}
