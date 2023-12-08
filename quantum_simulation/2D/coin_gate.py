from qiskit import Aer,transpile,QuantumCircuit, QuantumRegister,ClassicalRegister

from qiskit.circuit import Gate
from qiskit.extensions import UnitaryGate, Initialize
from qiskit.circuit.library import IGate, XGate, HGate, RYGate, MCXGate, StatePreparation
from qiskit.quantum_info import Statevector
from qiskit.quantum_info.synthesis import two_qubit_cnot_decompose
from qiskit.algorithms import AmplificationProblem
from qiskit.quantum_info import Operator

from qiskit.visualization import *
from qiskit.algorithms import Grover

import numpy as np
from qiskit import *

from typing import List
from qiskit.algorithms import EstimationProblem
from qiskit.primitives import Sampler
# import fea

# from fea import FasterAmplitudeEstimation, FasterAmplitudeEstimationResult
from qiskit.utils import QuantumInstance
# from pyeda.inter import *

# from pyeda.boolalg.expr import OrOp, Complement,Literal
import collections
collections.Sequence = collections.abc.Sequence

from concurrent.futures import ThreadPoolExecutor
from multiprocessing import Pool
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec

class Coin():
    def __init__(self, normal=None, diffuse = True):
        self.air = normal is None
        self.normal = normal
        self.diffuse = diffuse



def boundary_coin(cb: QuantumRegister):
    qc = QuantumCircuit(cb)
    qc.x(cb[2])
    return qc.to_gate(label="boundary_coin")

def lambertian_coin(normal): # convert to coin from new work?
    qc = QuantumCircuit(3)
    leading_term = 1.0 / np.sqrt(4)
    matrix = np.array([[0]*8]*8)
    praller_indexs = [(normal+2)%8, (normal-2)%8]
    incoming_indexs = [(normal+3)%8, (normal+4)%8,  (normal+5)%8]
    outgoing_indexs = [(normal)%8, (normal-1)%8, (normal+1)%8]
    negative_terms= [
        [2,3],
        [1,3],
        [1,2],
        [],
    ]
    for i in outgoing_indexs:
        matrix[praller_indexs[0]][i] = 1
        matrix[i][praller_indexs[1]] = 1
        for j in incoming_indexs:
            matrix[i][j] = 1

    for i in incoming_indexs:
        matrix[i][praller_indexs[0]] = 1
        matrix[praller_indexs[1]][i] = 1
        for j in outgoing_indexs:
            matrix[i][j] = 1
    for i in praller_indexs:
        matrix[i][i] = 1

    out_going_index = -1
    incoming_index = -1
    for (i,row) in enumerate(matrix):
        non_zero_index = 0 
        outgoing_row = row[0] == 0
        if outgoing_row:
            out_going_index += 1
        else:
            incoming_index += 1

        negative_terms_index = out_going_index if outgoing_row else incoming_index
        for (j,v) in enumerate(row):
            if v != 0:
                if non_zero_index in negative_terms[negative_terms_index]:
                    matrix[i][j] *= -1
                non_zero_index += 1
    print(matrix)
    qc.unitary(leading_term * matrix, qc.qubits)
    return qc.to_gate(label="lambertian_coin_"+str(normal))

def make_coin(path_bits, dir_bits, scene, pre_made_lambertian_coins = None, simplify = True):
    coins = coins = [lambertian_coin(i) for i in range(8)] if pre_made_lambertian_coins == None else pre_made_lambertian_coins # Premake and store the coins for faster circuit construction

    xb = QuantumRegister(path_bits)
    yb = QuantumRegister(path_bits)
    cb = QuantumRegister(dir_bits)
    qc = QuantumCircuit(xb, yb, cb)
    print("gates made")
    width = len(scene[0])
    xy = xb[:]+yb[:]
    # if simplify:
    #     bit_map = {}
    #     for (x, row) in enumerate(scene):
    #         for (y, coin) in enumerate(row): 
    #             if coin.air:
    #                 continue
    #             normal = coin.normal
    #             if bit_map.get(normal) == None:
    #                 bit_map[normal] = "0" * len(scene)**2
    #             i = width * x + y
    #             bit_map[normal] = bit_map[normal][:i] + "1" + bit_map[normal][i+1:]
    #     # print(bit_map)
    #     gates_num = 0
    #     for (normal, states) in bit_map.items():
    #         X = ttvars('x', 2*path_bits)
    #         states = states + "-" * (2**path_bits - len(states))
    #         f = truthtable(X, states)
    #         exp = espresso_tts(f)[0]
    #         cgate =  coins[normal]
    #         # print(exp)
    #         if exp.is_one():
    #             qc.append(cgate, cb)
    #             gates_num += 1
    #         elif isinstance(exp, Literal):
    #                     complement = isinstance(exp, Complement)
    #                     bit = xy[int(str(exp)[3 if complement else 2])]
    #                     ctrl_state = "0" if complement else "1"
    #                     # print(ctrl_state)
    #                     # print(bit)
    #                     qc.append(cgate.control(1, ctrl_state=ctrl_state), [bit] + cb[:])    
    #                     gates_num += 1
    #         else:
    #             for t in exp.xs:
    #                 # print(t)
    #                 if isinstance(t, Literal):
    #                     complement = isinstance(t, Complement)
    #                     bit = xy[int(str(t)[3 if complement else 2])]
    #                     ctrl_state = "0" if complement else "1"
    #                     # print(ctrl_state)
    #                     # print(bit)
    #                     qc.append(cgate.control(1, ctrl_state=ctrl_state), [bit] + cb[:])    
    #                     gates_num += 1
        
    #                 else:
    #                     control_bits = []
    #                     ctrl_state = ""
    #                     for b in t.xs:
    #                         if isinstance(b, Complement):
    #                             ctrl_state += "0"
    #                             control_bits.append(xy[int(str(b)[3])])
    #                         else:
    #                             ctrl_state += "1"
    #                             control_bits.append(xy[int(str(b)[2])])
    #                     qc.append(cgate.control(len(control_bits), ctrl_state=ctrl_state), control_bits[:]  + cb[:])
    #                     gates_num += 1
    #     # print("circuit reduced from {} to {}".format(len(sample_array), gates_num))
    # else:
    for (y,x), coin in np.ndenumerate(scene):
        print("coin at :", x,y, coin.air, coin.normal)
        if coin.air:
            continue
        x_ctrl_state = "{0:b}".format(x).zfill(xb.size)
        y_ctrl_state = "{0:b}".format(y).zfill(yb.size)
        if coin.diffuse:
            qc.append(coins[coin.normal].control(path_bits*2, label = str(x)+","+str(y), ctrl_state =  y_ctrl_state + x_ctrl_state), xb[:]+yb[:]+cb[:])
        else:
            qc.append(boundary_coin(cb).control(path_bits*2, label = str(x)+","+str(y), ctrl_state =  y_ctrl_state + x_ctrl_state), xb[:]+yb[:]+cb[:])
    print("coin made")
    print(qc.draw())
    return qc
