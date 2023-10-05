import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the CSV data into a DataFrame
data = pd.read_csv("results.csv")

# Group the data by 'nodes' and 'signal' and calculate the mean and standard deviation
grouped = data.groupby(["nodes", "signal"])["rps"].agg(["mean", "std"]).reset_index()

# Separate 'put' and 'get' data
put_data = grouped[grouped["signal"] == "put"]
get_data = grouped[grouped["signal"] == "get"]
print(put_data["std"].to_list())
# Create a plot
plt.figure(figsize=(10, 6))

# Plot 'put' data with error bars for standard deviation
plt.errorbar(
    put_data["nodes"],
    put_data["mean"],
    yerr=put_data["std"],
    label="Put",
    marker="o",
    elinewidth=15,
    capsize=10,
    capthick=5,
    linewidth=5,
)

# Plot 'get' data with error bars for standard deviation
plt.errorbar(
    get_data["nodes"],
    get_data["mean"],
    yerr=get_data["std"],
    label="Get",
    marker="o",
    elinewidth=15,
    capsize=10,
    capthick=5,
    linewidth=5,
)

# Add labels and a legend
plt.xlabel("Nodes")
plt.ylabel("Average RPS")
plt.title("Average RPS vs. Nodes (with Standard Deviation)")
plt.legend()
plt.xticks([1, 2, 4, 8, 16])
# Show the plot
plt.grid(True)
# plt.show()
