#! /usr/bin/python3

'''Simple example script for objective function evaluation'''

import sys
import json
import random

NUM_PTS = 10_000_000

SPEC = '''
{
    "x": [0, 1],
    "y": [0, 1]
}
'''

def obj_func(x_val, y_val):
    '''Evaluates objective function'''
    acc = 0.0
    for _ in range(NUM_PTS):
        tmp_x = x_val - random.random()
        tmp_y = y_val - random.random()
        acc += tmp_x * tmp_x + tmp_y * tmp_y
    return acc / NUM_PTS

if __name__ == '__main__':
    random.seed(0)
    print(SPEC)
    for line in sys.stdin:
        data = json.loads(line)
        obj_func_val = obj_func(data['x'], data['y'])
        result = {
            "obj_func_val": obj_func_val
        }
        print(json.dumps(result))
