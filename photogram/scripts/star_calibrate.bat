# 	${CAMERA_CALIBRATE} ${CAMERA_DB} -c nac/camera.json --use_body ${CAMERA_BODY} --use_lens ${CAMERA_LENS} --use_focus ${MM_FOCUS} --use_polys ${INITIAL_CAL}

--db nac/camera_db.json -c nac/orient_5005_final.json

--use_polys nac/lens_cal_5005_final.json
# --use_polys nac/lens_equiangular.json
# --use_polys nac/lens_stereographic.json
--use_polys nac/lens_linear.json

star nac/camera_5005_star_mappings.json --catalog hipp_bright --brightness 5.0  --triangle_closeness 0.5 --closeness 0.6 find_stars  --within 20 --yaw_error 0.03
star  --brightness 6.0 --write_mapping mapping_3d_0.json  --write_camera orient_0.json  --write_star_mapping stars_mapped_using_0.json calibrate_desc 
--use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 20.0 --poly_degree 3 --mappings mapping_3d_0.json --write_polys lens_cal_1.json

--use_polys lens_cal_1.json

star --brightness 8.0 --closeness 0.2 orient
star --brightness 8.0 --closeness 0.2 update_star_mapping --within 25.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.2 --write_mapping mapping_3d_1.json  --write_camera orient_1.json  --write_star_mapping stars_mapped_using_1.json calibrate_desc 

--use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 25.0 --poly_degree 3 --mappings mapping_3d_1.json --write_polys lens_cal_2.json

--use_polys lens_cal_2.json
# star --brightness 8.0 --triangle_closeness=0.08 --closeness 0.1 find_stars --within 30 --yaw_error 0.03

# Should not orient on stars outside 30.0 degrees as they may be wildly remapped
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 30.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.1 orient
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03

star --brightness 8.0 --closeness 0.2 --write_mapping mapping_3d_2.json  --write_camera orient_2.json  --write_star_mapping stars_mapped_using_2.json calibrate_desc 

# Plot mapping points (sensor pxy as direction, star direction) ignoring camera calibration
# and plot camera calibration polynomials
yaw_plot --mappings mapping_3d_2.json --min_yaw 2.0 --max_yaw 45.0 --write_svg a.svg

--use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 27.0 --poly_degree 5 --mappings mapping_3d_2.json --write_polys lens_cal_3.json
--use_polys lens_cal_3.json
yaw_plot --mappings mapping_3d_2.json --min_yaw 2.0 --max_yaw 35.0 --write_svg b.svg

star --brightness 8.0 --closeness 0.1 update_star_mapping --within 27.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.1 orient
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.2 --write_mapping mapping_3d_3.json  --write_camera orient_3.json  --write_star_mapping stars_mapped_using_3.json calibrate_desc 

--use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 37.0 --poly_degree 5 --mappings mapping_3d_3.json --write_polys lens_cal_4.json
--use_polys lens_cal_4.json


star --brightness 7.0 show_star_mapping --within 45 -r ../../Images/IMG_5005.JPG -w o.png
# yaw_plot --mappings map_N.json --min_yaw 2.0 --max_yaw 45.0 --write_svg c.svg --use_deltas

 
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 30.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.1 orient
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.2 --write_mapping map_N.json  --write_camera orient_N.json  --write_star_mapping stars_N.json calibrate_desc 
--use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 33.0 --poly_degree 5 --mappings map_N.json --write_polys lens_cal_N.json
--use_polys lens_cal_N.json
 
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 33.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.1 orient
star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03
star --brightness 8.0 --closeness 0.2 --write_mapping map_N.json  --write_camera orient_N.json  --write_star_mapping stars_N.json calibrate_desc 
yaw_plot --mappings map_N.json --min_yaw 2.0 --max_yaw 45.0 --write_svg b.svg
--use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 36.0 --poly_degree 5 --mappings map_N.json --write_polys lens_cal_N.json
--use_polys lens_cal_N.json
yaw_plot --mappings map_N.json --min_yaw 2.0 --max_yaw 45.0 --write_svg c.svg

# star --brightness 8.0 --closeness 0.1 update_star_mapping --within 36.0 --yaw_error 0.03
# star --brightness 8.0 --closeness 0.1 orient
# star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03
# star --brightness 8.0 --closeness 0.2 --write_mapping map_N.json  --write_camera orient_N.json  --write_star_mapping stars_N.json calibrate_desc 
# --use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 39.0 --poly_degree 5 --mappings map_N.json --write_polys lens_cal_N.json
# --use_polys lens_cal_N.json
# 
# star --brightness 8.0 --closeness 0.1 update_star_mapping --within 39.0 --yaw_error 0.03
# star --brightness 8.0 --closeness 0.1 orient
# star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03
# star --brightness 8.0 --closeness 0.2 --write_mapping map_N.json  --write_camera orient_N.json  --write_star_mapping stars_N.json calibrate_desc 
# --use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 42.0 --poly_degree 5 --mappings map_N.json --write_polys lens_cal_N.json
# --use_polys lens_cal_N.json
# 
# star --brightness 8.0 --closeness 0.1 update_star_mapping --within 42.0 --yaw_error 0.03
# star --brightness 8.0 --closeness 0.1 orient
# star --brightness 8.0 --closeness 0.1 update_star_mapping --within 45.0 --yaw_error 0.03
# star --brightness 8.0 --closeness 0.2 --write_mapping map_N.json  --write_camera orient_N.json  --write_star_mapping stars_N.json calibrate_desc 
# 
# yaw_plot --mappings map_N.json --min_yaw 2.0 --max_yaw 45.0 --write_svg d.svg
# 
# --use_polys nac/lens_linear.json lens_calibrate --min_yaw 1.0 --max_yaw 45.0 --poly_degree 5 --mappings map_N.json --write_polys lens_cal_N.json
# --use_polys lens_cal_N.json


