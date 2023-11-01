import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the CSV data into a DataFrame
data = pd.read_csv("leave.csv")

# Group the data by 'nodes' and 'signal' and calculate the mean and standard deviation
grouped = data.groupby("nodes")["Time"].agg(["mean", "std"]).reset_index()


# Write 'put' and 'get' data to separate CSV files
grouped.to_csv("leave_avg.csv", index=False)
print(grouped.head())
