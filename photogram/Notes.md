# Notes on what should work


## Grid lens calibration (use lens '50mm linear')

Using a grid, we can locate, orient and then show the mapping - note that at the edges the crosses are not well aligned

./target/release/camera_calibrate --db nac/camera_db.json -c nac/camera.json --use_body 5D --use_lens '50mm linear' --use_focus 2000 locate --num_pts 4 --mappings nac/camera_6028_mappings.json > located.json

./target/release/camera_calibrate --db nac/camera_db.json -c located.json orient --num_pts 4 --mappings nac/camera_6028_mappings.json > oriented.json

./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json grid_image --mappings nac/camera_6028_mappings.json -r ../../Images/4V3A6028.JPG -w a.png

Generate the lens calibration:

./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json lens_calibrate --mappings nac/camera_6028_mappings.json
./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json yaw_plot --mappings nac/camera_6028_mappings.json  > a.svg

Look at the 'a.svg' file and the yaw/yaw curve is clear

The polynomials can be added to the database (or compare with the values already there!)

## Grid post calibration (use lens '50mm')

Using a grid, we can locate, orient and then show the mapping (the first takes 30+ seconds for 20 mappings)

./target/release/camera_calibrate --db nac/camera_db.json -c nac/camera.json --use_body 5D --use_lens 50mm --use_focus 2000 locate --num_pts 20 --mappings nac/camera_6028_mappings.json > located.json

./target/release/camera_calibrate --db nac/camera_db.json -c located.json orient --mappings nac/camera_6028_mappings.json > oriented.json

./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json grid_image --mappings nac/camera_6028_mappings.json -r ../../Images/4V3A6028.JPG -w a.png


Check the calibration

./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json lens_calibrate --mappings nac/camera_6028_mappings.json
./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json yaw_plot --mappings nac/camera_6028_mappings.json  > a.svg

Recalibrate, by starting with the linear lens mapping

./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json --use_lens '50mm linear' lens_calibrate --mappings nac/camera_6028_mappings.json

./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json --use_lens '50mm linear' yaw_plot --mappings nac/camera_6028_mappings.json  > a.svg
./target/release/camera_calibrate --db nac/camera_db.json -c oriented.json --use_lens '50mm linear' roll_plot --mappings nac/camera_6028_mappings.json  > a.svg

# Stars

Find an initial orientation based on six stars

./target/release/ic_play --db nac/camera_db.json -c nac/camera.json --use_body T2i --use_lens '50mm' --use_focus 2000 nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 5.0 --closeness 0.5 find_stars > orient_on_six_stars.json

Find which points map to which stars with a fair degree of inaccuracy

./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.2 star_mapping > stars_on_six_stars.json

Now reorient based on those mappings

./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json stars_on_six_stars.json --catalog hipp_bright orient > orient_on_mapped.json

Show the mapped stars with that orientation

./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped.json stars_on_six_stars.json --catalog hipp_bright show_star_mapping -r ../../Images/IMG_4924.JPG -w a.png

Remap the stars the show again

./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped.json nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.3 star_mapping > a.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped.json a.json --catalog hipp_bright --search_brightness 6.0 show_star_mapping -r ../../Images/IMG_4924.JPG -w a.png

Now reorient again based on those mappings

./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped.json a.json --catalog hipp_bright orient > orient_on_all.json

Remap the stars the show again

./target/release/ic_play --db nac/camera_db.json -c orient_on_all.json nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.3 star_mapping > a.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_all.json a.json --catalog hipp_bright --search_brightness 6.0 show_star_mapping -r ../../Images/IMG_4924.JPG -w a.png


./target/release/ic_play --db nac/camera_db.json -c nac/camera_4924_start.json nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 5.0 --closeness 0.2 find_stars > orient_on_six_stars.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.2 star_mapping >  star_mappings_4924_on_six_stars.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json star_mappings_4924_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.2 map_stars > orient_on_mapped_stars.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped_stars.json nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.2 star_mapping > star_mappings_4924_on_mapped_stars.json


./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json star_mappings_4924_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.014 -r ../../Images/IMG_4924.JPG -w a.png map_stars
./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped_stars.json star_mappings_4924_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.014 calibrate_desc > mapping_4924.json
./target/release/camera_calibrate --db nac/camera_db.json -c orient_on_mapped_stars.json roll_plot --mappings mapping_4924.json  > a.svg

./target/release/camera_calibrate --db nac/camera_db.json -c orient_on_mapped_stars.json  grid_image --mappings mapping_4924.json -r ../../Images/IMG_4924.JPG -w a.png 
./target/release/camera_calibrate --db nac/camera_db.json -c orient_on_mapped_stars.json orient --mappings mapping_4924.json > reoriented.json
./target/release/ic_play --db nac/camera_db.json -c reoriented.json  nac/camera_4924_star_mappings.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.025 star_mapping > ro_star_mapping_4924.json
./target/release/ic_play --db nac/camera_db.json -c reoriented.json ro_star_mapping_4924.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.014 calibrate_desc > mapping_4924.json

./target/release/camera_calibrate --db nac/camera_db.json -c reoriented.json grid_image --mappings mapping_4924.json -r ../../Images/IMG_4924.JPG -w a.png 
./target/release/ic_play --db nac/camera_db.json -c reoriented.json ro_star_mapping_4924.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.014 -r ../../Images/IMG_4924.JPG -w a.png map_stars

./target/release/ic_play --db nac/camera_db.json -c reoriented.json star_mappings_4924_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 --closeness 0.014 -r ../../Images/IMG_4924.JPG -w a.png map_stars


./target/release/ic_play --db nac/camera_db.json -c nac/camera_5006_start.json nac/camera_5006_star_mappings.json --catalog hipp_bright --search_brightness 5.0 find_stars > orient_on_six_stars.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json nac/camera_5006_star_mappings.json --catalog hipp_bright --search_brightness 6.0 star_mapping > star_mappings_5006_on_six_stars.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json star_mappings_5006_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 map_stars > orient_on_mapped_stars.json
./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped_stars.json nac/camera_5006_star_mappings.json --catalog hipp_bright --search_brightness 6.0 star_mapping > star_mappings_5006_on_mapped_stars.json

./target/release/ic_play --db nac/camera_db.json -c orient_on_six_stars.json star_mappings_5006_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 -r ../../Images/IMG_5006.JPG -w a.png map_stars


./target/release/ic_play --db nac/camera_db.json -c orient_on_mapped_stars.json star_mappings_5006_on_six_stars.json --catalog hipp_bright --search_brightness 6.0 calibrate_desc > mapping_5006.json
./target/release/camera_calibrate --db nac/camera_db.json -c orient_on_mapped_stars.json lens_calibrate --mappings mapping_5006.json  > a.svg

./target/release/camera_calibrate --db nac/camera_db.json -c orient_on_mapped_stars.json roll_plot --mappings mapping_5006.json  > a.svg

