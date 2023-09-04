from qiskit import QuantumCircuit, QuantumRegister
from qiskit.algorithms import EstimationProblem, FasterAmplitudeEstimation
from qiskit.circuit.library import IGate, XGate, MCXGate, HGate, RYGate, StatePreparation

import numpy as np

def walk_circuit(diff_sample_circuit, light_sample_circuit, coin_circuit, shif_circuit, path_bits, dir_bits, steps, backend):
    x = QuantumRegister(path_bits, "x")
    y = QuantumRegister(path_bits, "y")
    dif_out = QuantumRegister(steps-1, "dif_out")
    light_out = QuantumRegister(steps, "light_out")
    dir = QuantumRegister(dir_bits, "dir")

    wc = QuantumCircuit(dif_out, light_out, x, y, dir, name="walk")
    for i in range(steps):
        wc.append(coin_circuit, x[:] + y[:] + dir[:])
        wc.append(shif_circuit, x[:] + y[:] + dir[:])
        print("step", i)
        wc.append(light_sample_circuit, x[:] + y[:] + [light_out[i]])
        if i +1 == steps:
            break
        wc.append(diff_sample_circuit, x[:] + y[:] + [dif_out[i]])

    print("walk made")
    return wc

def quantum_ray_marching_oracle(scene, point, path_bits, steps, dir_bits, walk_circuit, path_circuit):
    x = QuantumRegister(path_bits, "x")
    y = QuantumRegister(path_bits, "y")
    dir = QuantumRegister(dir_bits, "dir")
    dif_out = QuantumRegister(steps-1, "dif_out")
    light_out = QuantumRegister(steps, "light_out")
    num_step_bits = 1 + np.floor(np.log2(steps - 1)) if steps > 1 else 1
    control_bits = QuantumRegister(num_step_bits, "control_bits")
    out = QuantumRegister(1, "out")
    lfc = QuantumCircuit(out, dif_out, light_out, x, y, dir, control_bits, name = "LightFeildCircuit")
    
    lfc.append(StatePreparation("{0:b}".format(point[0]).zfill(path_bits)), x[:])
    lfc.append(StatePreparation("{0:b}".format(point[1]).zfill(path_bits)), y[:])
    if scene[point[1]][point[0]].air:
        lfc.h(dir)
    else:
        normal = scene[point[1]][point[0]].normal
        lfc.append(StatePreparation("{0:b}".format((normal +4)%8).zfill(dir_bits)), dir[:])


    lfc.append(walk_circuit, dif_out[:] + light_out[:] + x[:] + y[:] + dir[:])
    lfc.append(path_circuit, out[:]+ dif_out[:]+ light_out[:] + control_bits[:])

    return lfc

def evaluate_point(scene, pb_bits, steps, d_bits, point, walk_circuits, backend, process_path_circuit):
    results = []
    for i in range(3):
        print("starting walk", i)
        lfc = quantum_ray_marching_oracle(scene, point, pb_bits, steps, d_bits, walk_circuits[i], process_path_circuit) 
        print("qubits:" , lfc.num_qubits, "depth:", lfc.depth())
        print("circuit made")
        problem = EstimationProblem(
            state_preparation=lfc,  # A operator
            objective_qubits= 0,  # the "good" state Psi1 is identified as measuring |1> in qubit 0
        )
        print("problem made")
        fae = FasterAmplitudeEstimation(
            delta=0.000001,  # target accuracy
            maxiter=5,  # determines the maximal power of the Grover operator
            quantum_instance=backend,
            rescale=True
        )
        print("running")
        r = fae.estimate(problem)
        print(r)
        results.append(r)
    return results

def render_quantum_light_map(scene, pb_bits, steps, d_bits, backend, walk_circuits, process_path_circuit):
    q_light_map = np.zeros_like(scene)
    background = np.array([1.0,1.0,1.0])

    for (x, row)  in enumerate(q_light_map):
        for (y, pixel) in enumerate(row):
            print("Pixel", x,y)
            q_light_map[x,y] = evaluate_point(scene, pb_bits, steps, d_bits, [x,y], walk_circuits, backend, process_path_circuit)

    print(np.dstack(q_light_map))

def sample_circuit(sample_array, path_bits, c):
    x = QuantumRegister(path_bits, name = "x")
    y = QuantumRegister(path_bits, name = "y")
    out = QuantumRegister(1, name = "out")
    sc = QuantumCircuit(x,y,out, name = "SampleCircuit")
    width = len(sample_array[0])
    xy = x[:]+y[:]
    for (xi, row) in enumerate(sample_array):
        x_ctrl_state = "{0:b}".format(xi).zfill(path_bits)
        for (yi, v) in enumerate(row):
            if v[c] == 0:
                continue
            y_ctrl_state = "{0:b}".format(yi).zfill(path_bits)
            ry = RYGate(2 * np.arcsin(np.sqrt(v[c]))).control(2*path_bits, ctrl_state = x_ctrl_state+ y_ctrl_state)
            sc.append(ry, x[:] + y[:] + out[:])
    return sc