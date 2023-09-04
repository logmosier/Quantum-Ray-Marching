from qiskit import QuantumCircuit, QuantumRegister

def increment_gate(num_bits):
    x = QuantumRegister(num_bits)
    ic = QuantumCircuit(x, name="incerment")
    for i in reversed(range(x.size)):
        if i > 0:
            ic.mcx(x[:i], x[i])
        else:
            ic.x(x[i])
    return ic.to_gate()

def decrement_gate(num_bits):
    return increment_gate(num_bits).inverse()

def shift_gate(num_path_bits, num_dir_bits):
    xb = QuantumRegister(num_path_bits)
    yb = QuantumRegister(num_path_bits)
    cb = QuantumRegister(num_dir_bits)   
    qc = QuantumCircuit(xb, yb, cb)
    #Increment y if dir is 0, 1, 7
    iy_states = [("00",cb[1:]), ("111",cb[:])]
    for (state, bits) in iy_states:
        qc.append(decrement_gate(yb.size).control(len(bits), ctrl_state=state), bits+yb[:])
    
    #Decrement y if dir is 3, 4, 5
    dy_states = [("011",cb[:]), ("10",cb[1:])]
    for (state, bits) in dy_states:
        qc.append(increment_gate(yb.size).control(len(bits), ctrl_state=state), bits+yb[:])

    #Increment x if dir is 1,2,3
    dx_states = [("01",[cb[0], cb[2]]), ("01",[cb[1], cb[2]])]
    for (state, bits) in dx_states:
        qc.append(increment_gate(xb.size).control(len(bits), ctrl_state=state), bits+xb[:])
    qc.append(decrement_gate(xb.size).control(3, ctrl_state="011"), cb[:]+xb[:]) # subtract 1 from x if both coins are true

    #Decement x if dir is 5,6,7
    dx_states = [("11",[cb[0], cb[2]]), ("11",[cb[1], cb[2]])]
    for (state, bits) in dx_states:
        qc.append(decrement_gate(xb.size).control(len(bits), ctrl_state=state), bits+xb[:])
    qc.append(increment_gate(xb.size).control(3, ctrl_state="111"), cb[:]+xb[:]) # add 1 from x if both coins are true

    return qc.to_gate(label="shift")