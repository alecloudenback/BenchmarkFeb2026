package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"time"
)

var numTasks uint64 = 1000
var numWorkers uint8 = 8

// result_printer is a helper function to print data to the console
func result_printer(elapsed time.Duration, numTasks uint64, lastPremium float64) {
	fmt.Println("Results -------------------")
	fmt.Println("Last premium:", lastPremium)
	fmt.Println("Total time:", elapsed)
	fmt.Println("Tasks:", numTasks)
	fmt.Printf("Per task: %.4f\n", float64(elapsed) / float64(numTasks) / (1000000000.0))
	fmt.Println("---------------------------")
	fmt.Println()
}

// benchmark runs 4 tests for comparison purposes
// 1. N illustrations over 1 worker
// 2. N illustrations over W workers
// 3. N premium solves over 1 worker
// 4. N premium solves over W workers
// N and W are set by global parameters numTasks and numWorkers respectively
func benchmark() {
	issueAge := 35
	gender := "M"
	riskClass := "NS"
	faceAmount := 100000.00
	premium := 1255.03

	// Illustrations, single process
	fmt.Println("Starting", numTasks,"illustrations with 1 worker...")
	start := time.Now()
	for i:=0; i < int(numTasks); i++ {
		rates := get_rates(gender, riskClass, issueAge)
		_ = illustrate(rates, issueAge, faceAmount, premium)
	}
	end := time.Now()
	result_printer(end.Sub(start), numTasks, premium)

	// Illustration, multiple processes
	fmt.Println("Starting", numTasks, "illustrations with", numWorkers, "workers...")
	start = time.Now()
	err, _ := parallel_default_tasks(uint8(numWorkers), uint64(numTasks), "illustrate")
	if err != nil {
		log.Fatal(err)
	}
	end = time.Now()
	result_printer(end.Sub(start), numTasks, premium)

	// Solves, single process
	fmt.Println("Starting", numTasks,"solves with 1 worker...")
	start = time.Now()
	for i:=0; i < int(numTasks); i++ {
		rates := get_rates(gender, riskClass, issueAge)
		_ = solve(rates, issueAge, faceAmount)
	}
	end = time.Now()
	result_printer(end.Sub(start), numTasks, premium)

	// Solves, multiple processes
	fmt.Println("Starting", numTasks, "solves with", numWorkers, "workers...")
	start = time.Now()
	err, _ = parallel_default_tasks(numWorkers, numTasks, "solve")
	if err != nil {
		log.Fatal(err)
	}
	end = time.Now()
	result_printer(end.Sub(start), numTasks, premium)
}

func main() {
	benchmark()
	fmt.Print("Press 'Enter' to continue...")
	bufio.NewReader(os.Stdin).ReadBytes('\n')
}