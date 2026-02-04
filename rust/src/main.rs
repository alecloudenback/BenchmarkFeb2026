mod functions;
mod parallel;

use std::{
    error::Error, io, process, time::Instant, time::Duration
};

const NUM_TASKS: u32 = 1000;
const NUM_WORKERS: u8 = 8;
const GENDER: &'static str = "M";
const RISK_CLASS: &'static str = "NS";
const ISSUE_AGE: u8 = 35;
const FACE_AMOUNT: f64 = 100000.0;
const PREMIUM: f64 = 1255.03;

fn result_printer(premium: f64, elapsed: Duration) {
    println!("Results --------------------");
    println!("Last premium: {:.2?}", premium);
    println!("Total time: {:.5?}", elapsed);
    println!("Tasks: {:.0?}", NUM_TASKS);
    println!("Per task: {:.5?}", elapsed / NUM_TASKS);
    println!("----------------------------");
    println!();
}

fn benchmark() -> Result<(), Box<dyn Error>> {
    /* 
    This function runs illustrations and solves for benchmarking purposes
    Tasks are controlled by global parameters NUM_TASKS, GENDER, RISK_CLASS, 
    ISSUE_AGE, FACE_AMOUNT, and PREMIUM
    */

    // Run illustrations
    let mut ill: functions::Illustration = functions::new_illustration(1);

    let now = Instant::now();

    for _i in 0..NUM_TASKS {
        let rates = functions::get_rates(GENDER, RISK_CLASS, ISSUE_AGE)?;
        ill = functions::at_issue_projection(&rates, ISSUE_AGE, FACE_AMOUNT, PREMIUM)?;
    }
    let elapsed = now.elapsed();
    result_printer(ill.premium[0], elapsed);

    // Run parallel illustrates
    let now = Instant::now();
    let mut tasks: Vec<parallel::Task> = Vec::with_capacity(NUM_TASKS as usize);
    for _i in 0..NUM_TASKS {
        tasks.push(parallel::new_default_task());
    }
    ill = parallel::parallel(NUM_WORKERS, "illustrate", tasks);
    let elapsed = now.elapsed();
    result_printer(PREMIUM, elapsed);

    // Run solves
    let now = Instant::now();
    for _i in 0..NUM_TASKS {
        let rates = functions::get_rates(GENDER, RISK_CLASS, ISSUE_AGE)?;
        ill = functions::solve_for_premium(&rates, ISSUE_AGE, FACE_AMOUNT)?;
    }
    let elapsed = now.elapsed();
    result_printer(ill.premium[0], elapsed);

    // Run parallel solves
    let now = Instant::now();
    tasks = Vec::with_capacity(NUM_TASKS as usize);
    for _i in 0..NUM_TASKS {
        tasks.push(parallel::new_default_task());
    }
    ill = parallel::parallel(NUM_WORKERS, "solve", tasks);
    let elapsed = now.elapsed();
    result_printer(ill.premium[0], elapsed);

    Ok(())
}

fn pause() {
    println!("Press ENTER to continue...");
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read line");
}

fn main() {
    if let Err(err) = benchmark() {
        println!("{}", err);
        pause();
        process::exit(1);
    }
    pause();
}
