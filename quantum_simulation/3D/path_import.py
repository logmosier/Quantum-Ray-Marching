import random
import numpy as np

class PixelData:
    def __init__(self, d):
        self.pixel = d["position"]
        paths = d["paths"]
        self.samples = []
        steps = max([len(p) for p in paths])
        print("steps", steps)
        for c in range(3):
            self.samples.append(generate_path(paths, c))



def parse_file(file):
    import json
    pixels = []
    with open(file) as f:
        data = json.load(f)
        for d in data:
            pixels.append(PixelData(d))
    return pixels
    
def generate_path(paths, ci):
    num_paths = 32
    num_steps = max([len(p) for p in paths])
    print("num_paths", len(paths))
    print("num_steps", num_steps)
    emitted_samples = np.zeros((num_paths, num_steps))
    diffuse_samples = np.ones((num_paths, num_steps))
    random.shuffle(paths)
    for (pi,path) in enumerate(paths[:num_paths]):
        for (si,step) in enumerate(path[:num_paths]):
            diffuse_samples[pi,si] = step[0][ci]
            emitted_samples[pi,si] = step[1][ci]
    return (emitted_samples, diffuse_samples)