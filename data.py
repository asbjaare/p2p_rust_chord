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

# Write 'put' and 'get' data to separate CSV files
put_data.to_csv("put_data.csv", index=False)
get_data.to_csv("get_data.csv", index=False)

print("Put data saved to put_data.csv")
print("Get data saved to get_data.csv")
