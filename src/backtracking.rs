// backpropagation library ----------------------------------------------------

use std::{collections::HashSet, hash::Hash};
use itertools::Itertools;

trait State: ToString + Eq + Hash {
    fn is_final(&self) -> bool;
    fn get_possible_successors(&self) -> Vec<Box<Self>>;
}

struct Attempt<AState: State> {
    state: AState,
    successors_to_try: Vec<Box<AState>>,
}

#[derive(PartialEq)]
enum Verbosity {
    Quiet,
    Info,
    Trace,
}

fn get_sequence_to_final_state<AState: State + Clone>(
    initial_state: &AState,
    verbosity: &Verbosity,
) -> Result<Vec<AState>, &'static str> {
    // Things that we can currently try.
    let mut attempts: Vec<Attempt<AState>> = vec![Attempt {
        state: initial_state.clone(),
        successors_to_try: initial_state.get_possible_successors(),
    }];
    // Things that we have tried and failed, so do not re-try.
    let mut dead_ends: HashSet<AState> = HashSet::new();

    loop {
        if let Some(current_attempt) = attempts.last_mut() {
            if let Some(successor) = current_attempt.successors_to_try.pop() {
                // found a successor to try, so try it

                if *verbosity == Verbosity::Trace {
                    println!("Going to evaluate successors of {}.", successor.to_string());
                }

                if successor.is_final() {
                    attempts.push(Attempt {
                        state: successor.as_ref().clone(),
                        successors_to_try: vec![],
                    });
                    return Ok(attempts
                        .iter()
                        .map(|a| a.state.clone())
                        .collect::<Vec<AState>>());
                }

                if attempts
                    .iter()
                    .any(|old_attempt| &(old_attempt.state) == successor.as_ref()) || dead_ends.contains(successor.as_ref())
                {
                    // this has already been tested, so no need to re-try
                    if *verbosity == Verbosity::Trace {
                        println!("{} has been tried before, not considering it.", successor.as_ref().to_string());
                    }
                } else {
                    attempts.push(Attempt {
                        state: successor.as_ref().clone(),
                        successors_to_try: successor.get_possible_successors(),
                    });
                }
            } else {
                // no more successors -- backtrack
                if let Some(attempt) = attempts.pop() {
                    if *verbosity != Verbosity::Quiet {
                        println!("Backtracking from {}", attempt.state.to_string());
                    }
                    dead_ends.insert(attempt.state);
                    if *verbosity == Verbosity::Trace {
                        println!("  Known dead ends: {:?}", dead_ends.iter().map(|s| s.to_string()).format(", "));
                    }

                }
                else {
                    assert!(false); // attempts must not have been empty
                }
            }
        } else {
            return Err(
                "No suitable chain of states found to final state; all possibilities exhausted.",
            );
        }
    }
}

// example 1 ------------------------------------------------------------------

// just count up until a target value is reached

#[derive(Clone, Eq, Hash)]
struct Counter {
    value: i32,
}

impl State for Counter {
    fn get_possible_successors(&self) -> Vec<Box<Self>> {
        vec![Box::new(Self {
            value: self.value + 1,
        })]
    }
    fn is_final(&self) -> bool {
        self.value == 7
    }
}

impl PartialEq for Counter {
    fn eq(&self, other_state: &Self) -> bool {
        other_state.value == self.value
    }
}

impl ToString for Counter {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

fn demo_example_1(verbosity: &Verbosity) {
    println!("");
    println!("Demo example 1");
    match get_sequence_to_final_state::<Counter>(&Counter { value: 1i32 }, verbosity) {
        Err(msg) => eprintln!("even simple example did not work due to '{}'", msg),
        Ok(counter_result) => counter_result
            .iter()
            .for_each(|state| println!("{}", state.value)),
    }
}

// example 2 ------------------------------------------------------------------

// count up by either 2 or 1 until a target value is reached

#[derive(Clone, Eq, Hash)]
struct JumpingCounter {
    value: i32,
}

impl State for JumpingCounter {
    fn get_possible_successors(&self) -> Vec<Box<Self>> {
        // ensure that we do not count up infinitively
        // If we allow value+2 in every case and if we 'miss' the final value,
        // we just count up and up.
        match self.value {
            0..=10 => vec![
                Box::new(Self {
                    value: self.value + 1,
                }),
                Box::new(Self {
                    value: self.value + 2,
                }),
            ],
            _ => vec![],
        }
    }
    fn is_final(&self) -> bool {
        self.value == 4
    }
}

impl PartialEq for JumpingCounter {
    fn eq(&self, other_state: &Self) -> bool {
        other_state.value == self.value
    }
}

impl ToString for JumpingCounter {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}

fn demo_example_2(verbosity: &Verbosity) {
    println!("");
    println!("Demo example 2");
    match get_sequence_to_final_state::<JumpingCounter>(&JumpingCounter { value: 1i32 }, verbosity)
    {
        Err(msg) => eprintln!("even simple example did not work due to '{}'", msg),
        Ok(counter_result) => counter_result
            .iter()
            .for_each(|state| println!("{}", state.value)),
    }
}

// example 3 ------------------------------------------------------------------

#[derive(Clone, Eq, PartialEq, Hash)]
enum Piece {
    Free,
    Ship,
    OneTL,
    OneTR,
    OneBL,
    OneBR,
    TwoDiagDown,
    TwoDiagUp,
    TwoHorT,
    TwoHorL,
    TwoHorB,
    TwoHorR,
    LargeEdgeT,
    LargeEdgeL,
    LargeEdgeB,
    LargeEdgeR,
    LargeCornerTL,
    LargeCornerTT,
    LargeCornerBL,
    LargeCornerBR,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct AsteroidsState(Piece, Piece,  Piece,  Piece,  Piece,  Piece,  Piece,  Piece,  Piece, );

fn main() {
    demo_example_1(&Verbosity::Trace);
    demo_example_2(&Verbosity::Trace);
}
