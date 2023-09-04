from enum import Enum
import numpy as np

class Direction(Enum):
    N = 0
    NE = 1
    E = 2
    SE = 3
    S = 4 
    SW = 5
    W = 6
    NW = 7

def step(point, d):
    match Direction(d):
        case Direction.N:
            return np.array([point[0], point[1] - 1])
        case Direction.NE:
            return np.array([point[0] + 1, point[1] - 1])
        case Direction.E:
            return np.array([point[0] + 1, point[1]])
        case Direction.SE:
            return  np.array([point[0] + 1, point[1] + 1])
        case Direction.S:
            return np.array([point[0], point[1] + 1])
        case Direction.SW:
            return np.array([point[0] - 1, point[1] + 1])
        case Direction.W:
            return np.array([point[0] - 1, point[1]])
        case Direction.NW:
            return np.array([point[0] - 1, point[1] - 1])

def scatter(scene, point, direction):
    coin = scene[point[1]][point[0]]
    if coin.air:
        return [direction]
    elif coin.diffuse:
        # print(Direction(coin.normal))
        return [(coin.normal + (-2)) % 8,(coin.normal - 1) % 8, coin.normal, (coin.normal + 1) % 8]
    else:
        return [(direction+4)%8]
    
def ray_march(scene, emitted, diffuse, point, steps, direction):
    next_point = step(point, direction)
    if steps == 0:
        # print("\t"*(2-steps),next_point, emitted[next_point[1]][next_point[0]])
        return emitted[next_point[1]][next_point[0]]
    
    # print("\t"*(2-steps),next_point, Direction(direction), steps)
    diffuse_at_point = diffuse[next_point[1]][next_point[0]]
    emitted_at_point = emitted[next_point[1]][next_point[0]]

    # print("\t"*(2-steps),next_point, diffuse_at_point, emitted_at_point)

    next_directions = scatter(scene, next_point, direction)
    
    # print("\t"*(2-steps), next_point, [Direction(d) for d in next_directions])

    incoming = np.zeros(3)
    for d in next_directions:
        incoming  += ray_march(scene, emitted, diffuse, next_point, steps - 1, d)
    
    return diffuse_at_point * incoming/len(next_directions) + emitted_at_point
                    
def capture_light_clasical(scene, emitted, diffuse, point, steps):
    coin = scene[point[1]][point[0]]
    if coin.air:
        directions = list(range(8))
    elif coin.diffuse:
        directions = [(coin.normal +(-2)) % 8,(coin.normal - 1) % 8, coin.normal, (coin.normal + 1) % 8]
    else:
        # return emitted[point[1]][point[0]]
        # print("top", point, [Direction(coin.normal)])
        directions = [coin.normal]
    print("top", point, [Direction(d) for d in directions])
    incoming = np.zeros(3)
    diffuse_at_point = diffuse[point[1]][point[0]]
    emitted_at_point = emitted[point[1]][point[0]]
    if steps>0:
        for d in directions:
            incoming += ray_march(scene, emitted, diffuse, point, steps, d)    
    print(incoming/len(directions))
    return incoming/len(directions)
    
def render_clasical_light_map(scene, emitted, diffuse, steps):
    light_map = np.zeros_like(scene)
    background = np.array([1.0,1.0,1.0])

    for (x, row)  in enumerate(light_map):
        for (y, pixel) in enumerate(row):
            light_map[x,y] = capture_light_clasical(scene, emitted, diffuse, np.array([x,y]), steps)

    print(np.dstack(light_map))