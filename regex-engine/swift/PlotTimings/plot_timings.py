import argparse
from pathlib import Path

import pandas as pd
import matplotlib.pyplot as plt


parser = argparse.ArgumentParser(
    description="Plot VM timing results from a CSV file."
)

parser.add_argument(
    "csv_file",
    type=Path,
    help="Path to the timing CSV file",
)

args = parser.parse_args()

df = pd.read_csv(args.csv_file)

plt.figure(figsize=(12, 7))

plt.plot(
    df["n"],
    df["backtracking_ns"],
    marker="o",
    linewidth=2,
    label="Backtracking VM",
)

plt.plot(
    df["n"],
    df["thompson_ns"],
    marker="o",
    linewidth=2,
    label="Thompson VM",
)

plt.xlabel("n")
plt.ylabel("Execution time (ns)")
plt.title("Backtracking VM vs Thompson VM")
plt.xticks(df["n"])
plt.yscale("log")

plt.grid(True, which="both", alpha=0.3)
plt.legend()
plt.tight_layout()

output_file = args.csv_file.with_suffix(".png")
plt.savefig(output_file, dpi=200)
print(f"Saved plot to {output_file}")

plt.show()
