from qiskit import QuantumCircuit, QuantumRegister,ClassicalRegister

def measure_register(qc: QuantumCircuit, reg: QuantumRegister, frac_bits = 0):
    if frac_bits > 0:
        int_reg = ClassicalRegister(reg.size - frac_bits, reg.name+"_int_mes")
        frac_reg = ClassicalRegister(frac_bits, reg.name+"_frac_mes")
        qc.add_register(frac_reg,int_reg)
        qc.measure(reg[frac_bits:], int_reg)
        qc.measure(reg[:frac_bits], frac_reg)

    else:
        m_reg = ClassicalRegister(reg.size, reg.name+"_mes")
        qc.add_register(m_reg)
        qc.measure(reg, m_reg)
