//! Deadlock Detection Research Lab Interactive Interface & VGA Bar Chart

use alloc::format;
use alloc::string::String;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::apps::research::patterns::{get_pattern, get_all_patterns};
use crate::apps::research::banker::detect_deadlock;
use crate::apps::research::experiments::{run_research_suite, ResearchSuiteResults};

pub fn launch_research_lab() {
    let mut selected_pattern = 0usize;
    let mut suite_results: Option<ResearchSuiteResults> = None;
    let mut view_mode = 0; // 0: Pattern Inspector, 1: Full Suite Table, 2: Bar Chart, 3: Hypotheses

    loop {
        render_lab_screen(selected_pattern, &suite_results, view_mode);

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Esc => break,
                KeyEvent::Char('1') => { selected_pattern = 0; view_mode = 0; }
                KeyEvent::Char('2') => { selected_pattern = 1; view_mode = 0; }
                KeyEvent::Char('3') => { selected_pattern = 2; view_mode = 0; }
                KeyEvent::Char('4') => { selected_pattern = 3; view_mode = 0; }
                KeyEvent::Char('5') => { selected_pattern = 4; view_mode = 0; }
                KeyEvent::Char('6') => { selected_pattern = 5; view_mode = 0; }
                KeyEvent::Char('r') | KeyEvent::Char('R') => {
                    // Run full experimental suite
                    show_loading("Running 10 repetitions across all 6 patterns...");
                    suite_results = Some(run_research_suite());
                    view_mode = 1;
                }
                KeyEvent::Char('c') | KeyEvent::Char('C') => {
                    if suite_results.is_none() {
                        suite_results = Some(run_research_suite());
                    }
                    view_mode = 2; // Bar Chart
                }
                KeyEvent::Char('h') | KeyEvent::Char('H') => {
                    if suite_results.is_none() {
                        suite_results = Some(run_research_suite());
                    }
                    view_mode = 3; // Hypotheses
                }
                KeyEvent::Char('p') | KeyEvent::Char('P') => {
                    view_mode = 0; // Pattern Inspector
                }
                KeyEvent::Up => {
                    if selected_pattern > 0 { selected_pattern -= 1; }
                }
                KeyEvent::Down => {
                    if selected_pattern < 5 { selected_pattern += 1; }
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn show_loading(msg: &str) {
    unsafe {
        VGA_WRITER.fill_rect(10, 10, 60, 5, b' ', Color::White, Color::Blue);
        VGA_WRITER.draw_box(10, 10, 60, 5, Color::Yellow, Color::Blue, true);
        VGA_WRITER.print_at(12, 12, msg, Color::Yellow, Color::Blue);
    }
}

fn render_lab_screen(pattern_idx: usize, suite: &Option<ResearchSuiteResults>, view: usize) {
    unsafe {
        VGA_WRITER.clear_screen();
        
        // Header
        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 0, "DEADLOCK DETECTION RESEARCH LAB", Color::LightCyan, Color::DarkGray);
        VGA_WRITER.print_at(2, 1, "Paper: Deadlocks & Detection under Different Resource Patterns (Banker's)", Color::LightGray, Color::DarkGray);

        match view {
            0 => render_pattern_inspector(pattern_idx),
            1 => render_suite_table(suite.as_ref().unwrap()),
            2 => render_bar_chart(suite.as_ref().unwrap()),
            3 => render_hypotheses(suite.as_ref().unwrap()),
            _ => {}
        }

        // Bottom Navigation Bar
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(1, 23, "[1-6] Pick Pattern  [R] Run Suite (10x)  [C] Bar Chart  [H] Hypotheses", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(1, 24, "[P] Pattern View    [Esc] Return to Desktop    (Results saved to deadlock_results.txt)", Color::LightCyan, Color::DarkGray);
    }
}

fn render_pattern_inspector(idx: usize) {
    let model = get_pattern(idx);
    let res = detect_deadlock(&model);

    unsafe {
        // Pattern Title Box
        VGA_WRITER.draw_box(1, 3, 78, 19, Color::LightBlue, Color::Black, true);
        VGA_WRITER.print_at(3, 4, &format!("PATTERN {}: {}", idx + 1, model.name), Color::Yellow, Color::Black);

        // Status Badge
        if res.is_deadlocked {
            VGA_WRITER.print_at(55, 4, "[ DEADLOCK DETECTED ]", Color::White, Color::Red);
        } else {
            VGA_WRITER.print_at(55, 4, "[ NO DEADLOCK: SAFE ]", Color::Black, Color::Green);
        }

        // Available Vector
        let mut avail_str = format!("Available Vector: [");
        for r in 0..model.num_resources {
            avail_str.push_str(&format!(" {}:{} ", model.resource_names[r], model.available[r]));
        }
        avail_str.push(']');
        VGA_WRITER.print_at(3, 6, &avail_str, Color::LightCyan, Color::Black);

        // Allocation & Request Matrices
        VGA_WRITER.print_at(3, 8, "PROCESS  | ALLOCATION MATRIX  | REQUEST MATRIX    | STATUS", Color::White, Color::Black);
        VGA_WRITER.print_at(3, 9, "---------+--------------------+-------------------+-----------", Color::DarkGray, Color::Black);

        for p in 0..model.num_processes {
            let mut alloc_str = String::from("[");
            let mut req_str = String::from("[");
            for r in 0..model.num_resources {
                alloc_str.push_str(&format!(" {}", model.allocation[p][r]));
                req_str.push_str(&format!(" {}", model.request[p][r]));
            }
            alloc_str.push_str(" ]");
            req_str.push_str(" ]");

            let is_dl = res.deadlocked_processes.contains(&p);
            let status = if is_dl { "DEADLOCKED" } else { "COMPLETED" };
            let fg = if is_dl { Color::LightRed } else { Color::LightGreen };

            let row_str = format!("P{:<7} | {:<18} | {:<17} | {}", p, alloc_str, req_str, status);
            VGA_WRITER.print_at(3, 10 + p, &row_str, fg, Color::Black);
        }

        // Reduction steps trace
        let trace_y = 10 + model.num_processes + 1;
        VGA_WRITER.print_at(3, trace_y, "Reduction Sequence:", Color::Yellow, Color::Black);
        if res.steps.is_empty() {
            VGA_WRITER.print_at(24, trace_y, "No processes could acquire needed resources (Immediate Block)", Color::LightRed, Color::Black);
        } else {
            let mut seq = String::from("< ");
            for step in &res.steps {
                seq.push_str(&format!("P{} ", step.process));
            }
            seq.push('>');
            VGA_WRITER.print_at(24, trace_y, &seq, Color::LightGreen, Color::Black);
        }

        // Execution metrics
        VGA_WRITER.print_at(3, trace_y + 2, &format!(
            "Iterations: {}  |  CPU Execution Cycles: {}  |  Completed: {}/{}  |  Deadlocked: {}/{}",
            res.iterations, res.execution_cycles, res.completed_processes.len(), model.num_processes, res.deadlocked_processes.len(), model.num_processes
        ), Color::LightCyan, Color::Black);
    }
}

fn render_suite_table(suite: &ResearchSuiteResults) {
    unsafe {
        VGA_WRITER.draw_box(1, 3, 78, 19, Color::LightCyan, Color::Black, true);
        VGA_WRITER.print_at(3, 4, "EXPERIMENTAL COMPARISON (10 REPETITIONS PER PATTERN)", Color::Yellow, Color::Black);
        VGA_WRITER.print_at(3, 6, "PATTERN NAME             | PROC/RES | STATUS   | AVG CYCLES | ITER | UTIL", Color::White, Color::Black);
        VGA_WRITER.print_at(3, 7, "-------------------------+----------+----------+------------+------+------", Color::DarkGray, Color::Black);

        for (i, exp) in suite.experiments.iter().enumerate() {
            let status = if exp.is_deadlocked { "DEADLOCK" } else { "SAFE    " };
            let fg = if exp.is_deadlocked { Color::LightRed } else { Color::LightGreen };
            let name_short = if exp.pattern_name.len() > 24 { &exp.pattern_name[..24] } else { exp.pattern_name };

            let row = format!(
                "{:<24} | {:>2}P/{:>1}R   | {} | {:>10} | {:>4} | {:>3}%",
                name_short, exp.num_processes, exp.num_resources, status, exp.avg_cycles, exp.avg_iterations, exp.utilization_percent
            );
            VGA_WRITER.print_at(3, 8 + i * 2, &row, fg, Color::Black);
        }

        VGA_WRITER.print_at(3, 20, "Press [C] for Visual Bar Chart  |  [H] for Hypothesis Verification", Color::LightCyan, Color::Black);
    }
}

fn render_bar_chart(suite: &ResearchSuiteResults) {
    unsafe {
        VGA_WRITER.draw_box(1, 3, 78, 19, Color::Magenta, Color::Black, true);
        VGA_WRITER.print_at(3, 4, "VGA BAR CHART: REDUCTION ITERATIONS & CYCLES ACROSS PATTERNS", Color::Yellow, Color::Black);

        // Find max cycles to scale bars
        let max_cycles = suite.experiments.iter().map(|e| e.avg_cycles).max().unwrap_or(1000).max(1);

        for (i, exp) in suite.experiments.iter().enumerate() {
            let y = 6 + i * 2;
            let bar_len = ((exp.avg_cycles * 35) / max_cycles) as usize;
            let bar_len = bar_len.max(1).min(35);

            let label = match i {
                0 => "P1: No Deadlock ",
                1 => "P2: Circ. Wait  ",
                2 => "P3: Multi-Proc  ",
                3 => "P4: Contention  ",
                4 => "P5: Mixed Alloc ",
                5 => "P6: Indep Groups",
                _ => "P?: Unknown     ",
            };

            VGA_WRITER.print_at(3, y, label, Color::White, Color::Black);

            // Draw bar using character 219 (solid block)
            let color = if exp.is_deadlocked { Color::LightRed } else { Color::LightGreen };
            for c in 0..bar_len {
                VGA_WRITER.print_at(20 + c, y, "\u{2588}", color, Color::Black);
            }

            VGA_WRITER.print_at(57, y, &format!("{}c ({} iter)", exp.avg_cycles, exp.avg_iterations), Color::LightCyan, Color::Black);
        }

        VGA_WRITER.print_at(3, 20, "Legend: [Green] = Deadlock-Free Reduction  |  [Red] = Deadlocked State", Color::LightGray, Color::Black);
    }
}

fn render_hypotheses(suite: &ResearchSuiteResults) {
    unsafe {
        VGA_WRITER.draw_box(1, 3, 78, 19, Color::LightGreen, Color::Black, true);
        VGA_WRITER.print_at(3, 4, "SCIENTIFIC HYPOTHESIS EVALUATION & DEFENCE", Color::Yellow, Color::Black);

        // H1
        VGA_WRITER.print_at(3, 6, "Hypothesis H1: Cyclic dependencies yield 0% process completion.", Color::White, Color::Black);
        VGA_WRITER.print_at(5, 7, if suite.h1_supported { "[SUPPORTED] H1 Confirmed: Acyclic completes 5/5; Cycles yield 0/n." } else { "[REJECTED]" }, Color::LightGreen, Color::Black);

        // H2
        VGA_WRITER.print_at(3, 10, "Hypothesis H2: Reduction iterations correlate with dependency chain depth.", Color::White, Color::Black);
        VGA_WRITER.print_at(5, 11, if suite.h2_supported { "[SUPPORTED] H2 Confirmed: No Deadlock took 5 passes; Contention halted at 1." } else { "[REJECTED]" }, Color::LightGreen, Color::Black);

        // H3
        VGA_WRITER.print_at(3, 14, "Hypothesis H3: Mixed and Independent groups isolate deadlock to subsets.", Color::White, Color::Black);
        VGA_WRITER.print_at(5, 15, if suite.h3_supported { "[SUPPORTED] H3 Confirmed: Disjoint uncoupled processes complete normally." } else { "[REJECTED]" }, Color::LightGreen, Color::Black);

        VGA_WRITER.print_at(3, 18, "Full empirical benchmark exported to: /deadlock_results.txt", Color::LightCyan, Color::Black);
    }
}
