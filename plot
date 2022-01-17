#!/usr/bin/python3

from cycler import cycler
import argparse
import sys
import os
import seaborn as sns
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

def create_arg_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('file')
    parser.add_argument('-l', '--lambdas', nargs="+", default=[0.0, 0.25, 0.5, 0.75, 1.0])
    parser.add_argument('--max', action='store_true')

    return parser


def get_data(filename):
    data = pd.read_csv(filename)
    data = data.round(3)
    data['cr'] = data['alg'] / data['opt']
    return data


colors = ['tab:blue', 'tab:orange', 'tab:green', 'tab:red', 'tab:purple', 'tab:brown','tab:blue', 'tab:orange', 'tab:green', 'tab:red', 'tab:purple', 'tab:brown']


def plot(df, args, xaxis):
    plt.figure()

    df_phase = df[df['name'].str.contains("Im")]
    df_RR = df[df['name'].str.contains("Round-Robin")]
    df_prr = df[df['name'].str.contains("PRR")]

    df_prr = df_prr.loc[:, ['name','param', 'cr', xaxis]]
    df_RR = df_RR.loc[:, ['name', 'cr', xaxis]]
    df_phase = df_phase.loc[:, ['name', 'param', 'cr', xaxis]]

    df_prr['param'] = df_prr['param'].map('PRR (λ = {})'.format)
    df_prr['param'] = df_prr['param'].map('PRR (λ = {})'.format)
    df_phase['param'] = df_phase['param'].map('Im et al. (ε = {})'.format)

    #plt.rc('axes', prop_cycle=(cycler('color', colors)))


    sns.set_theme(style='white')


    sns.lineplot(data=df_phase, x="sigma", y="cr", hue='param', linestyle=':', markers=True)

    sns.lineplot(data=df_RR, x="sigma", y="cr", label="Round-Robin")

    sns.lineplot(data=df_prr, x="sigma", y="cr", hue="param", linestyle='--', markers=True)

    
    #grouped_data = df_phase.groupby(['param', xaxis]).mean().unstack('param')
    #for cr, param in list(grouped_data):
        #grouped_data[(cr,param)].plot(
        #    style='D-', markersize=4, linewidth=1.2, label=f"Im et al. (ε = {param})", legend=True)

    #grouped_data = df_prr.groupby(['param', xaxis]).mean().unstack('param')
    #for cr, param in list(grouped_data):
    #    grouped_data[(cr,param)].plot(
    #        style='s:', markersize=4, linewidth=1.2, label=f"PRR (λ = {param})", legend=True)

    #grouped_data = df_RR.groupby([xaxis]).mean()
    #grouped_data['cr'].plot(style='o--', markersize=4, linewidth=1.2, label=f"Round-Robin", legend=True)

 
    #plt.plot((0, max_bin), (1, 1), 'black')
    plt.xlabel(xaxis)
    plt.ylabel('Empirical competitive ratio')
    plt.legend()
    plt.tight_layout()
    #plt.axis([0, max_bin, 0.99, 1.1])


    fig = plt.gcf()
    fig.set_dpi(290)
    fig.set_size_inches(3,2)


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.file):

        data = get_data(parsed_args.file)
        plot(data, parsed_args, 'sigma')
       # plot_lambda(data, float(parsed_args.bin_size),
       #             parsed_args, det_alg, pred_alg)
        plt.show()
    else:
        print("Path not valid!")
