include("functions.jl")
include("parallel.jl")

using BenchmarkTools

const N = 1000
const W = Threads.nthreads()

issue_age = 35
face_amount = 100_000.0
premium = 1255.03
coi_table = get_coi(Male, NS, issue_age)
unit_table = get_unit_load(issue_age)
proj_len = 12 * (121 - issue_age)
illus = Illustration(proj_len)

println("Threads: $W")
println()

println("1) $N illustrations, 1 worker (sequential)")
@btime for _ in 1:$N
    illustrate!($illus, $coi_table, $unit_table, $issue_age, $face_amount, $premium)
end

println("2) $N illustrations, $W workers (parallel)")
tasks = generate_sample_tasks(N)
@btime parallel_illustrate($tasks, $W)

println("3) $N solves, 1 worker (sequential)")
@btime for _ in 1:$N
    solve_for_premium!($illus, $coi_table, $unit_table, $issue_age, $face_amount)
end

println("4) $N solves, $W workers (parallel)")
@btime parallel_solve($tasks, $W)
