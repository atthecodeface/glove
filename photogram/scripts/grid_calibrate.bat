# ./target/release/camera_calibrate --batch scripts/grid_calibrate.bat

--db nac/camera_db.json -c nac/camera.json --use_body 5D --use_focus 2000 --use_polys nac/lens_linear.json

locate --write_camera located.json                   --mappings nac/camera_6028_mappings.json --num_pts 6 
orient --write_camera oriented.json  --mappings nac/camera_6028_mappings.json
grid_image      --mappings nac/camera_6028_mappings.json -r ../../Images/4V3A6028.JPG -w a.png
lens_calibrate  --mappings nac/camera_6028_mappings.json --min_yaw 0.01 --max_yaw 25.0 --poly_degree 7 --write_polys lens_cal.json

--use_polys lens_cal.json

locate --write_camera located_cal.json  --mappings nac/camera_6028_mappings.json --num_pts 20
orient --write_camera oriented_cal.json --mappings nac/camera_6028_mappings.json

grid_image --mappings nac/camera_6028_mappings.json -r ../../Images/4V3A6028.JPG -w b.png

yaw_plot --min_yaw 0.1 --max_yaw 25.0 --mappings nac/camera_6028_mappings.json --write_svg lens_calibration.svg
