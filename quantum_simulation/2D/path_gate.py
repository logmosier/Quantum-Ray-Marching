from qiskit import QuantumCircuit, QuantumRegister
from qiskit.circuit.library import MCXGate, StatePreparation

import numpy as np

def init_range(max_value):
    num_bits = 1 + np.floor(np.log2(max_value - 1)) if max_value > 1 else 1
    control_bits = QuantumRegister(num_bits, "control_bits")
    qc = QuantumCircuit(control_bits)
    intial_state = [1/np.sqrt(max_value) for i in range(max_value)] + [0]* (2**control_bits.size - max_value)
    qc.append(StatePreparation(intial_state), control_bits)
    return qc.to_gate(), qc.to_gate().inverse()

def path_evaluation_gate(num_steps):
    difuse_steps  = QuantumRegister(num_steps-1, "difuse_steps")
    light_steps = QuantumRegister(num_steps, "light_steps")
    num_step_bits = 1 + np.floor(np.log2(num_steps - 1)) if num_steps > 1 else 1
    control_bits = QuantumRegister(num_step_bits, "control_bits")

    out = QuantumRegister(1, "out") 

    pc = QuantumCircuit(out, difuse_steps, light_steps ,control_bits)
    
    if num_step_bits > 1:
        (inti_instruction , _) = init_range(num_steps)
        pc.append(inti_instruction, control_bits)

    #calculate the products of each step evluated for the light coming into the intal point
    for i in range(num_steps):
        mult_gate = MCXGate(
            control_bits.size + i + 1, 
            ctrl_state = "{0:b}".format(i).zfill(control_bits.size)+ "1" * i + "1"
        )
        pc.append(mult_gate, difuse_steps[0:i] + [light_steps[i]] + control_bits[:] +  [out[0]])


    return pc.to_gate(label="path_evaluation")