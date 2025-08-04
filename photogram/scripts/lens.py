import math
rad = 3.14159265/180

class Poly:
    def __init__(self, p):
        self.p = p[:]
        pass
    def calc(self, n):
        r = 0
        x = 1
        for p in self.p:
            r += p*x
            x *= n
            pass
        return r
    pass


lens_stars = Poly(
    [
    0.0,
    0.9926782375237195,
    0.0,
    0.08523540767237137,
    0.0,
    0.18717584140176768
  ]
    )
lens_grid = Poly(
    [
    0.0,
    0.9910080700407145,
    0.0,
    0.14212768870589798,
    0.0,
    0.04690762704558438
  ]
)
lens_grid_inv = Poly(
    [
0.0,
    1.0090754323185678,
    0.0,
    -0.1475322994748467,
    0.0,
    0.01851542667918693
  ]
)

fov = 20
px_width = 5200
for i in range(14*5):
    s = (i+1)/5
    w_stars = lens_stars.calc(rad*s)/rad
    w_grid = lens_grid.calc(rad*s)/rad
    print(s, w_grid, w_stars-w_grid, (math.tan(w_stars*rad)-math.tan(w_grid*rad))*px_width/2/math.tan(fov/2*rad) )
    pass

for i in range(14*5):
    s_stars = (i+1)/5
    w_stars = lens_stars.calc(rad*s_stars)/rad
    s_grid = lens_grid_inv.calc(rad*w_stars)/rad
    print(s_stars, w_stars, s_grid, s_grid/s_stars)
    pass
