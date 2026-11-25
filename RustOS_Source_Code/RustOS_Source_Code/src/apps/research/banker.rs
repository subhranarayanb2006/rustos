//! Deadlock Detection Algorithm (Work-Finish Reduction / Banker's Detection)
//! Implements standard deadlock detection for multiple resource types (Silberschatz et al.)

use alloc::vec::Vec;
use core::arch::x86::_rdtsc;

pub const MAX_PROCESSES: usize = 8;
pub const MAX_RESOURCES: usize = 6;

#[derive(Debug, Clone)]
pub struct ResourceModel {
    pub name: &'static str,
    pub num_processes: usize,
    pub num_resources: usize,
    pub resource_names: [&'static str; MAX_RESOURCES],
    pub total_instances: [u32; MAX_RESOURCES],
    pub allocation: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
    pub request: [[u32; MAX_RESOURCES]; MAX_PROCESSES],
    pub available: [u32; MAX_RESOURCES],
}

#[derive(Debug, Clone)]
pub struct DetectionStep {
    pub process: usize,
    pub work_before: [u32; MAX_RESOURCES],
    pub work_after: [u32; MAX_RESOURCES],
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub is_deadlocked: bool,
    pub deadlocked_processes: Vec<usize>,
    pub completed_processes: Vec<usize>,
    pub iterations: usize,
    pub execution_cycles: u64,
    pub steps: Vec<DetectionStep>,
    pub final_work: [u32; MAX_RESOURCES],
}

pub fn detect_deadlock(model: &ResourceModel) -> DetectionResult {
    let start_tsc = unsafe { _rdtsc() };

    let n = model.num_processes;
    let m = model.num_resources;

    let mut work = model.available;
    let mut finish = [false; MAX_PROCESSES];
    let mut steps = Vec::new();
    let mut iterations = 0;

    // Step 1: If Allocation[i] == 0, finish[i] = true (processes holding no resources cannot cause deadlock)
    for i in 0..n {
        let holds_any = (0..m).any(|r| model.allocation[i][r] > 0);
        if !holds_any {
            finish[i] = true;
        }
    }

    // Step 2 & 3: Reduction Loop
    loop {
        iterations += 1;
        let mut found = false;

        for i in 0..n {
            if !finish[i] {
                // Check if Request[i] <= Work
                let can_satisfy = (0..m).all(|r| model.request[i][r] <= work[r]);
                if can_satisfy {
                    let before = work;
                    // Work += Allocation[i]
                    for r in 0..m {
                        work[r] += model.allocation[i][r];
                    }
                    finish[i] = true;
                    found = true;

                    steps.push(DetectionStep {
                        process: i,
                        work_before: before,
                        work_after: work,
                    });
                    break; // Restart scan
                }
            }
        }

        if !found {
            break;
        }
    }

    let end_tsc = unsafe { _rdtsc() };
    let execution_cycles = end_tsc.saturating_sub(start_tsc);

    let mut deadlocked = Vec::new();
    let mut completed = Vec::new();

    for i in 0..n {
        if !finish[i] {
            deadlocked.push(i);
        } else {
            completed.push(i);
        }
    }

    DetectionResult {
        is_deadlocked: !deadlocked.is_empty(),
        deadlocked_processes: deadlocked,
        completed_processes: completed,
        iterations,
        execution_cycles,
        steps,
        final_work: work,
    }
}
