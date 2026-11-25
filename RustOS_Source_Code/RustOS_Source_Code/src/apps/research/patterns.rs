//! Six Standardized Resource Allocation Benchmark Patterns

use crate::apps::research::banker::{ResourceModel, MAX_PROCESSES, MAX_RESOURCES};

pub fn get_pattern(index: usize) -> ResourceModel {
    match index {
        0 => get_pattern_no_deadlock(),
        1 => get_pattern_circular_wait(),
        2 => get_pattern_multi_process_deadlock(),
        3 => get_pattern_high_contention(),
        4 => get_pattern_mixed_allocation(),
        5 => get_pattern_independent_groups(),
        _ => get_pattern_no_deadlock(),
    }
}

pub fn get_all_patterns() -> [ResourceModel; 6] {
    [
        get_pattern_no_deadlock(),
        get_pattern_circular_wait(),
        get_pattern_multi_process_deadlock(),
        get_pattern_high_contention(),
        get_pattern_mixed_allocation(),
        get_pattern_independent_groups(),
    ]
}

/// Pattern 1: No Deadlock
/// Valid execution reduction sequence exists; all 5 processes complete.
fn get_pattern_no_deadlock() -> ResourceModel {
    let mut alloc = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
    let mut req = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];

    // P0 holds (0, 1, 0), requests (0, 0, 0)
    alloc[0] = [0, 1, 0, 0, 0, 0];
    req[0]   = [0, 0, 0, 0, 0, 0];

    // P1 holds (2, 0, 0), requests (2, 0, 2)
    alloc[1] = [2, 0, 0, 0, 0, 0];
    req[1]   = [2, 0, 2, 0, 0, 0];

    // P2 holds (3, 0, 3), requests (0, 0, 0)
    alloc[2] = [3, 0, 3, 0, 0, 0];
    req[2]   = [0, 0, 0, 0, 0, 0];

    // P3 holds (2, 1, 1), requests (1, 0, 0)
    alloc[3] = [2, 1, 1, 0, 0, 0];
    req[3]   = [1, 0, 0, 0, 0, 0];

    // P4 holds (0, 0, 2), requests (0, 0, 2)
    alloc[4] = [0, 0, 2, 0, 0, 0];
    req[4]   = [0, 0, 2, 0, 0, 0];

    ResourceModel {
        name: "1. No Deadlock (Acyclic Baseline)",
        num_processes: 5,
        num_resources: 3,
        resource_names: ["R0", "R1", "R2", "", "", ""],
        total_instances: [7, 2, 6, 0, 0, 0],
        allocation: alloc,
        request: req,
        available: [0, 0, 0, 0, 0, 0], // Available = Total - sum(Alloc) = (7-7, 2-2, 6-6) = (0, 0, 0)
    }
}

/// Pattern 2: Simple Circular Wait
/// Classic 2-process circular wait: P0 holds R0 requests R1; P1 holds R1 requests R0.
fn get_pattern_circular_wait() -> ResourceModel {
    let mut alloc = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
    let mut req = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];

    alloc[0] = [1, 0, 0, 0, 0, 0];
    req[0]   = [0, 1, 0, 0, 0, 0];

    alloc[1] = [0, 1, 0, 0, 0, 0];
    req[1]   = [1, 0, 0, 0, 0, 0];

    ResourceModel {
        name: "2. Simple Circular Wait (2-Process Cycle)",
        num_processes: 2,
        num_resources: 2,
        resource_names: ["R0", "R1", "", "", "", ""],
        total_instances: [1, 1, 0, 0, 0, 0],
        allocation: alloc,
        request: req,
        available: [0, 0, 0, 0, 0, 0],
    }
}

/// Pattern 3: Multiple Process Deadlock
/// 4 processes forming a multi-hop circular dependency: P0->P1->P2->P3->P0.
fn get_pattern_multi_process_deadlock() -> ResourceModel {
    let mut alloc = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
    let mut req = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];

    alloc[0] = [1, 0, 0, 0, 0, 0];
    req[0]   = [0, 1, 0, 0, 0, 0];

    alloc[1] = [0, 1, 0, 0, 0, 0];
    req[1]   = [0, 0, 1, 0, 0, 0];

    alloc[2] = [0, 0, 1, 0, 0, 0];
    req[2]   = [0, 0, 0, 1, 0, 0];

    alloc[3] = [0, 0, 0, 1, 0, 0];
    req[3]   = [1, 0, 0, 0, 0, 0];

    ResourceModel {
        name: "3. Multiple Process Deadlock (4-Process Ring)",
        num_processes: 4,
        num_resources: 4,
        resource_names: ["R0", "R1", "R2", "R3", "", ""],
        total_instances: [1, 1, 1, 1, 0, 0],
        allocation: alloc,
        request: req,
        available: [0, 0, 0, 0, 0, 0],
    }
}

/// Pattern 4: High Resource Contention
/// Heavy over-subscription of scarce resource instances across 5 processes.
fn get_pattern_high_contention() -> ResourceModel {
    let mut alloc = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
    let mut req = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];

    alloc[0] = [2, 1, 0, 0, 0, 0];
    req[0]   = [1, 2, 1, 0, 0, 0];

    alloc[1] = [1, 2, 1, 0, 0, 0];
    req[1]   = [2, 0, 2, 0, 0, 0];

    alloc[2] = [1, 0, 2, 0, 0, 0];
    req[2]   = [0, 2, 1, 0, 0, 0];

    alloc[3] = [0, 1, 1, 0, 0, 0];
    req[3]   = [1, 1, 0, 0, 0, 0];

    alloc[4] = [1, 1, 1, 0, 0, 0];
    req[4]   = [1, 0, 1, 0, 0, 0];

    ResourceModel {
        name: "4. High Resource Contention (Over-subscribed)",
        num_processes: 5,
        num_resources: 3,
        resource_names: ["R0", "R1", "R2", "", "", ""],
        total_instances: [5, 5, 5, 0, 0, 0],
        allocation: alloc,
        request: req,
        available: [0, 0, 0, 0, 0, 0],
    }
}

/// Pattern 5: Mixed Allocation
/// Partial completion: P0 and P1 finish, but P2 and P3 remain in deadlocked wait.
fn get_pattern_mixed_allocation() -> ResourceModel {
    let mut alloc = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
    let mut req = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];

    // P0: can finish with Available [1, 0, 0]
    alloc[0] = [1, 0, 0, 0, 0, 0];
    req[0]   = [0, 0, 0, 0, 0, 0];

    // P1: finishes after P0 releases resources
    alloc[1] = [1, 1, 0, 0, 0, 0];
    req[1]   = [1, 0, 0, 0, 0, 0];

    // P2 & P3: deadlock on R1 & R2
    alloc[2] = [0, 2, 0, 0, 0, 0];
    req[2]   = [0, 0, 2, 0, 0, 0];

    alloc[3] = [0, 0, 2, 0, 0, 0];
    req[3]   = [0, 2, 0, 0, 0, 0];

    ResourceModel {
        name: "5. Mixed Allocation (Partial Deadlock)",
        num_processes: 4,
        num_resources: 3,
        resource_names: ["R0", "R1", "R2", "", "", ""],
        total_instances: [3, 3, 2, 0, 0, 0],
        allocation: alloc,
        request: req,
        available: [1, 0, 0, 0, 0, 0],
    }
}

/// Pattern 6: Independent Resource Groups
/// Disjoint resource clusters: Group A (P0, P1) is clean; Group B (P2, P3) is deadlocked.
fn get_pattern_independent_groups() -> ResourceModel {
    let mut alloc = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];
    let mut req = [[0u32; MAX_RESOURCES]; MAX_PROCESSES];

    // Group A (R0, R1) - Acyclic, P0 finishes first then P1
    alloc[0] = [1, 0, 0, 0, 0, 0];
    req[0]   = [0, 0, 0, 0, 0, 0];

    alloc[1] = [0, 1, 0, 0, 0, 0];
    req[1]   = [1, 0, 0, 0, 0, 0];

    // Group B (R2, R3) - Mutually dependent cycle
    alloc[2] = [0, 0, 1, 0, 0, 0];
    req[2]   = [0, 0, 0, 1, 0, 0];

    alloc[3] = [0, 0, 0, 1, 0, 0];
    req[3]   = [0, 0, 1, 0, 0, 0];

    ResourceModel {
        name: "6. Independent Resource Groups (Disjoint Subgraphs)",
        num_processes: 4,
        num_resources: 4,
        resource_names: ["R0_A", "R1_A", "R2_B", "R3_B", "", ""],
        total_instances: [1, 1, 1, 1, 0, 0],
        allocation: alloc,
        request: req,
        available: [0, 0, 0, 0, 0, 0],
    }
}
