#!/usr/bin/env python
import math

class pt:
    def __init__(self, coords):
        self.coords = coords[:]
        pass
    def copy(self):
        return pt(coords=self.coords)
    def scale(self,x):
        self.coords = [self.coords[i]*x for i in range(len(self.coords))]
        return self
    def add_scaled(self,sc,oth,oth_sc):
        self.coords = [self.coords[i]*sc + oth_sc*oth.coords[i] for i in range(len(self.coords))]
        return self
    def dot_product(self,oth):
        r = 0.
        for i in range(len(self.coords)):
            r += self.coords[i] * oth.coords[i]
            pass
        return r
    def length(self):
        return math.sqrt(self.dot_product(self))
    def normalize(self):
        l = self.length()
        if (l>=0.00000001): self.scale(1.0/l)
        return self
    @staticmethod
    def diff (x,y):
        return x.copy().add_scaled(1.0,y,-1.0)
    @staticmethod
    def cross_product(a,b):
        x = a.coords[1]*b.coords[2] - a.coords[2]*b.coords[1]
        y = a.coords[2]*b.coords[0] - a.coords[0]*b.coords[2]
        z = a.coords[0]*b.coords[1] - a.coords[1]*b.coords[0]
        return pt(coords=[x,y,z])
    def __repr__(self):
        return "(%f,%f,%f)"%(self.coords[0],
                             self.coords[1],
                             self.coords[2])

                             
class line:
    def __init__(self, start, direction):
        self.start = start
        self.direction = direction.copy().normalize()
        pass
    def distance_from_line(self, p):
        """
        rp = p relative to start
        rp.direction = amount of direction
        Hence rp = rp.direction * direction + perpendicular_vector
        perpendicular_vector = rp - rp.direction * direction
        And distance = mod(perpendicular_vector)
        """
        rp = pt.diff(p, self.start) # make relative to start of line
        rp_along_d = self.direction.copy().scale(rp.dot_product(self.direction)) # Get component in direction of line
        rpr = pt.diff(rp, rp_along_d) # perpendicular
        return rpr.length()
    @staticmethod
    def distance_between_lines(a,b):
        bvd = pt.cross_product(a.direction, b.direction)
        sd = pt.diff(a.start, b.start)
        return abs(sd.dot_product(bvd))
    @staticmethod
    def midpoint_between_lines(a,b):
        """
        (define the line between midpoints to be c1<->c2, c1 on a, c2 on b)
        avb = vector between the closest points (perp to both line directions)
        (line between midpoints is intersection of these planes, call it p1<->p2)
        a_n = normal to plane containing a and avb = ad x (ad x bd), and (p-as).(ad x (ad x bd))=0 for all points on the plane
        b_n = normal to plane containing b and avb = bd x (bd x ad), and (p-bs).(bd x (bd x ad))=0 for all points on the plane
        (line between midpoints is intersection of these planes)
        (c1 - bs).(bd x (bd x ad))=0 and c1 = as + k * ad hence
        (as + k * ad - bs).(bd x (bd x ad))=0
        (k * ad).(bd x (bd x ad)) + (as-bs).(bd x (bd x ad))=0
        k * ad.(bd x (bd x ad)) = (bs-as).(bd x (bd x ad))
        k = (bs-as).(bd x (bd x ad)) / ad.(bd x (bd x ad)) 
        k = (bs-as).b_n / ad.b_n
        c1 = as + (bs-as).b_n / ad.b_n * ad
        and
        c2 = bs + (as-bs).a_n / bd.a_n * bd
        """
        avb = pt.cross_product(a.direction, b.direction)
        a_n = pt.cross_product(a.direction, avb)
        b_n = pt.cross_product(b.direction, avb)
        bs_m_as = pt.diff(b.start, a.start)
        c1 = a.start.copy().add_scaled(sc=1.0, oth=a.direction, oth_sc=bs_m_as.dot_product(b_n) / a.direction.dot_product(b_n))
        c2 = b.start.copy().add_scaled(sc=1.0, oth=b.direction, oth_sc=-1.0*bs_m_as.dot_product(a_n) / b.direction.dot_product(a_n))
        d = pt.diff(c1,c2).length()
        #print "Distance",pt.diff(c1,c2).length(), c1, c2
        return (c1.add_scaled(sc=0.5, oth=c2, oth_sc=0.5), d)
    def __repr__(self):
        return "[%s -> %s]"%(str(self.start), str(self.direction))
    pass

l0 = line(pt([0.,0.,0.]), pt([0.,0.,-1.]))
l1 = line(pt([1.,0.,0.]), pt([1.,1.,0.]))
print line.distance_between_lines(l0,l1)
print line.distance_between_lines(l1,l0)

class led:
    def __init__(self, xyz):
        self.xyz = xyz
        pass
    def distance_from_line(self, l):
        return l.distance_from_line(self.xyz)
    
class camera:
    """
    Each camera has an x,y,z position, a view direction and magnification (dx,dy,dz), and a unit up direction (perpendicular to dx,dy,dz) effectively given by a single value.
    Then every point in 3D space is mapped to a 2d XY location for the camera.
    We can characterize the camera as a 3d vector and a unit quaternion and a scale.
    Then camera(xy) = Perspective(Cs*Rotation_Matrix(Cq)*(LED XYZ - Cxyz))
    OR
    k*(camera(xy),1) = Cs*Rotation_Matrix(Cq)*(Lxyz - Cxyz)
    OR 
    k*(camera(xy),1) = Cs*Rotation_Matrix(Cq)*Lxyz + C'xyz
    OR
    k*(camera(xy),1) - C'xyz = Cs*Rotation_Matrix(Cq)*Lxyz
    [Cs*Rotation_Matrix(Cq)^-1] (k*(camera(xy),1) - C'xyz) = Lxyz
    M^-1 . (k.Cx k.Cy k 1) = Lx Ly Lz 1

    If we define a camera to be at the origin we and looking straight with scale 1 (to give a frame of reference) we have
    camera0(xy) = Perspective(LED XYZ), or LED XYZ = k.(cx cy 1) for some k
    Indeed, we can 
    If there are 2 cameras for the same LED(XYZ), the second camera now has 
    camera0(xy) = Cs
    """
    cx = 320.0
    width = 640.0
    cy = 240.0
    def __init__(self, xyz, zrot=0., yrot=0., xfov=45):
        self.pts = []
        self.xyz = xyz
        self.zrot = zrot
        self.yrot = yrot
        self.fov = math.radians(xfov)
        pass
    def add_point(self, t, sx, sy):
        self.pts.append(((sx+0.)/2/t-self.cx,(sy+0.)/t-self.cy))
        pass
    def angles_of_pt(self, n):
        """
        Perform inverse of lens projection for point (x,y)
        """
        (x,y) = self.pts[n]
        pitch = math.sqrt(x*x+y*y) * self.fov / self.width
        roll = math.atan2(y,x)
        return (pitch,roll)
    def xy_of_angles(self, p, r):
        """
        Perform lens projection for pitch p roll r
        Pitch is actually yaw - atan( sqrt(x^2+y^2) / sqrt(x^2+y^2+z^2) )
        Basically we rotate (X,0) anticlockwise by roll
        where X is the pixel distance due to 'pitch'
        So if pitch is equal to FOV/2 then X is width/2
        """
        d = p / self.fov * self.width # assuming angle proportional to pixels from centre
        y = d * math.sin(r) + self.cy
        x = d * math.cos(r) + self.cx
        return (x,y)
    def line_of_pt(self, n):
        """
        Determine direction vector (x,y,z) for the point
        (for no real lens projection should be k*(x,y,1))
        """
        (p,r) = self.angles_of_pt(n)
        x = math.sin(p)*math.cos(r+self.zrot)
        y = math.sin(p)*math.sin(r+self.zrot)
        z = math.cos(p)
        x2 = x*math.cos(self.yrot) + z*math.sin(self.yrot)
        z2 = z*math.cos(self.yrot) - x*math.sin(self.yrot)
        return line(self.xyz, pt(coords=[x2,y,z2]))
    def pr_of_xyz(self, xyz):
        """
        Determine pitch/roll of an xyz from the projection
        """
        d = xyz.length()
        x = xyz.coords[0]
        y = xyz.coords[1]
        z = xyz.coords[2]
        x2 = math.cos(-self.yrot)*x + math.sin(-self.yrot)*z
        z2 = math.cos(-self.yrot)*z - math.sin(-self.yrot)*x
        p = math.acos(z2/d)
        if p > math.pi/2: p -= math.pi
        r = math.atan2(y,x2)
        r -= self.zrot
        return (p,r)
    def xy_of_xyz(self, xyz):
        """
        Determine screen x,y of an xyz from the projection
        """
        (p,r) = self.pr_of_xyz(xyz)
        return self.xy_of_angles(p,r)
    def __str__(self):
        r = str(self.xyz) + str(self.pts)
        return r
    pass

# (20,10,-60)
pts = [(0,4,3152,928),(0,4,2832,1040),(0,4,3280,1168),(0,4,2576,1360),(0,4,3088,1408),(1,4,4112,736),(1,4,4208,816),(1,4,3312,928),(1,4,3728,1024),(1,4,3216,1280),(1,4,3600,1312),]
#pts = [(0,4,3376,1168),(1,4,4336,976),]
#pts = [(0,4,2800,1280),(1,4,4400,1024),(1,4,2928,1152)]
#pts = [(0,4,2960,1136),(0,4,2704,1296),(0,4,3152,1392),(0,4,3024,1648),(1,4,4208,1072),(1,4,3792,1280),(1,4,3216,1568),(1,4,3472,1632),]
#pts = [(0,8,7552,1152),(0,4,3440,928),(1,4,4336,288),(1,4,3984,656),]
#pts = [(0,4,3248,928),(0,4,1936,1440),(0,4,2704,1632),(0,4,2032,1696),(1,4,3984,736),(1,4,2768,1456),(1,4,3632,1584),(1,8,6080,3424),]
pts = [(0,4,3184,1024),(0,4,2768,1040),(0,4,3120,1216),(0,4,2960,1440),(1,4,4176,848),(1,4,4240,960),(1,4,3536,1104),(1,4,3120,1296),(1,4,3568,1360),]
       
# (0.000000,0.000000,0.000000) [(74.0, -8.0),   (34.0, 20.0),                 (90.0, 52.0), (2.0, 100.0), (66.0, 112.0)]
# (1.000000,0.000000,0.000000)[(194.0, -56.0), (206.0, -36.0), (94.0, -8.0), (146.0, 16.0), (82.0, 80.0), (130.0, 88.0)]

cs = [camera(xyz=pt([0.,0.,0.]), xfov=45.0, zrot=0., yrot=0.),
      camera(xyz=pt([30.,0.,0.]), xfov=45.0, zrot=0.22, yrot=-0.677),
    ]

if False: # Test xy_of_xyz, add_point, distance_from_line
    cs[0].yrot = 0.1
    cs[0].zrot = 0.3
    test_p = pt([20.,10.,-60.])
    (p,r) = cs[0].pr_of_xyz(test_p)
    # The following should be (for 20, 10, -60) about 20.38 degrees pitch and 26.6 degrees roll
    print "Should be about 20.38 and 26.6 (if yrot/zrot are 0)",math.degrees(p), math.degrees(r)
    (x,y) = cs[0].xy_of_xyz(test_p)
    # Since fov is 45 degrees (x,y) should be about 20.38/45*640 pixels from the centre (290 pixels)
    # This is x of 260 and y of 130, and if centred on 320,240 then this is 60/580, 110/370
    print "Should be 60,110 (if yrot/zrot are 0)",(x,y)
    cs[0].add_point(1.0,x*2.0,y)
    line_p = cs[0].line_of_pt(0)
    # Line goes from (0,0,0) through test_p, so its direction should be k*test_p
    print "Should be proportional", test_p, line_p.direction
    print "Hence should be equal", test_p.coords[0]/line_p.direction.coords[0], test_p.coords[1]/line_p.direction.coords[1], test_p.coords[2]/line_p.direction.coords[2]
    # Line goes through test_p, so distance should be 0
    print "Should be zero", line_p.distance_from_line(test_p)
    print (x,y), cs[0].xy_of_xyz(line_p.direction)
    #print math.degrees(cs[0].angles_of_pt(0)[0]), math.degrees(cs[0].angles_of_pt(0)[1])
    pass

for (d,t,sx,sy) in pts:
    cs[d].add_point(t,sx,sy)
    pass
print cs[0], cs[1]

cs[1].zrot = 0.307
cs[1].yrot = -0.327
cs[1].fov  = 0.700
if False:
  min_err = (None, None, None, None)
  for z in range(20):
    cs[1].zrot = 0.307    +(z-10)*0.003
    cs[1].yrot = -0.327 #   + (z-10)*0.003
    cs[1].fov = 0.700 #  + (z-10)*0.001
    print z,math.degrees(cs[1].fov),cs[1].fov,cs[1].zrot,cs[1].yrot,
    err = 0.
    # (1,2), 
    for (p1,p2) in [(2,3), (3,4), (4,5)]:
        d = line.distance_between_lines(cs[0].line_of_pt(p1), cs[1].line_of_pt(p2))
        print d,
        err = err + abs(d)
        pass
    print err
    if (min_err[0] is None) or (err<min_err[0]):
      min_err = (err, cs[1].zrot, cs[1].yrot, cs[1].fov)
      pass
    pass
  print min_err
  cs[1].zrot = min_err[1]
  cs[1].yrot = min_err[2]
  cs[1].fov  = min_err[3]
  
if True:
  for p1 in range(len(cs[0].pts)):
    min = (None, None)
    for p2 in range(len(cs[1].pts)):
        d = line.distance_between_lines(cs[0].line_of_pt(p1), cs[1].line_of_pt(p2))
        if min[0] is None:
            min = (d, p2)
            pass
        elif d<min[0]:
            min = (d, p2)
            pass
        pass
    print p1, min, line.midpoint_between_lines(cs[0].line_of_pt(p1), cs[1].line_of_pt(min[1]))
    pass
               

