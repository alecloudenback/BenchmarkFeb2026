package main

import "errors"

// Data structure for a Task object which defines case parameters for illustration or solve routines
type Task struct {
	gender string
	riskClass string
	issueAge uint8
	faceAmount float64
	premium float64
}

// NewTask returns a Task object with
// gender = g 
// riskClass = r
// issueAge = i
// faceAmount = f
// premium = p
func NewTask (g string, r string, i uint8, f float64, p float64) *Task {
	return &Task{
		gender: g,
		riskClass: r,
		issueAge: i,
		faceAmount: f,
		premium: p,
	}
}

// Generates an array of tasks with default values
// gender = M
// riskClass = NS
// issueAge = 35
// faceAmount = 100,000
// premium = 1,255.03
func generate_default_tasks(numTasks uint64) []Task {
	tasks := make([]Task, numTasks)
	for i := 0; i < int(numTasks); i++ {
		tasks[i] = *NewTask("M", "NS", 35, 100000, 1255.03)
	}
	return tasks
}

// worker for illustration tasks
func worker_illustration(id int, taskChannel <-chan Task, resultChannel chan<- *Illustration) {
	for task := range taskChannel {
		rates := get_rates(task.gender, task.riskClass, int(task.issueAge))
		illustration := illustrate(rates, int(task.issueAge), task.faceAmount, task.premium)
		resultChannel <- &illustration
	}
}

// worker for premium solve tasks
func worker_solve(id int, taskChannel <-chan Task, resultChannel chan<- *Illustration) {
	for task := range taskChannel {
		rates := get_rates(task.gender, task.riskClass, int(task.issueAge))
		illustration := solve(rates, int(task.issueAge), task.faceAmount)
		resultChannel <- &illustration
	}
}

// parallel_default_tasks generates numJobs tasks for applicable jobType and executes over numWorkers
// this function returns either
// - error and nil, or 
// - nil and an illustration corresponding to the last job ran
func parallel_default_tasks(numWorkers uint8, numJobs uint64, jobType string) (error, *Illustration) {
	if jobType != "illustrate" && jobType != "solve" {
		return errors.New("Invalid jobType provided, must be 'solve' or 'illustrate'"), nil
	}
	
	taskChannel := make(chan Task) //, numJobs)
	resultChannel := make(chan *Illustration, numJobs)
	tasks := generate_default_tasks(numJobs)

	for i := 1; i <= int(numWorkers); i++ {
		if jobType == "illustrate" {
			go worker_illustration(i, taskChannel, resultChannel)
		} else {
			go worker_solve(i, taskChannel, resultChannel)
		}
	}

	for _, task := range tasks {
		taskChannel <- task
	}
	close(taskChannel)
	
	var illustration *Illustration
	for i:=1; i<=int(numJobs); i++ {
		illustration = <- resultChannel
	}
	close(resultChannel)

	return nil, illustration
}