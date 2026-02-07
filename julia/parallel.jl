using OhMyThreads: @tasks, @set, @local

struct TaskParameter
    gender::Gender
    risk_class::RiskClass
    issue_age::Int
    face_amount::Float64
    premium::Float64
end

function generate_sample_tasks(n::Int)
    return [TaskParameter(Male, NS, 35, 100_000.0, 1255.03) for _ in 1:n]
end

# ── OhMyThreads parallelism ─────────────────────────────────────

function parallel_illustrate(tasks::Vector{TaskParameter}, num_workers::Int)
    @tasks for i in eachindex(tasks)
        @set ntasks = num_workers
        @local illus = Illustration(12 * (121 - tasks[1].issue_age))
        task = tasks[i]
        coi_table = get_coi(task.gender, task.risk_class, task.issue_age)
        unit_table = get_unit_load(task.issue_age)
        illustrate!(illus, coi_table, unit_table, task.issue_age, task.face_amount, task.premium)
    end
end

function parallel_solve(tasks::Vector{TaskParameter}, num_workers::Int)
    @tasks for i in eachindex(tasks)
        @set ntasks = num_workers
        @set reducer = (_, b) -> b
        @local illus = Illustration(12 * (121 - tasks[1].issue_age))
        task = tasks[i]
        coi_table = get_coi(task.gender, task.risk_class, task.issue_age)
        unit_table = get_unit_load(task.issue_age)
        solve_for_premium!(illus, coi_table, unit_table, task.issue_age, task.face_amount)
    end
end
