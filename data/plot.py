import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the CSV data into a DataFrame
data = pd.read_csv("join_avg.csv")


# Create a plot
plt.figure(figsize=(10, 6))


# Plot 'get' data with error bars for standard deviation
plt.errorbar(
    data["nodes"],
    data["mean"],
    yerr=data["std"],
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
plt.xticks([10, 20, 30, 40, 50])
# Show the plot
plt.grid(True)
# plt.show()

plt.savefig("output_plot.png", dpi=300, bbox_inches='tight')  # saves as PNG with a DPI of 300
