include("functions.jl")
include("parallel.jl")

const N = 1000
const W = Threads.nthreads()

function result_printer(elapsed_ns::UInt64, num_tasks::Int, last_premium::Float64)
    elapsed_ms = elapsed_ns / 1_000_000
    per_task_ms = elapsed_ms / num_tasks
    println("Results -------------------")
    println("Last premium: ", last_premium)
    println("Total time: ", round(elapsed_ms; digits=3), "ms")
    println("Tasks: ", num_tasks)
    println("Per task: ", round(per_task_ms; digits=4), "ms")
    println("---------------------------")
    println()
end

function benchmark()
    issue_age = 35
    gender = Male
    risk_class = NS
    face_amount = 100_000.0
    premium = 1255.03

    # Look up the tables once — they're just references into the cache
    coi_table = get_coi(gender, risk_class, issue_age)
    unit_table = get_unit_load(issue_age)

    # 1. N illustrations, 1 worker (sequential)
    println("Starting $N illustrations with 1 worker...")
    proj_len = 12 * (121 - issue_age)
    illus = Illustration(proj_len)
    t = time_ns()
    for _ in 1:N
        illustrate!(illus, coi_table, unit_table, issue_age, face_amount, premium)
    end
    elapsed = time_ns() - t
    result_printer(elapsed, N, premium)

    # 2. N illustrations, W workers (parallel)
    println("Starting $N illustrations with $W workers...")
    tasks = generate_sample_tasks(N)
    t = time_ns()
    parallel_illustrate(tasks, W)
    elapsed = time_ns() - t
    result_printer(elapsed, N, premium)

    # 3. N premium solves, 1 worker (sequential)
    println("Starting $N solves with 1 worker...")
    t = time_ns()
    local solved_premium
    for _ in 1:N
        solved_premium = solve_for_premium!(illus, coi_table, unit_table, issue_age, face_amount)
    end
    elapsed = time_ns() - t
    result_printer(elapsed, N, solved_premium)

    # 4. N premium solves, W workers (parallel)
    println("Starting $N solves with $W workers...")
    tasks = generate_sample_tasks(N)
    t = time_ns()
    solved_premium = parallel_solve(tasks, W)
    elapsed = time_ns() - t
    result_printer(elapsed, N, solved_premium)

    # JuliaActuary integration demo: PV and IRR
    println("JuliaActuary Integration ---")
    solve_for_premium!(illus, coi_table, unit_table, issue_age, face_amount)
    pv, irr_rate = compute_pv_and_irr(illus)
    println("PV of terminal death benefit: ", round(pv; digits=2))
    println("IRR (policyholder perspective): ", irr_rate)
    println("IRR annualized: ", round(rate(irr_rate) * 100; digits=4), "%")
    println("----------------------------")
end

benchmark()
