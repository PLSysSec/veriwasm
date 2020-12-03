import json
import sys
import math
import matplotlib
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MultipleLocator
#matplotlib.use('tkagg')

def median(lst):
    n = len(lst)
    s = sorted(lst)
    return (sum(s[n//2-1:n//2+1])/2.0, s[n//2])[n % 2] if n else None

def graph_blocks_vs_time(dataset):
    for data in dataset.values():
        times = [x[1] for x in data]
        block_counts = [x[2] for x in data]
        plt.plot(block_counts, times, 'o')
    plt.show()
    #raise NotImplementedError

def graph_funcs_vs_time(dataset):
    func_count = []
    total_times = []
    for data in dataset.values():
        func_count.append(len(data))
        total_time = sum([x[1] for x in data])
        total_times.append(total_time)
        plt.plot(func_count, total_times, 'o')
    plt.show()

def get_data(filenames):
    dataset = {}
    for filename in filenames:
        with open(filename) as f:
            data = json.load(f)
        dataset[filename] = data
    return dataset

def get_aggregate_data(dataset):
    aggregate_data = []
    #print(len(dataset.keys()))
    for name,data in dataset.items():
        name = name.split('/')[-1].split(".")[0]
        times = [x[2] + x[3] + x[4] + x[5] for x in data]
        average_t = sum(times) / len(times)
        median_t = median(times)
        max_t = max(times)
        min_t = min(times)
        total_t = sum(times)
        num_funcs = len(times)
        N = len(times) // 100
        print("top 1% = ", N, " functions out of", len(times))
        top_n = sorted(times, reverse = True)[:N] 
        top_percent = sum(top_n) / total_t
        cfg_percent = sum([x[2] for x in data]) / total_t
        stack_percent = sum([x[3] for x in data]) / total_t
        heap_percent = sum([x[4] for x in data]) / total_t
        call_percent = sum([x[5] for x in data]) / total_t
        print(top_n, top_percent)
        aggregate_data.append( (name,average_t,median_t,max_t,min_t,num_funcs,total_t,top_percent, cfg_percent, stack_percent, heap_percent, call_percent))
    return aggregate_data
    

def generate_summary_table(aggregate_data):
    names_row = " &"
    average_row = "Average Function Validation Time (s) & "
    median_row = "Median Function Validation Time (s) & "
    max_row = "Max Function Validation Time (s) & "
    min_row = "Min Function Validation Time (s) & "
    num_funcs_row = "\\# Functions in Module & "
    total_row = "Total Validation Time (s) & "
    #for name,average_t,median_t,max_t,min_t in aggregate_data:
    names_row +=     " & ".join([str(d[0]) for d in aggregate_data]) + "\\\\"
    average_row +=   " & ".join([str(round(d[1],2)) for d in aggregate_data]) + "\\\\"
    median_row +=    " & ".join([str(round(d[2],2)) for d in aggregate_data]) + "\\\\"
    max_row +=       " & ".join([str(round(d[3],2)) for d in aggregate_data]) + "\\\\"
    min_row +=       " & ".join([str(round(d[4],2)) for d in aggregate_data]) + "\\\\"
    num_funcs_row += " & ".join([str(round(d[5],2)) for d in aggregate_data]) + "\\\\"
    total_row +=     " & ".join([str(round(d[6],2)) for d in aggregate_data]) + "\\\\" 
    table_str = "\n".join([names_row, average_row, median_row, max_row, min_row, num_funcs_row, total_row]) + "\n"
    return table_str

#print out some quick statistics
def summarise_data(aggregate_data):
    medians = [round(d[2],2) for d in aggregate_data]
    maxes = [round(d[3],2) for d in aggregate_data]
    num_funcs = [round(d[5],2) for d in aggregate_data] 
    times = [round(d[6],2) for d in aggregate_data] 
    one_percent = [d[7] for d in aggregate_data]
    cfg_percent = [d[8] for d in aggregate_data]
    stack_percent = [d[9] for d in aggregate_data]
    heap_percent = [d[10] for d in aggregate_data]
    call_percent = [d[11] for d in aggregate_data]
    #print(averages)
    #medians = [round(d[2],2) for d in aggregate_data]
    #print(medians)
    print(f"Number of binaries = {len(times)}")
    print(f"Median function validation time: {median(medians)}")
    num_above_min = len([time for time in maxes if time > 60.0])
    print(f"Number of binariess with a function that took > 1 minute to validate: {num_above_min}")
    print(f"Top 1% of functions account for (on average) {sum(one_percent) / len(one_percent) * 100}% of total execution time")
    print(f"{sum(cfg_percent) / len(one_percent) * 100}% of verification time spent making CFGs")
    print(f"{sum(stack_percent) / len(stack_percent) * 100}% of verification time spent checking stack")
    print(f"{sum(heap_percent) / len(heap_percent) * 100}% of verification time spent checking heap")
    print(f"{sum(call_percent) / len(call_percent) * 100}% of verification time spent checking calls")

    print(f"Average Time = {sum(times) / len(times)}")
    #print(f"Average Max function Time = {sum(maxes) / len(maxes)}")
    print(f"Min Validation Time: {min(times)}")
    print(f"Max Validation Time: {max(times)}")
    print(f"Median Validation Time = {median(times)}")
    print(f"Min Functions: {min(num_funcs)}")
    print(f"Max Functions: {max(num_funcs)}")
    print(f"Median Functions: {median(num_funcs)}")
    fig, ax = plt.subplots()
    ax.xaxis.set_minor_locator(MultipleLocator(5))
    plt.xlabel('Module Validation Time (s)')
    plt.ylabel('# of Modules')  
    plt.hist(times, bins= math.ceil((max(times) - min(times))/5) )
    print("Histogram Created")
    plt.savefig('fastly_times.pdf')  
    print("Histogram Saved")

def run(filenames):
    dataset = get_data(filenames)
    #graph_blocks_vs_time(dataset)
    #graph_funcs_vs_time(dataset)
    aggregate_data = get_aggregate_data(dataset)
    summarise_data(aggregate_data)
    table = generate_summary_table(aggregate_data)
    print(table)

def main():
    filename = sys.argv[1]
    print(sys.argv)
    filenames = sys.argv[1:]
    run(filenames)

if __name__ == "__main__":
    main()

