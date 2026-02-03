use crate::NUM_TASKS;
use crate::functions;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;

const DEFAULT_GENDER: &'static str = "M";
const DEFAULT_RISK_CLASS: &'static str = "NS";
const DEFAULT_ISSUE_AGE:  u8 = 35;
const DEFAULT_FACE_AMOUNT: f64 = 100000.0;
const DEFAULT_PREMIUM: f64 = 1255.03;

pub struct Task {
    pub gender: &'static str,
    pub risk_class: &'static str,
    pub issue_age: u8,
    pub face_amount: f64,
    pub premium: f64,
}

fn new_task(gender: &'static str, risk_class: &'static str, issue_age: u8, face_amount: f64, premium: f64) -> Task {
    Task{gender: gender, risk_class: risk_class, issue_age: issue_age, face_amount: face_amount, premium: premium}
}

pub fn new_default_task() -> Task {
    new_task(DEFAULT_GENDER, DEFAULT_RISK_CLASS, DEFAULT_ISSUE_AGE, DEFAULT_FACE_AMOUNT, DEFAULT_PREMIUM)
}

fn illustrate_worker(receiver: Arc<Mutex<Receiver<Task>>>, sender: Sender<functions::Illustration>) {
    // loop indefinitely to get tasks
    loop {
        let task = receiver.lock().unwrap().recv();

        match task {
            Ok(task_data) => {
                let rates = functions::get_rates(task_data.gender, task_data.risk_class, task_data.issue_age).unwrap();
                let ill = functions::at_issue_projection(&rates, task_data.issue_age, task_data.face_amount, task_data.premium).unwrap();
                sender.send(ill).unwrap();
            }
            Err(_) => {
                // Sender has been dropped, shut down
                break;
            }
        }
    }
}

fn solve_worker(receiver: Arc<Mutex<Receiver<Task>>>, sender: Sender<functions::Illustration>) {
    // loop indefinitely to get tasks
    loop {
        let task = receiver.lock().unwrap().recv();

        match task {
            Ok(task_data) => {
                let rates = functions::get_rates(task_data.gender, task_data.risk_class, task_data.issue_age).unwrap();
                let ill = functions::solve_for_premium(&rates, task_data.issue_age, task_data.face_amount).unwrap();
                sender.send(ill).unwrap();
            }
            Err(_) => {
                // Sender has been dropped, shut down
                break;
            }
        }
    }
}

pub fn parallel(num_workers: u8, task_type: &'static str, tasks: Vec<Task>) -> functions::Illustration {
    if task_type != "illustrate" && task_type != "solve" {
        panic!("Invalid task_type submitted, must be 'illustrate' or 'solve'.");
    }
    // Create a channel
    let (task_sender, task_receiver) = std::sync::mpsc::channel::<Task>();
    let (result_sender, result_receiver) = std::sync::mpsc::channel::<functions::Illustration>();

    // Wrap receiver in an Arc and Mutex for safe access
    let shared_task_receiver = Arc::new(Mutex::new(task_receiver));

    let mut handles = vec![];

    // Spawn threads
    for _i in 0..num_workers {
        let receiver_clone = Arc::clone(&shared_task_receiver);
        let sender_clone = result_sender.clone();

        handles.push(thread::spawn(move || {
            // loop indefinitely to get tasks
            if task_type == "illustrate" {
                illustrate_worker(receiver_clone, sender_clone);
            } else {
                solve_worker(receiver_clone, sender_clone);
            } 
        }))
    }

    // Main thread
    // send all tasks
    for task in tasks {
        task_sender.send(task).unwrap();
    }

    // drop sender to signal no more jobs
    drop(task_sender);

    // collect results
    let mut result = functions::new_illustration(1);
    for _i in 0..NUM_TASKS {
        result = result_receiver.recv().unwrap();
    }
    return result;
}