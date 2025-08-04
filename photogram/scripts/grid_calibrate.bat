# ./target/release/camera_calibrate --batch scripts/grid_calibrate.bat

--db nac/camera_db.json -c nac/camera.json --use_body 5D --use_lens 50mm --use_focus 500000 --use_polys nac/lens_linear.json

locate --write_camera located.json   --mappings nac/camera_6028_mappings.json --num_pts 6 
orient --write_camera oriented.json  --mappings nac/camera_6028_mappings.json
grid_image      --mappings nac/camera_6028_mappings.json -r ../../Images/4V3A6028.JPG -w a.png
lens_calibrate  --mappings nac/camera_6028_mappings.json --yaw_min 0.01 --yaw_max 25.0 --poly_degree 7 --write_polys lens_cal.json

--use_polys lens_cal.json

locate --write_camera located_cal.json  --mappings nac/camera_6028_mappings.json --num_pts 12
# --db nac/camera_db.json -c located_cal.json --use_polys lens_cal.json

orient --write_camera oriented_cal.json --mappings nac/camera_6028_mappings.json

grid_image --mappings nac/camera_6028_mappings.json -r ../../Images/4V3A6028.JPG -w b.png

# Note that (0,0) grid is at (3367,2202)
# Note that (0,10) grid is at (3367,2432)
# Note that (10,10) grid is at (3597,2431)
# Hence 3360, 2240 is at (-7/230 * 10, 38/230 * 10) = (-0.3, 1.65)
#
# With focus = 600m  we are at CamPoly[6720x4480 lens EF50mm f1.8 @ 600000mm]   @[12.63,-0.48,410.56] in dir [-3.149e-2,5.309e-3,-9.995e-1]
# Which has (x,y,0) of (-0.28,1.69,0)
#
# With focus = 1m we are at CamPoly[6720x4480 lens EF50mm f1.8 @ 1000mm]   @[14.00,-0.97,432.10] in dir [-3.309e-2,6.174e-3,-9.994e-1]
# Which has (x,y,0) of (-0.28,2.19,0)
#
# So the focus is best (as probably expected) as long

yaw_plot --yaw_min 0.1 --yaw_max 14.0 --mappings nac/camera_6028_mappings.json --write_svg lens_calibration.svg
# --use_deltas
