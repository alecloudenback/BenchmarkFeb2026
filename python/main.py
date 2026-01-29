import timeit

import functions as functions
import parallel as parallel

N = 1000    # number of cases
W = 8       # number of workers


def _run(code, setup, number, number_display):
    """
    Docstring for _run

    code: str
        String of code to pass to timeit for benchmarking
    setup: str
        String of setup instructions (e.g., imports, parameter generation) excluded from benchmarking
    number: int
        Number of iterations for timeit (parallel processes should be 1)
    number_display: int
        Number of iterations to display (single and parallel should be number of cases)
    """
    t = timeit.Timer(code, setup=setup)
    print(f"{number_display} calls:", t.timeit(number=number))

    many = t.repeat(number=number)
    print(f"\n{number_display} calls, 5 repeats:\n",
          many, "\nAvg:", sum(many)/len(many))


def benchmark():
    """
    Runs 4 different setups for comparison 
    1. N illustrations with 1 worker
    2. N illustrations with W workers
    3. N premium solves with 1 worker
    4. N premium solves with W workers

    N and W are global constants
    """
    print("ILLUSTRATIONS: SINGLE PROCESS--------------")
    # run N illustrations with 1 worker
    setup = "import functions as functions"
    code = "rates = functions.get_rates('M', 'NS', 35)"
    code += "\nfunctions.illustrate(rates, 35, 100000, 1255.03)"
    _run(code, setup, N, N)

    print("\nILLUSTRATIONS: MULTI PROCESS---------------")
    # run N illustrations with W workers
    setup = "import parallel"
    code = f"params = parallel.generate_sample_taskparams({N})"
    code += f"\nparallel.multi_illustrate(params,{W})"
    _run(code, setup, 1, N)

    print("\nSOLVES: SINGLE PROCESS---------------------")
    # run N solves with 1 worker
    setup = "import functions as functions"
    code = "functions.solve_for_premium('M', 'NS', 35, 100000)"
    _run(code, setup, N, N)

    print("\nSOLVES: MULTI PROCESS----------------------")
    # run N solves with W workers
    setup = "import parallel"
    code = f"params = parallel.generate_sample_taskparams({N})"
    code += f"\nparallel.multi_solve(params,{W})"
    _run(code, setup, 1, N)


if __name__ == '__main__':
    benchmark()
