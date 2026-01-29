// Implements utility functions for illustration and premium solve benchmarking

package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"log"
	"math"
	"os"
	"strconv"
)

// create_rate_array returns a float64 array with 120 elements all set to value
func create_rate_array(value float64) [120]float64 {
	var array [120]float64
	for i := range len(array) {
		array[i] = value
	}
	return array
}

// get_per_unit_rates returns annual per unit rates in an array where index 0 is for year 1
func get_per_unit_rates(issue_age int) [120]float64 {
	// create default output
	rates := create_rate_array(0)

	// create variables outside of loops
	var age_col, year_col, rate_col int
	var file_age, file_year int
	var file_rate float64

	// open file
	file, err := os.Open("../python/unit_load.csv")
	if err != nil {
		log.Fatal("Error while reading the file", err)
	}

	defer file.Close()
	reader := csv.NewReader(file)
	row, _ := reader.Read()

	for idx, val := range row {
		switch val {
		case "Issue_Age":
			age_col = idx
		case "Policy_Year":
			year_col = idx
		case "Rate":
			rate_col = idx
		}
	}

	for {
		row, err := reader.Read()
		if err == io.EOF {
			break
		}
		file_age, _ = strconv.Atoi(row[age_col])
		if file_age == issue_age {
			file_rate, _ = strconv.ParseFloat(row[rate_col], 64)
			file_year, _ = strconv.Atoi(row[year_col])
			rates[file_year-1] = file_rate
		}
	}
	return rates
}

// get_coi_rates returns annual COI rates in an array where index 0 is for year 1
func get_coi_rates(gender string, risk_class string, issue_age int) [120]float64 {
	// create array
	rates := create_rate_array(0)

	// create variables outside of loops
	var age_col, year_col, rate_col, gender_col, class_col int
	var file_age, file_year int
	var file_rate float64

	// open file
	file, err := os.Open("../python/coi.csv")
	if err != nil {
		log.Fatal("Error while reading the file", err)
	}

	defer file.Close()
	reader := csv.NewReader(file)
	row, _ := reader.Read()

	for idx, val := range row {
		switch val {
		case "Issue_Age":
			age_col = idx
		case "Policy_Year":
			year_col = idx
		case "Rate":
			rate_col = idx
		case "Gender":
			gender_col = idx
		case "Risk_Class":
			class_col = idx
		}
	}

	for {
		row, err := reader.Read()
		if err == io.EOF {
			break
		}
		file_age, _ = strconv.Atoi(row[age_col])
		if file_age == issue_age && row[gender_col] == gender && row[class_col] == risk_class {
			file_rate, _ = strconv.ParseFloat(row[rate_col], 64)
			file_year, _ = strconv.Atoi(row[year_col])
			rates[file_year-1] = file_rate
		}
	}
	return rates
}

// get_corridor_factors returns corridor factors in an array where index 0 is for year 1
func get_corridor_factors(issue_age int) [120]float64 {
	rates := create_rate_array(1.0)
	var age_col, rate_col int

	file, err := os.Open("../python/corridor_factors.csv")
	if err != nil {
		log.Fatal("Error when opening file", err)
	}

	defer file.Close()
	reader := csv.NewReader(file)
	row, _ := reader.Read()
	for idx, val := range row {
		switch val {
		case "Attained_Age":
			age_col = idx
		case "Rate":
			rate_col = idx
		}
	}

	var file_age int
	var file_rate float64
	for {
		row, err = reader.Read()
		if err == io.EOF {
			break
		}
		file_age, _ = strconv.Atoi(row[age_col])
		if file_age >= issue_age {
			file_rate, _ = strconv.ParseFloat(row[rate_col], 64)
			rates[file_age-issue_age] = file_rate
		}
	}
	return rates
}

// get_rates returns a map of float64 arrays filled with rates for a policy projection.
// Map keys include "premium_load", "policy_fee", "per_unit", "cf", "naar_disc", "coi", and "interest". 
// Arrays stored in map are of length 120 and index 0 corresponds to policy year 1
func get_rates(gender string, risk_class string, issue_age int) map[string][120]float64 {
	var rates map[string][120]float64
	rates = make(map[string][120]float64)
	coi_rates := get_coi_rates(gender, risk_class, issue_age)
	per_unit_rates := get_per_unit_rates(issue_age)
	corridor_factors := get_corridor_factors(issue_age)
	premium_loads := create_rate_array(0.06)
	policy_fees := create_rate_array(120)
	naar_discount := create_rate_array(math.Pow(1.01, -1/12.0))
	interest_rates := create_rate_array(math.Pow(1.03, 1/12.0) - 1)

	rates["premium_load"] = premium_loads
	rates["policy_fee"] = policy_fees
	rates["per_unit"] = per_unit_rates
	rates["cf"] = corridor_factors
	rates["naar_disc"] = naar_discount
	rates["coi"] = coi_rates
	rates["interest"] = interest_rates
	
	return rates
}

// Illustration is a data container for fields calculated during an illustrate() process
// Go does not allow mixed type maps (closest equivalent to Python dictionary) so this was
// used instead
type Illustration struct {
	length uint16
	policyMonth []uint16
	policyYear []uint8
	monthInPolicyYear []uint8
	valueStart []float64
	premium []float64
	premiumLoad []float64
	expenseCharge []float64
	deathBenefit []float64
	naar []float64
	coiCharge []float64
	interest []float64
	valueEnd []float64
}

// NewIllustration constructs a Illustration object based on length parameter provided
func NewIllustration(length uint16) *Illustration {
	if length == 0 {
		fmt.Println("Invalid length, setting to 1")
		length = 1
	}
	return &Illustration{
		length: length,
		policyMonth: make([]uint16, length),
		policyYear: make([]uint8, length),
		monthInPolicyYear: make([]uint8, length),
		valueStart: make([]float64, length),
		premium: make([]float64, length),
		premiumLoad: make([]float64, length),
		expenseCharge: make([]float64, length),
		deathBenefit: make([]float64, length),
		naar: make([]float64, length),
		coiCharge: make([]float64, length),
		interest: make([]float64, length),
		valueEnd: make([]float64, length),
	}
}

// illustrate returns an Illustration object after projecting based on the provided parameters
func illustrate(rates map[string][120]float64, issue_age int, face_amount float64, annual_premium float64) Illustration {
	maturity_age := 121
	projection_length := 12 * (maturity_age - issue_age)
	illustration := *NewIllustration(uint16(projection_length))

	end_value := 0.0
	var policy_year uint8	
	policy_year = 0
	var start_value, premium, premium_load, expense_charge, av_for_db, db, naar, coi, av_for_interest, interest float64

	for i := 1; i <= projection_length; i++ {
		if (i % 12) == 1 {
			policy_year += 1
			premium = annual_premium
		} else {
			premium = 0.0
		}
		start_value = end_value
		premium_load = premium * rates["premium_load"][policy_year-1]
		expense_charge = (rates["policy_fee"][policy_year-1] + rates["per_unit"][policy_year-1]*face_amount/1000) / 12.0
		av_for_db = start_value + premium - premium_load - expense_charge
		db = max(face_amount, rates["cf"][policy_year-1]*av_for_db)
		naar = max(0, db*rates["naar_disc"][policy_year-1]-max(0, av_for_db))
		coi = (naar / 1000.0) * (rates["coi"][policy_year-1] / 12)
		av_for_interest = av_for_db - coi
		interest = max(0, av_for_interest) * rates["interest"][policy_year-1]
		end_value = av_for_interest + interest

		illustration.policyMonth[i-1] = uint16(i)
		illustration.policyYear[i-1] = policy_year
		illustration.monthInPolicyYear[i-1] = uint8(((i-1) % 12) + 1)
		illustration.valueStart[i-1] = start_value
		illustration.premium[i-1] = premium
		illustration.premiumLoad[i-1] = premium_load
		illustration.expenseCharge[i-1] = expense_charge
		illustration.deathBenefit[i-1] = db
		illustration.naar[i-1] = naar
		illustration.coiCharge[i-1] = coi
		illustration.interest[i-1] = interest
		illustration.valueEnd[i-1] = end_value
	}

	return illustration
}

// solve returns an Illustration object after determining minimum premium that allows corresponding call to 
// illustrate() to successfully reach maturity
func solve(rates map[string][120]float64, issue_age int, face_amount float64) Illustration {
	guess_lo := 0.0
	guess_hi := face_amount / 100.0

	var illustration Illustration
	for {
		illustration = illustrate(rates, issue_age, face_amount, guess_hi)
		if illustration.valueEnd[illustration.length-1] <= 0 {
			guess_lo = guess_hi
			guess_hi *= 2
		} else {
			break
		}
	}

	guess_md := 0.0
	for ; (guess_hi - guess_lo) > 0.005; {
		guess_md = (guess_lo + guess_hi) / 2.0
		illustration = illustrate(rates, issue_age, face_amount, guess_md)
		if illustration.valueEnd[illustration.length-1] <= 0 {
			guess_lo = guess_md
		} else {
			guess_hi = guess_md
		}
	}

	result := math.Round(guess_md * 100.0) / 100.0
	illustration = illustrate(rates, issue_age, face_amount, result)
	if illustration.valueEnd[illustration.length-1] <= 0 {
		result += 0.01
		illustration = illustrate(rates, issue_age, face_amount, result)
	}
	return illustration
}