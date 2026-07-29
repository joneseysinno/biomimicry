//! biomimicry-inspector — render causal DAG + attractor landscape traces.

use std::env;
use std::sync::Arc;

use biomimicry::attractor::{Basin, Landscape, settle_trace};
use biomimicry::causality::causal_order_dot;
use biomimicry::expression::{NetworkRegulator, m4_handles, m4_network, m4_transducer};
use biomimicry::membrane::{echo_ready, echo_request};
use biomimicry::metabolism::Cadence;
use biomimicry::organism::{OrganismBuilder, trigger_signal};
use biomimicry_aec::run_reflex;

#[derive(Clone, Copy)]
enum Scenario {
    Cascade,
    Echo,
    WallMove,
}

fn parse_args() -> (u64, Scenario, u64, bool) {
    let mut seed = 42u64;
    let mut scenario = Scenario::Cascade;
    let mut max_ticks = 32u64;
    let mut checkpoint = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                seed = args
                    .next()
                    .expect("--seed value")
                    .parse()
                    .expect("seed u64");
            }
            "--scenario" => {
                let v = args.next().expect("--scenario value");
                scenario = match v.as_str() {
                    "cascade" => Scenario::Cascade,
                    "echo" => Scenario::Echo,
                    "wall-move" => Scenario::WallMove,
                    other => panic!("unknown scenario {other}; use cascade|echo|wall-move"),
                };
            }
            "--max-ticks" => {
                max_ticks = args
                    .next()
                    .expect("--max-ticks value")
                    .parse()
                    .expect("max-ticks u64");
            }
            "--checkpoint" => checkpoint = true,
            "--help" | "-h" => {
                eprintln!(
                    "Usage: biomimicry-inspector [--seed N] [--scenario cascade|echo|wall-move] [--max-ticks N] [--checkpoint]"
                );
                std::process::exit(0);
            }
            other => panic!("unknown arg {other}"),
        }
    }
    (seed, scenario, max_ticks, checkpoint)
}

fn main() {
    let (seed, scenario, max_ticks, do_checkpoint) = parse_args();
    println!("biomimicry-inspector scenario={scenario:?} seed={seed}");

    match scenario {
        Scenario::Cascade => run_cascade(seed, max_ticks, do_checkpoint),
        Scenario::Echo => run_echo(seed, max_ticks, do_checkpoint),
        Scenario::WallMove => run_wall(seed, max_ticks),
    }
}

impl std::fmt::Debug for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cascade => write!(f, "cascade"),
            Self::Echo => write!(f, "echo"),
            Self::WallMove => write!(f, "wall-move"),
        }
    }
}

fn run_cascade(seed: u64, max_ticks: u64, do_checkpoint: bool) {
    let handles = m4_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(seed)
        .cadence(Cadence::new(2))
        .population_size(2)
        .seed_gene(handles.cascade_path)
        .regulator(NetworkRegulator::new(m4_network(handles.downstream)))
        .transducer(m4_transducer(handles.cascade_path))
        .build()
        .expect("build");

    org.perturb(trigger_signal()).expect("perturb");
    let status = org.settle(max_ticks).expect("settle");
    print_landscape_and_traces(&org.scheduler.log, org.trajectory(), status);

    if do_checkpoint {
        org.open_commit_gate();
        let meta = org.checkpoint("inspector", false).expect("checkpoint");
        println!("checkpoint id={}", meta.id.0);
    }
}

fn run_echo(seed: u64, max_ticks: u64, do_checkpoint: bool) {
    let mut org = echo_ready(seed);
    let _ = org.ingress(echo_request(100)).expect("ingress");
    let status = org.settle(max_ticks).expect("settle");
    print_landscape_and_traces(&org.scheduler.log, org.trajectory(), status);
    if do_checkpoint {
        org.open_commit_gate();
        let meta = org.checkpoint("inspector-echo", false).expect("checkpoint");
        println!("checkpoint id={}", meta.id.0);
    }
}

fn run_wall(seed: u64, max_ticks: u64) {
    let (_org, report) = run_reflex(seed, max_ticks).expect("wall-move");
    println!(
        "wall-move settle={:?} cascade_fired={} chosen={:?}",
        report.settle, report.cascade_fired, report.chosen_option
    );
    println!("tags={:?}", report.reflex_tags);
}

fn print_landscape_and_traces(
    log: &biomimicry::causality::CausalEventLog,
    trajectory: &[u128],
    status: biomimicry::attractor::SettleStatus,
) {
    let fp = trajectory.last().copied().unwrap_or(0);
    let basin = Basin::new(fp, 0);
    let mut land = Landscape::new();
    land.insert(fp, 0);
    println!("basin center={fp:032x} contains={}", basin.contains(fp));
    println!("landscape potential={}", land.potential(fp));
    print!("{}", settle_trace(log, trajectory, status));
    print!("{}", causal_order_dot(log));
}
