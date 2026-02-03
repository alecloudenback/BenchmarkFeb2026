/*
Example Rust implementation similar to Python & Go

Run from command line in parent directory (the one above src which contains this file)
Build: cargo build .
Build with optimizations: cargo build --release .
Run and build if needed: cargo run .
Run and build if needed with optimization: cargo run --release .

*/

use std::{
    error::Error, fs::File, collections::HashMap
};

// serde is for serialization and deserialization of data
// using here to simplify reading csv files
use serde::Deserialize;

// need to prepare to deserialize data to structure
#[derive(Deserialize)]

/// An issue age by policy year record is represented here
struct IAPYRecord {
    // unfortunately there is not a shortcut here given the nameing used in the csvs
    // there is a quick way to indicate the fields are lowercase, UPPERCASE, PascalCase, 
    // camelCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case, and 
    // SCREAMING-KEBAB-CASE
    #[serde(alias="Issue_Age")]
    issue_age: u8,
    #[serde(alias="Policy_Year")]
    policy_year: u8,
    #[serde(alias="Rate")]
    rate: f64,
}

#[derive(Deserialize)]

/// A gender, risk class, issue age, policy year record is represented here
struct GenRCIAPYRecord {
    #[serde(alias="Gender")]
    gender: String,
    #[serde(alias="Risk_Class")]
    risk_class: String,
    #[serde(alias="Issue_Age")]
    issue_age: u8,
    #[serde(alias="Policy_Year")]
    policy_year: u8,
    #[serde(alias="Rate")]
    rate: f64,
}

#[derive(Deserialize)]
/// An attained age record is represented here
struct AARecord {
    #[serde(alias="Attained_Age")]
    attained_age: u8,
    #[serde(alias="Rate")]
    rate: f64,
}

fn read_ia_py_csv(path: &str, default: f64, issue_age: u8, ) -> Result<[f64;121], Box<dyn Error>> {
    // Reads a rate file csv that varies by issue age and policy year
    //
    // Returns an array of length 121 where index 0 is policy year 1

    // create default array
    let mut rates: [f64; 121] = [default; 121];

    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.deserialize() {
        let record: IAPYRecord = result?;
        if record.issue_age == issue_age{
            rates[(record.policy_year - 1) as usize] = record.rate
        }
    }
    return Ok(rates);
}

fn read_gen_rc_ia_py_csv(path: &str, default: f64, gender: &str, risk_class: &str, issue_age: u8) -> Result<[f64;121], Box<dyn Error>> {
    // Reads a rate file csv that varies by gender, risk class, issue age, and policy year
    //
    // Returns an array of length 121 where index 0 is policy year 1
    let mut rates: [f64;121] = [default;121];

    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.deserialize() {
       let record: GenRCIAPYRecord = result?;
       if record.gender == gender && record.risk_class == risk_class && record.issue_age == issue_age {
           rates[(record.policy_year - 1) as usize] = record.rate
       }
    }
    return Ok(rates);
}

fn read_aa_csv(path: &str, default: f64, issue_age: u8) -> Result<[f64;121], Box<dyn Error>> {
    // Reads a rate file csv that varies by attained age
    //
    // Returns an array of length 121 where index i corresponds to issue_age + i
    let mut rates: [f64;121] = [default;121];

    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    for result in rdr.deserialize() {
       let record: AARecord = result?;
       if record.attained_age >= issue_age {
           rates[(record.attained_age - issue_age) as usize] = record.rate;
       }
    }
    return Ok(rates);
}

pub fn get_rates(gender: &str, risk_class: &str, issue_age: u8) -> Result<HashMap<&'static str, [f64;121]>, Box<dyn Error>> {
    let mut rates: HashMap<&'static str, [f64;121]> = HashMap::new();
    rates.insert("premium_loads", [0.06; 121]);
    rates.insert("policy_fees", [120.0;121]);
    rates.insert("unit_loads", read_ia_py_csv("../data/unit_load.csv", 0.0, issue_age)?);
    rates.insert("corr_facts", read_aa_csv("../data/corridor_factors.csv", 1.0, issue_age)?);
    rates.insert("naar_discs", [f64::powf(1.01, -1.0/12.0);121]);
    rates.insert("coi_rates", read_gen_rc_ia_py_csv("../data/coi.csv", 0.0, gender, risk_class, issue_age)?);
    rates.insert("interest_rates", [f64::powf(1.03,1.0/12.0)-1.0;121]);
    return Ok(rates);
}

// Representation of an illustration object that has a specified length and data 'columns'
pub struct Illustration {
    pub length: usize,
    pub policy_month: Vec<u16>,
	pub policy_year: Vec<u8>,
	pub month_in_policy_year: Vec<u8>,
	pub value_start: Vec<f64>,
	pub premium: Vec<f64>,
	pub premium_load: Vec<f64>,
	pub expense_charge: Vec<f64>,
	pub death_benefit: Vec<f64>,
	pub naar: Vec<f64>,
	pub coi_charge: Vec<f64>,
	pub interest: Vec<f64>,
	pub value_end: Vec<f64>,
}


pub fn new_illustration(length: usize) -> Illustration {
    // Creates a new Illustration of length length
    Illustration {
        length: length,
        policy_month: vec![0; length],
        policy_year: vec![0; length],
        month_in_policy_year: vec![0; length],
        value_start: vec![0.0; length],
        premium: vec![0.0; length],
        premium_load: vec![0.0; length],
        expense_charge: vec![0.0; length],
        death_benefit: vec![0.0; length],
        naar: vec![0.0; length],
        coi_charge: vec![0.0; length],
        interest: vec![0.0; length],
        value_end: vec![0.0; length],
    }
}

pub fn at_issue_projection(rates: &HashMap<&'static str, [f64;121]>, issue_age: u8, face_amount: f64, annual_premium: f64) -> Result<Illustration, Box<dyn Error>> {
    // 
    let maturity_age: u8 = 121;
    let projection_years: usize = (maturity_age - issue_age) as usize;    
    let length: usize = 12 * projection_years;
    let mut end_value = 0.0;
    let mut policy_year = 0;

    let mut ill = new_illustration(length);

    // Pre-fetch rate arrays to avoid repeated HashMap lookups
    let premium_loads = &rates["premium_loads"];
    let policy_fees = &rates["policy_fees"];
    let unit_loads = &rates["unit_loads"];
    let corr_facts = &rates["corr_facts"];
    let naar_discs = &rates["naar_discs"];
    let coi_rates = &rates["coi_rates"];
    let interest_rates = &rates["interest_rates"];

    for i in 0..length {
        if i % 12 == 0 {
            policy_year += 1;
        }
        let py_idx = policy_year - 1;
        let start_value = end_value;
        let premium = if (i % 12) == 0 {annual_premium} else {0.0};
        let premium_load = premium * premium_loads[py_idx];
        let expense_charge = (policy_fees[py_idx] + unit_loads[py_idx] * face_amount / 1000.0) / 12.0;
        let av_for_db = start_value + premium - premium_load - expense_charge;
        let db = face_amount.max(corr_facts[py_idx] * av_for_db);
        let naar = (db * naar_discs[py_idx] - av_for_db.max(0.0)).max(0.0);
        let coi = (naar / 1000.0) * (coi_rates[py_idx] / 12.0);
        let av_for_interest = av_for_db - coi;
        let interest = (av_for_interest * interest_rates[policy_year - 1]).max(0.0);
        end_value = av_for_interest + interest;

        // store data
        
        ill.policy_month[i] = (i+1) as u16;
        ill.policy_year[i] = policy_year as u8;
        ill.month_in_policy_year[i] = (((i+11) % 12) + 1) as u8;
        ill.value_start[i] = start_value;
        ill.premium[i] = premium;
        ill.premium_load[i] = premium_load;
        ill.expense_charge[i] = expense_charge;
        ill.death_benefit[i] = db;
        ill.naar[i] = naar;
        ill.coi_charge[i] = coi;
        ill.interest[i] = interest;
        ill.value_end[i] = end_value;

    }
    
    return Ok(ill);
}

pub fn solve_for_premium(rates: &HashMap<&'static str, [f64;121]>, issue_age: u8, face_amount: f64) -> Result<Illustration, Box<dyn Error>> {

    let mut guess_lo = 0.0;
    let mut guess_hi = face_amount / 100.0;
    let mut guess_md = 0.0;
    let mut ill: Illustration;

    // get rates
    loop {
        ill = at_issue_projection(rates, issue_age, face_amount, guess_hi)?;
        if ill.value_end[ill.length-1] <= 0.0 {
            guess_lo = guess_hi;
            guess_hi *= 2.0;
        } else {
            break;
        }
    }

    while (guess_hi - guess_lo) > 0.005 {
        guess_md = (guess_lo + guess_hi) / 2.0;
        ill = at_issue_projection(rates, issue_age, face_amount, guess_md)?;
        if ill.value_end[ill.length-1] <= 0.0 {
            guess_lo = guess_md;
        } else {
            guess_hi = guess_md;
        }
    }

    let mut result = (guess_md * 100.0).round() / 100.0;
    ill = at_issue_projection(rates, issue_age, face_amount, result)?;
    if ill.value_end[ill.length-1] <= 0.0 {
        result += 0.01;
        ill = at_issue_projection(rates, issue_age, face_amount, result)?;
    }

    return Ok(ill);
}