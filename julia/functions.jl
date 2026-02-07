using CSV
using MortalityTables
using FinanceModels
using ActuaryUtilities

@enum Gender Male = 1 Female = 2
@enum RiskClass NS = 1 SM = 2

# ── Illustration: only store what the projection computes ─────────

mutable struct Illustration
    length::Int
    value_start::Vector{Float64}
    premium::Vector{Float64}
    premium_load::Vector{Float64}
    expense_charge::Vector{Float64}
    death_benefit::Vector{Float64}
    naar::Vector{Float64}
    coi_charge::Vector{Float64}
    interest::Vector{Float64}
    value_end::Vector{Float64}
end

function Illustration(n::Int)
    Illustration(n,
        Vector{Float64}(undef, n), Vector{Float64}(undef, n),
        Vector{Float64}(undef, n), Vector{Float64}(undef, n),
        Vector{Float64}(undef, n), Vector{Float64}(undef, n),
        Vector{Float64}(undef, n), Vector{Float64}(undef, n),
        Vector{Float64}(undef, n))
end

# policy_month, policy_year, month_in_policy_year are pure functions of i —
# compute on demand instead of storing 3 vectors
policy_month(i) = i
policy_year(i) = (i - 1) ÷ 12 + 1
month_in_year(i) = (i - 1) % 12 + 1

# ── CSV data loaded once into MortalityTables types ────────────────

const DATA_DIR = joinpath(@__DIR__, "..", "data")

const _corridor_table = let
    v = ones(Float64, 122)
    for row in CSV.File(joinpath(DATA_DIR, "corridor_factors.csv"))
        v[row.Attained_Age + 1] = row.Rate
    end
    UltimateMortality(v, start_age=0)
end
const _corridor_omega = omega(_corridor_table)

const _unit_load_tables = let
    raw = Dict{Int, Vector{Float64}}()
    for row in CSV.File(joinpath(DATA_DIR, "unit_load.csv"))
        v = get!(raw, row.Issue_Age) do; zeros(120) end
        v[row.Policy_Year] = row.Rate
    end
    # Vector indexed by issue_age + 1 (ages 0–120)
    dummy = UltimateMortality(zeros(120), start_age=1)
    tables = fill(dummy, 121)
    for (age, v) in raw
        tables[age + 1] = UltimateMortality(v, start_age=1)
    end
    tables
end

const _coi_tables = let
    raw = Dict{Tuple{Int,Int,Int}, Vector{Float64}}()
    for row in CSV.File(joinpath(DATA_DIR, "coi.csv"))
        g = row.Gender == "M" ? 1 : 2
        rc = row.Risk_Class == "NS" ? 1 : 2
        key = (g, rc, Int(row.Issue_Age))
        v = get!(raw, key) do; zeros(120) end
        v[row.Policy_Year] = row.Rate
    end
    # 3D array indexed by [gender, risk_class, issue_age + 1]
    dummy = UltimateMortality(zeros(120), start_age=1)
    tables = fill(dummy, 2, 2, 121)
    for ((g, rc, age), v) in raw
        tables[g, rc, age + 1] = UltimateMortality(v, start_age=1)
    end
    tables
end

# ── FinanceModels yield curves ─────────────────────────────────────

const _credited_curve = Yield.Constant(0.03)
const _monthly_interest = accumulation(_credited_curve, 0, 1 // 12) - 1
const _monthly_naar_disc = discount(Yield.Constant(0.01), 0, 1 // 12)

const PREMIUM_LOAD = 0.06
const POLICY_FEE = 120.0

# ── Table lookups ─────────────────────────────────────────────────

get_coi(gender::Gender, risk_class::RiskClass, issue_age::Int) =
    @inbounds _coi_tables[Int(gender), Int(risk_class), issue_age + 1]

get_unit_load(issue_age::Int) = @inbounds _unit_load_tables[issue_age + 1]

# ── Illustrate ────────────────────────────────────────────────────

function illustrate!(illus, coi_table, unit_table, issue_age, face_amount, annual_premium)
    projection_length = 12 * (121 - issue_age)
    illus.length = projection_length

    end_value = 0.0
    py = 0

    @inbounds for i in 1:projection_length
        if (i - 1) % 12 == 0
            py += 1
            premium = annual_premium
        else
            premium = 0.0
        end

        start_value = end_value
        pl = premium * PREMIUM_LOAD
        ec = (POLICY_FEE + unit_table[py] * face_amount / 1000.0) / 12.0
        av_for_db = start_value + premium - pl - ec
        att = issue_age + py - 1
        cf = att <= _corridor_omega ? _corridor_table[att] : 1.0
        db = max(face_amount, cf * av_for_db)
        naar = max(0.0, db * _monthly_naar_disc - max(0.0, av_for_db))
        coi = (naar / 1000.0) * (coi_table[py] / 12.0)
        av_for_interest = av_for_db - coi
        interest = max(0.0, av_for_interest) * _monthly_interest
        end_value = av_for_interest + interest

        illus.value_start[i] = start_value
        illus.premium[i] = premium
        illus.premium_load[i] = pl
        illus.expense_charge[i] = ec
        illus.death_benefit[i] = db
        illus.naar[i] = naar
        illus.coi_charge[i] = coi
        illus.interest[i] = interest
        illus.value_end[i] = end_value

        if end_value < 0.0
            illus.length = i
            return illus
        end
    end

    return illus
end

# ── Solve ─────────────────────────────────────────────────────────

function solve_for_premium!(illus, coi_table, unit_table, issue_age, face_amount)
    guess_lo = 0.0
    guess_hi = face_amount / 100.0

    while true
        illustrate!(illus, coi_table, unit_table, issue_age, face_amount, guess_hi)
        if illus.value_end[illus.length] <= 0.0
            guess_lo = guess_hi
            guess_hi *= 2.0
        else
            break
        end
    end

    guess_md = 0.0
    while (guess_hi - guess_lo) > 0.005
        guess_md = (guess_lo + guess_hi) / 2.0
        illustrate!(illus, coi_table, unit_table, issue_age, face_amount, guess_md)
        if illus.value_end[illus.length] <= 0.0
            guess_lo = guess_md
        else
            guess_hi = guess_md
        end
    end

    result = round(guess_md; digits=2)
    illustrate!(illus, coi_table, unit_table, issue_age, face_amount, result)
    if illus.value_end[illus.length] <= 0.0
        result += 0.01
        illustrate!(illus, coi_table, unit_table, issue_age, face_amount, result)
    end

    return result
end

# ── PV and IRR using ActuaryUtilities ──────────────────────────────

function compute_pv_and_irr(illus::Illustration)
    n = illus.length

    # PV of terminal death benefit is just one discounted cash flow
    pv = illus.death_benefit[n] * discount(_credited_curve, 0, n / 12)

    # IRR: premiums are annual (every 12 months) + death benefit at end
    num_years = (n - 1) ÷ 12 + 1
    # worst case: num_years premiums + 1 death benefit (last premium and DB may share a slot)
    cfs = Vector{Float64}(undef, num_years + 1)
    times = Vector{Float64}(undef, num_years + 1)
    k = 0
    @inbounds for i in 1:n
        cf = -illus.premium[i] + (i == n ? illus.death_benefit[n] : 0.0)
        if cf != 0.0
            k += 1
            cfs[k] = cf
            times[k] = i / 12.0
        end
    end

    irr_rate = irr(view(cfs, 1:k), view(times, 1:k))
    return (pv, irr_rate)
end
