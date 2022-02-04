#!/usr/bin/python3

from cycler import cycler
import argparse
import sys
import os
import seaborn as sns
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

def create_arg_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('file')
    parser.add_argument('--save', action='store_true')

    return parser



def legend(name, param):
    if "Im" in name:
        return f"Im et al. (ε = {param})"
    elif "PRR" in name:
        return f"PTS (λ = {param})"
    elif "Two" in name:
        return f"Two-Stage (λ = {param})"
    else:
        return "Round-Robin"

def plot(filename, save):
    if "exp1" in filename:
        x_name = "sigma"
    else:
        x_name = "round"

    sns.set_theme(style='ticks')

    df = pd.read_csv(filename)
    df = df.round(3)
    df['cr'] = df['alg'] / df['opt']
    df['param'] = df[['name','param']].apply(lambda x: legend(*x),axis=1)

    ax = sns.lineplot(data=df, x=x_name, y="cr", hue='param', style='param', markers=('round' in list(df)), linewidth=2.5, markersize=8)
    
    if x_name == 'round':
        ax.xaxis.set_major_locator(MaxNLocator(integer=True))
        plt.legend(labels=df['param'].unique(),ncol=2, loc="right", bbox_to_anchor=(1.0,0.63))
        plt.xlabel("Round")
    else:
        plt.legend(labels=df['param'].unique(),ncol=2, loc="upper left")
        ax.set(xscale='symlog')
        plt.ylim(top=4.5)
        plt.xlabel("Noise parameter ω")

    plt.ylabel('Empirical competitive ratio')
    plt.tight_layout()

    fig = plt.gcf()
    fig.set_dpi(600)
    fig.set_size_inches(7,4)

    if save:
        f = filename.split(".")[0]
        plt.savefig(f"{f}.pdf")
    else:
        plt.show()



if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.file):

        plot(parsed_args.file, parsed_args.save)
    else:
        print("Path not valid!")
