#! /usr/bin/python3

'''Simple example script for objective function evaluation'''

import sys
import json
import random

MAX_NUM_PTS = 10_000_000


def obj_func(x_val, y_val):
    '''Evaluates objective function'''
    NUM_PTS = int(random.random() * MAX_NUM_PTS)
    acc = 0.0
    for _ in range(NUM_PTS):
        tmp_x = x_val - random.random()
        tmp_y = y_val - random.random()
        acc += tmp_x * tmp_x + tmp_y * tmp_y
    return acc / NUM_PTS


if __name__ == '__main__':
    data = json.loads(sys.argv[1])
    obj_func_val = obj_func(data['x'], data['y'])
    result = {
        "obj_func_val": obj_func_val
    }
    print(json.dumps(result))
