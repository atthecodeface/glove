# Notes on what should work


## Grid lens calibration (use lens '50mm linear')

Using a grid, we can locate (using a few points), orient and then show the mapping - note that at the edges the crosses are not well aligned

Then we can calibrate the lens, relocate (using a few more points), orient, and then show the mapping. This has near-perfect alignment of the crosses.

See scripts/grid_calibrate.bat

# Stars

To calibrate on stars, first use "star find\_stars" to find *two*
triangles of stars (mag 1 and mag 2) which the tool will correlate to
find the correct orientation of the image. This should be fine with a
poor lens calibration (i.e. linear) with a tolerance of 0.5 degrees
for the angles between the stars (calculated and real) - this is the
triangle_closeness.

Given this rough orientation the tool will have updated its internal
star mapping - i.e. it will have tried to map all the provided pixel
XY to actual real stars (and reoriented again and reupdated the star
mapping, as it happens...)

Now a 3d mapping file can be created from the star mapping, which
basically produces a 'grid mapping calibration' file that can be used
by the 'lens\_calibrate' command; so this can be run - on a limited
range of *yaw* to start with, to provide a lens calibration that maps
the central portion of the image (maybe up to *yaw* of 20 degrees).

With this lens calibration the process can be restarted - although
*finding* the original orientation is not needed, just an *orient* and
then an *update\_star\_mapping*. This should have better mappings for
stars at a *yaw* of 20 degrees and above (to some amount), and so a
lens calibration can be determined for a larger *yaw* range.

This process can be repeated, increasing the yaw range, until the
whole of the lens is mapped.

See scripts/star_calibrate.bat, or the Makefile.
