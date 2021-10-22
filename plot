#!/usr/bin/python3

from cycler import cycler
import argparse
import sys
import os
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
    data['phase_cr'] = data['phase'] / data['opt']
    data['prr_cr'] = data['prr'] / data['opt']
    data['simple_error_rel'] = data['simple_error'] / data['opt']
    data['maxmin_error_rel'] = data['maxmin_error'] / data['opt']
    return data


colors = ['tab:blue', 'tab:orange', 'tab:green', 'tab:red', 'tab:purple', 'tab:brown', 'tab:pink', 'tab:gray', 'tab:olive', 'tab:cyan']


def plot_eta(df, args):
    df = df[df['lambda'].isin(args.lambdas)]

    df_phase = df.loc[:, ['lambda', 'sigma', 'phase_cr']]
    df_prr = df.loc[:, ['lambda', 'sigma', 'prr_cr']]

    plt.rc('axes', prop_cycle=(cycler('color', colors[0:len(args.lambdas)])))

    grouped_data = df_phase.groupby(['lambda', 'sigma']).mean().unstack('lambda')
    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(
            style='D-', markersize=4, linewidth=1.2, label=f"phase (λ = {l:1.2f})", legend=True)

    grouped_data = df_prr.groupby(['lambda', 'sigma']).mean().unstack('lambda')
    for label, l in list(grouped_data):
        grouped_data[(label, l)].plot(
            style='s:', markersize=4, linewidth=1.2, label=f"PRR (λ = {l:1.2f})", legend=True)

    #plt.plot((0, max_bin), (1, 1), 'black')
    plt.xlabel('sigma')
    plt.ylabel('Empirical competitive ratio')
    plt.legend()
    plt.tight_layout()
    #plt.axis([0, max_bin, 0.99, 1.1])


    fig = plt.gcf()
    fig.set_dpi(500)
    fig.set_size_inches(4,2.5)
    # fig.subplots_adjust(right=0.7)
    #fig.savefig("result.png", dpi=400)


if __name__ == "__main__":
    arg_parser = create_arg_parser()
    parsed_args = arg_parser.parse_args(sys.argv[1:])
    if os.path.exists(parsed_args.file):

        data = get_data(parsed_args.file)
        plot_eta(data, parsed_args)
       # plot_lambda(data, float(parsed_args.bin_size),
       #             parsed_args, det_alg, pred_alg)
        plt.show()
    else:
        print("Path not valid!")
