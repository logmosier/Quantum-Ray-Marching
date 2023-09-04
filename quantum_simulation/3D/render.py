from qiskit import Aer, QuantumCircuit, QuantumRegister
from qiskit.circuit.library import RYGate, MCXGate, StatePreparation
import numpy as np
from qiskit.algorithms import EstimationProblem,FasterAmplitudeEstimation

def sample_circuit(sample_array, path_bits):
    x = QuantumRegister(path_bits)
    out = QuantumRegister(1)
    sc = QuantumCircuit(x, out, name="sample")
    for (i,v) in enumerate(sample_array):
        ctrl_state = "{0:b}".format(i).zfill(x.size)
        ry =  RYGate(2 * np.arcsin(np.sqrt(v))).control(x.size, ctrl_state=ctrl_state)
        sc.append(ry, x[:] + out[:])
    return sc

def walk_circuit(dif_sample_array, light_sample_array):
    (paths,steps) = dif_sample_array.shape
    state_bits = 1 + np.floor(np.log2(paths - 1)) if paths > 1 else 1
    x = QuantumRegister(state_bits, "state")
    dif_out = QuantumRegister(steps, "dif_out")
    light_out = QuantumRegister(steps, "light_out")
    wc = QuantumCircuit(dif_out, light_out, x, name="walk")
    wc.append(init(paths)[0], x[:])
    for i in range(steps):
        wc.append(sample_circuit(dif_sample_array[:,i], x.size), x[:] + [dif_out[i]])
        wc.append(sample_circuit(light_sample_array[:,i], x.size), x[:] + [light_out[i]])
    return wc , dif_out, light_out, x

def init(steps):
    num_bits = 1 + np.floor(np.log2(steps - 1)) if steps > 1 else 1
    control_bits = QuantumRegister(num_bits, "control_bits")
    qc = QuantumCircuit(control_bits)
    intial_state = [1/np.sqrt(steps) for i in range(steps)] + [0]* (2**control_bits.size - steps)
    qc.append(StatePreparation(intial_state), control_bits)
    return qc.to_gate(), qc.to_gate().inverse()

def process_path(steps):
    difuse_steps  = QuantumRegister(steps, "difuse_steps")
    light_steps = QuantumRegister(steps, "light_steps")
    num_step_bits = 1 + np.floor(np.log2(steps - 1)) if steps > 1 else 1
    control_bits = QuantumRegister(num_step_bits, "control_bits")
    out = QuantumRegister(1, "out") 
    pc = QuantumCircuit(out, difuse_steps, light_steps ,control_bits, name="process_path")
    
    if num_step_bits > 1:
        (inti_instruction ,init_inverse) = init(steps)
        pc.append(inti_instruction, control_bits)

    #calculate the products of each step 
    for i in range(steps):
        mult_gate = MCXGate(
            control_bits.size + i + 1, 
            ctrl_state = "{0:b}".format(i).zfill(control_bits.size)+ "1" * i + "1"
        )
        pc.append(mult_gate, difuse_steps[0:i] + [light_steps[i]] + control_bits[:] +  [out[0]])
    return pc


def A(diffuse_samples, emitted_samples , max_path_qubits = 100):
    (paths, steps) = diffuse_samples.shape
    (qc, dif_out, light_out, x) = walk_circuit(diffuse_samples, emitted_samples)
    pout = QuantumRegister(1, "path out")
    num_step_bits = 1 + np.floor(np.log2(steps - 1)) if steps > 1 else 1
    control_bits = QuantumRegister(num_step_bits, "control_bits")
    work_bit = QuantumRegister(1, "work_bits")

    qc.add_register(control_bits, pout)
    qc.barrier()
    qc.compose(process_path(steps), qubits= [pout[0]] + dif_out[:] + light_out[:] + control_bits[:], inplace=True)
    return (qc, pout, control_bits, work_bit)

def estimate_samples(emitted_samples, diffuse_samples, maxiter = 5):
    backend = Aer.get_backend('aer_simulator')
    (qc,pout, control_bits, work_bit) = A(diffuse_samples, emitted_samples)
    problem = EstimationProblem(
        state_preparation=qc,  # A operator
        objective_qubits= qc.num_qubits-1,  # the "good" state Psi1 is identified as measuring |1> in qubit 0
    )
    fae = FasterAmplitudeEstimation(
        delta=1,  # target accuracy
        maxiter=maxiter,  # determines the maximal power of the Grover operator
        # sampler=sampler,
        quantum_instance = backend,
        rescale=True
    )
    return fae.estimate(problem)

def render(pixels, result_images, filter = None):
    for (ci, result_image) in enumerate(result_images):
        for d in pixels:
            x = d.pixel[0]
            y = d.pixel[1]
            if filter is not None  and [x,y,ci] not in filter:
                continue
            print("pixel", x,y,ci)
            (emitted_samples, diffuse_samples) = d.samples[ci]
            num_paths = len(emitted_samples)
            sample = 0
            steps = max([len(p) for p in emitted_samples])

            for (diffuse, emited) in zip(diffuse_samples, emitted_samples):
                r = 1
                path_value = 0
                for (d,e) in zip(diffuse, emited):
                    path_value += e * r
                    r *= d
                sample += path_value
            # print("sample", sample, "num_paths", num_paths, "average", sample/num_paths)
            # print("sample", sample)

            result = estimate_samples(emitted_samples, diffuse_samples)
            print(result.estimation * steps, sample/num_paths)
            result_image[x][y] = (1, 1)   