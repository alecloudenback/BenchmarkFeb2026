import multiprocessing as mp

import functions as f


class TaskParameter:

    def __init__(self, gender: str, risk_class: str, issue_age: int, face_amount: int, premium: float):
        self.gender = gender
        self.risk_class = risk_class
        self.issue_age = issue_age
        self.face_amount = face_amount
        self.premium = premium


def illustrate(gender, risk_class, issue_age, face_amount, premium):
    """
    Wrapper function to easily distribute amongst helpers
    """
    rates = f.get_rates(gender, risk_class, issue_age)
    return f.illustrate(rates, issue_age, face_amount, premium)


def solve(gender, risk_class, issue_age, face_amount, premium):
    """
    Wrapper function to easily distribute amongst helpers
    """
    return f.solve_for_premium(gender, risk_class, issue_age, face_amount)


def worker(input: mp.Queue, output: mp.Queue):
    """
    Driver code for individual workers / processes
    """
    for func, taskParameter in iter(input.get, 'STOP'):
        result = func(
            taskParameter.gender,
            taskParameter.risk_class,
            taskParameter.issue_age,
            taskParameter.face_amount,
            taskParameter.premium)
        output.put(result)


def _multi(task, taskParameters: list[TaskParameter], num_processes: int):
    """
    Main multiprocessing execution

    Parameters
    ----------
    task: function
        Actuarial process to run, i.e. "illustrate" or "solve"
    taskParameters: list[TaskParameter]
        List of TaskParameter objects to run
    num_processes: int
        Number of parallel processes to run
    """
    mp.freeze_support()

    # create queues
    task_queue = mp.Queue()
    done_queue = mp.Queue()

    # submit tasks
    for taskParameter in taskParameters:
        task_queue.put([task, taskParameter])

    # start workers
    processes = [mp.Process(target=worker, args=(
        task_queue, done_queue)) for _ in range(num_processes)]
    for process in processes:
        process.start()

    # send stop signals
    for _ in range(num_processes):
        task_queue.put('STOP')

    # get results
    for _ in taskParameters:
        result = done_queue.get()
        # print(result)

    # rejoin processes
    for process in processes:
        process.join()


def multi_illustrate(taskParameters: list[TaskParameter], num_processes: int):
    """
    Convenience wrapper to perform multiprocessing of illustrate routine
    """
    _multi(illustrate, taskParameters, num_processes)


def multi_solve(taskParameters: list[TaskParameter], num_processes: int):
    """
    Convenience wrapper to perform multiprocessing of solve routine
    """
    _multi(solve, taskParameters, num_processes)


def generate_sample_taskparams(n: int) -> list[TaskParameter]:
    return [TaskParameter("M", "NS", 35, 100000, 1255.03) for _ in range(n)]


if __name__ == '__main__':
    """
    Example execution

    - Create list of parameters for tasks
    - Run tasks
        - Initialize queues
        - Submit tasks
        - Spin up workers
        - Send "STOP" signals
        - Collect results
        - Tear down workers
    """

    t = 1000
    taskParameters = generate_sample_taskparams(t)

    p = 4
    multi_illustrate(taskParameters, p)
