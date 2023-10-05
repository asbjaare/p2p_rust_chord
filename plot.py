import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the CSV data into a DataFrame
data = pd.read_csv("results.csv")

# Group the data by 'nodes' and 'signal' and calculate the mean and standard deviation
grouped = data.groupby(["nodes", "signal"])["rps"].agg([np.mean, np.std]).reset_index()

# Separate 'put' and 'get' data
put_data = grouped[grouped["signal"] == "put"]
get_data = grouped[grouped["signal"] == "get"]

# Create a plot
plt.figure(figsize=(10, 6))

# Plot 'put' data with error bars for standard deviation
plt.errorbar(
    put_data["nodes"], put_data["mean"], yerr=put_data["std"], label="Put", marker="o"
)

# Plot 'get' data with error bars for standard deviation
plt.errorbar(
    get_data["nodes"], get_data["mean"], yerr=get_data["std"], label="Get", marker="o"
)

# Add labels and a legend
plt.xlabel("Nodes")
plt.ylabel("Average RPS")
plt.title("Average RPS vs. Nodes (with Standard Deviation)")
plt.legend()

# Show the plot
plt.grid(True)
plt.show()
