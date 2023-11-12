import numpy as np


def get_A_at_z(z, xs):
    result = 1
    for x in xs:
        result *= (z - x)
    return result

def get_A_deriv(i, xs):
    result = 1
    xi = xs[i]
    for j in range(len(xs)):
        if j != i:
            result *= (xi - xs[j])
    return result



points = [2,3]
evals = [2, 3]
        
z = 5
 
result = get_A_at_z(z, points)
s = 0
for i in range(len(evals)):
    s += evals[i] / ((z - points[i])* get_A_deriv(i, points))
result *= s
print(result)

points = [32, 33, 34, 35, 36]
evals = [1,11,111,1111,11111]
        
z = 2
 
result = get_A_at_z(z, points)
s = 0
for i in range(len(evals)):
    s += evals[i] / ((z - points[i])* get_A_deriv(i, points))
result *= s
print(result)

    
