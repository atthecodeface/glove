Q = @

QMAKE = ${Q} ${MAKE} Q=${Q}

IMG_CAL = ${Q} ./target/release/image_calibrate
ICAL_LOC = locate --steps=100
ICAL_ORIENT = orient
ICAL_IMG = image --pms_color "\#ffff00" --model_color "\#00ffff"
ICAL_RAYS_FROM_MODEL = create_rays_from_model
ICAL_RAYS_FROM_CAMERA = create_rays_from_camera
ICAL_COMB_MODEL = combine_rays_from_model
ICAL_COMB_CAMERA = combine_rays_from_camera

CAMERA_CALIBRATE = ${Q} ./target/release/camera_calibrate
IMAGE_ANALYZE = ${Q} ./target/release/image_analyze

IMAGE_DIR = /Users/gjstark/Git/Images
DATA_DIR = nac/
OUTPUT_DIR = nac_output/
CAL_DIR = cal_output/

LOCATE_STEPS = --steps 31
LOCATE_SIZES = 100 50 10 5 1

CAMERA_DB = --db ${DATA_DIR}camera_db.json
NPS_JSON = ${DATA_DIR}nps_noughts_and_crosses.json
NPS_BASE_JSON = ${DATA_DIR}nps_base.json
NPS_ALL_JSON  = ${DATA_DIR}nps_all.json

NPS = --nps ${NPS_JSON}
NPS_BASE = --nps ${NPS_BASE_JSON}
NPS_ALL = --nps ${NPS_BASE_JSON} --nps ${NPS_ALL_JSON}
NPS_GPM = --nps ${NPS_BASE_JSON} --nps ${OUTPUT_DIR}nps_gpm.json

IMAGE = 4v3a6040
IMAGE_GRID = 4v3a6028

STARS_WITHIN = 60
INITIAL_CAL = nac/lens_linear.json
INITIAL_CAL_EQUIANGULAR = nac/lens_equiangular.json

#a Generics
all:
	@echo "All"

#a Image server
.PHONY: image_server
image_server:
	cargo run --release --bin image_server -- --file_root ~/Git/image_calibrate_wasm/ --threads 16 --project_root ~/Git/glove/photogram/nac --image_root ~/Git/image_calibrate_wasm/http --verbose 1

.PHONY: image_server_debug
image_server_debug:
	cargo run --bin image_server -- --file_root ~/Git/image_calibrate_wasm/ --threads 16 --project_root ~/Git/glove/photogram/nac --image_root ~/Git/image_calibrate_wasm/http --verbose 1

#a Camera calibration
.PHONY: cal_img_4924
cal_img_4924:
	${MAKE} CAMERA_BODY=T2i CAMERA_LENS=50mm MM_FOCUS=2000 YAW_MIN=1.0 YAW_MAX=50.0 CAL_STAR_MAPPINGS=nac/camera_4924_star_mappings.json STAR_CATALOG='--catalog hipp_bright' STAR_BRIGHTNESS='--brightness 9.0' IMG_BG=IMG_4924.JPG camera_calibrate

.PHONY: cal_img_5005
cal_img_5005:
	${MAKE} CAMERA_BODY=T2i CAMERA_LENS=15mm MM_FOCUS=1102 INITIAL_CAL=${INITIAL_CAL} YAW_MIN=2.0 YAW_MAX=60.0 CAL_STAR_MAPPINGS=nac/camera_5005_star_mappings.json STAR_CATALOG='--catalog hipp_bright' STAR_BRIGHTNESS='--brightness 8.0' IMG_BG=IMG_5005.JPG camera_calibrate

.PHONY: cal_img_5006
cal_img_5006:
	${MAKE} CAMERA_BODY=T2i CAMERA_LENS=50mm MM_FOCUS=1390 YAW_MIN=0.3 YAW_MAX=20.0 CAL_STAR_MAPPINGS=nac/camera_5006_star_mappings.json STAR_CATALOG='--catalog hipparcos.json' STAR_BRIGHTNESS='--brightness 9.0' IMG_BG=IMG_5006.JPG camera_calibrate

#a Camera calibration
# The process is stw map six points to generate angles to map 6 stars to get an orientation
#
# Use orientation and lens to stw map all mappings, and find closest stars, record the mapping
#
# Map all the stars using stw and update the orientation
#
# Convert pixel XY mapping to star, to pixel XY mapping to star direction in 3D
#
# Calibrate the lens by linear wts/stw mappings, with 3d point mapping
# to RY relative to the camera and pixel XY mapping to RY relative to
# the sensor
#
# The calibration polynomial of best fit mapping sensor Y to camara Y
#
# A reverse polynomial is generated that is a best fit for poly(sensor
# Y) to sensor Y (rather than using camera Y, as the polynomials are
# meant to be their inverses).
#

# Then update the star mapping for stars close to the orientation
# Now orient using the new selected stars

.PHONY: camera_calibrate
camera_calibrate:
	${MAKE} STAR_BRIGHTNESS='--brightness 5.0' YAW_MIN=3.5 YAW_MAX=20 cal_initial
	${MAKE} cal_step_1
	${MAKE} cal_step_2
	${MAKE} cal_step_3
	${MAKE} cal_step_4
	${MAKE} cal_step_5
	${MAKE} cal_step_6
	${MAKE} cal_step_7
	${MAKE} cal_step_8
	${MAKE} cal_step_9
	${MAKE} cal_step_10
	${MAKE} N=10 YAW_ERROR=0.2 cal_step_final
	${MAKE} N=7 cal_show_mapping_n
	${MAKE} N=final cal_show_mapping_n

# First find the general orientation (note the stars required are of mag 4.49 and brighter)
# Then update the star mapping for stars close to the orientation
# Now orient using the new selected stars
.PHONY: cal_initial
cal_initial:
	@echo "Find initial orientation from 6 stars (three each marked as magnification 1 and 2)"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c nac/camera.json --use_body ${CAMERA_BODY} --use_lens ${CAMERA_LENS} --use_focus ${MM_FOCUS} --use_polys ${INITIAL_CAL} star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.6 find_stars --write_camera ${CAL_DIR}/base_camera.json
	@echo "Now create an updated star mapping with ids from the star catalog for mappings that map on to stars"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/base_camera.json --use_polys ${INITIAL_CAL} star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.4 update_star_mapping --within 20 --write_mapping ${CAL_DIR}/stars_mapped_using_initial_orientation.json
	@echo "Orient based on updated star mapping"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/base_camera.json --use_polys ${INITIAL_CAL} star ${CAL_DIR}/stars_mapped_using_initial_orientation.json ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.2 orient --write_camera ${CAL_DIR}/orient_0.json
	@echo "Update the star mapping again, based on the proper orientation - create '0' calibration stars_mapped"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/base_camera.json --use_polys ${INITIAL_CAL} star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.2 update_star_mapping --within 20 --write_mapping ${CAL_DIR}/stars_mapped_using_0.json


.PHONY: cal_step_1
cal_step_1:
	${MAKE} N=0 NP1=1 POLY_DEGREE=3 YAW_MAX=20 YAW_ERROR=0.2 STAR_BRIGHTNESS='--brightness 7.0' cal_step_n

.PHONY: cal_step_2
cal_step_2:
	${MAKE} N=1 NP1=2 POLY_DEGREE=3 YAW_MAX=22 YAW_ERROR=0.1 STAR_BRIGHTNESS='--brightness 7.5' cal_step_n

.PHONY: cal_step_3
cal_step_3:
	${MAKE} N=2 NP1=3 POLY_DEGREE=3 YAW_MAX=25 YAW_ERROR=0.05 STAR_BRIGHTNESS='--brightness 8.0' cal_step_n

.PHONY: cal_step_4
cal_step_4:
	${MAKE} N=3 NP1=4 POLY_DEGREE=5 YAW_MAX=27 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 8.5' cal_step_n

.PHONY: cal_step_5
cal_step_5:
	${MAKE} N=4 NP1=5 POLY_DEGREE=5 YAW_MAX=30 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 9.0' cal_step_n

.PHONY: cal_step_6
cal_step_6:
	${MAKE} N=5 NP1=6 POLY_DEGREE=5 YAW_MAX=33 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 9.0' cal_step_n

.PHONY: cal_step_7
cal_step_7:
	${MAKE} N=6 NP1=7 POLY_DEGREE=5 YAW_MAX=36 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 9.0' cal_step_n

.PHONY: cal_step_8
cal_step_8:
	${MAKE} N=7 NP1=8 POLY_DEGREE=5 YAW_MAX=39 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 9.0' cal_step_n

.PHONY: cal_step_9
cal_step_9:
	${MAKE} N=8 NP1=9 POLY_DEGREE=5 YAW_MAX=42 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 9.0' cal_step_n

.PHONY: cal_step_10
cal_step_10:
	${MAKE} N=9 NP1=10 POLY_DEGREE=5 YAW_MAX=44 YAW_ERROR=0.02 STAR_BRIGHTNESS='--brightness 9.0' cal_step_n_reset

.PHONY: cal_step_final
cal_step_final:
	${MAKE} NP1=final POLY_DEGREE=5 cal_step_n


.PHONY: cal_reset_np1
cal_reset_np1:
	@echo "Find initial orientation from 6 stars (three each marked as magnification 1 and 2) *given* a calibration N"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c nac/camera.json --use_body ${CAMERA_BODY} --use_lens ${CAMERA_LENS} --use_focus ${MM_FOCUS} --use_polys ${CAL_DIR}/lens_cal_${NP1}.json star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} --brightness 5.5 --closeness 0.6 find_stars --write_camera ${CAL_DIR}/rebase_camera.json
	@echo "Now create an updated star mapping with ids from the star catalog for mappings that map on to stars"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/rebase_camera.json --use_polys ${CAL_DIR}/lens_cal_${NP1}.json star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.4 update_star_mapping --within ${YAW_MAX} --write_mapping ${CAL_DIR}/stars_mapped_using_rebase.json
	@echo "Orient based on updated star mapping"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/base_camera.json --use_polys ${CAL_DIR}/lens_cal_${NP1}.json star ${CAL_DIR}/stars_mapped_using_rebase.json ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.2 orient --write_camera ${CAL_DIR}/orient_${NP1}.json


.PHONY: cal_step_n
cal_step_n:
	${MAKE} cal_output_3d_mapping_n cal_lens_calibrate_n cal_yaw_plot_n cal_orient_n cal_update_star_mapping_n

.PHONY: cal_step_n_reset
cal_step_n_reset:
	${MAKE} cal_output_3d_mapping_n cal_lens_calibrate_n cal_yaw_plot_n cal_reset_np1 cal_update_star_mapping_n

.PHONY: cal_lens_calibrate_n
cal_lens_calibrate_n:
	@echo "Calibrate the lens cal ${NP1}"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/orient_${N}.json --use_polys nac/lens_linear.json lens_calibrate --min_yaw ${YAW_MIN} --max_yaw ${YAW_MAX} --poly_degree ${POLY_DEGREE} --mappings ${CAL_DIR}/mapping_${N}.json --write_polys ${CAL_DIR}/lens_cal_${NP1}.json

.PHONY: cal_yaw_plot_n
cal_yaw_plot_n:
	@echo "Create the Yaw plot of ${N} & ${NP1}"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/orient_${N}.json --use_polys ${CAL_DIR}/lens_cal_${NP1}.json yaw_plot --min_yaw ${YAW_MIN} --max_yaw ${YAW_MAX} --mappings ${CAL_DIR}/mapping_${N}.json > ${CAL_DIR}/cal_mapping_${N}.svg

.PHONY: cal_orient_n
cal_orient_n:
	@echo "Orient based on mapping from ${N} using lens cal ${NP1}"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/base_camera.json --use_polys ${CAL_DIR}/lens_cal_${NP1}.json star ${CAL_DIR}/stars_mapped_using_${N}.json ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.2 orient --write_camera ${CAL_DIR}/orient_${NP1}.json

.PHONY: cal_update_star_mapping_n
cal_update_star_mapping_n:
	@echo "Update the star mapping from ${N} using lens cal and orient ${NP1}"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/orient_${NP1}.json --use_polys ${CAL_DIR}/lens_cal_${NP1}.json star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} ${STAR_BRIGHTNESS} --closeness 0.2 update_star_mapping --within ${YAW_MAX} --yaw_error ${YAW_ERROR} --write_mapping ${CAL_DIR}/stars_mapped_using_${NP1}.json

.PHONY: cal_output_3d_mapping_n
cal_output_3d_mapping_n:
	@echo "Output 3d mapping for ${N} - that is the mapping pxy to the catalog star directions"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/base_camera.json  star ${CAL_DIR}/stars_mapped_using_${N}.json ${STAR_CATALOG} calibrate_desc --write_mapping ${CAL_DIR}/mapping_${N}.json

.PHONY: cal_show_mapping_n
cal_show_mapping_n:
	@echo "Show the mapping given for lens cal ${N}"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_DIR}/orient_${N}.json --use_polys ${CAL_DIR}/lens_cal_${N}.json star ${CAL_DIR}/stars_mapped_using_${N}.json ${STAR_CATALOG} ${STAR_BRIGHTNESS} show_star_mapping --within ${STARS_WITHIN} -r ${IMAGE_DIR}/${IMG_BG} -w ${CAL_DIR}/mapped_stars_${N}.png

.PHONY: cal_show_mapping
cal_show_mapping:
	@echo "Showing star mapping for given orientation, lens calibration, mapped stars, and catalog brightness on top of given background image"
	${CAMERA_CALIBRATE} ${CAMERA_DB} -c ${CAL_ORIENT} --use_polys ${CAL_LENS} star ${CAL_STAR_MAPPINGS} ${STAR_CATALOG} ${STAR_BRIGHTNESS} show_star_mapping --within ${YAW_MAX} -r ${IMAGE_DIR}/${IMG_BG} -w ${IMAGE}.png

.PHONY: x
x:
	${CAMERA_CALIBRATE} ${CAMERA_DB} image_grid --camera ${DATA_DIR}camera_nac_canon.json --read ${IMAGE_DIR}/4v3a6028.JPG --write 28.png
.PHONY: x

y:
	${CAMERA_CALIBRATE} ${CAMERA_DB} calibrate --camera ${DATA_DIR}camera_calibrate_6028.json --read ${IMAGE_DIR}/4v3a6028.JPG --write 28.png


#a Correlation
FLAGS ?= 0
ANGLE ?= 0.0
# DO NOT USE!
# COPY ?= --kernel copy
KERNEL = window_corr 
KERNEL_FLAGS = ${COPY} --kernel ${KERNEL} --kernel_flags ${FLAGS} --kernel_angle ${ANGLE}

.PHONY: correlate
correlate:
	# 667x263 maps to 888x274
	# 888x274 maps to 1170x262 ? 888x266, 735x70, 1221x206, 1155x126 - all badly with flags 1
	# 259x914 maps to 203x434
	# 825x774 maps to 203x434
	# ${IMAGE_ANALYZE} --read ${IMAGE_DIR}/IMG_1683_sm.JPG --read ${IMAGE_DIR}/IMG_1683_sm.JPG --write correlated${ANGLE}.png luma_kernel_pair --kernel_size 40 --kernel_scale 1.000 --kernel_x 259 --kernel_y 914 ${KERNEL_FLAGS}
	# ${IMAGE_ANALYZE} --read ${IMAGE_DIR}/IMG_1683_sm.JPG --read ${IMAGE_DIR}/IMG_1684_sm.JPG --write correlated${ANGLE}.png luma_kernel_pair --kernel_size 40 --kernel_scale 1.000 --kernel_x 259 --kernel_y 914 ${KERNEL_FLAGS}
	# ${IMAGE_ANALYZE} --read ${IMAGE_DIR}/IMG_1683_sm.JPG --read ${IMAGE_DIR}/IMG_1684_sm.JPG --write correlated.png luma_kernel_pair --kernel_size 10 --kernel_scale 1.000 --kernel_x 667 --kernel_y 263 ${KERNEL_FLAGS}
	# ${IMAGE_ANALYZE} --read ${IMAGE_DIR}/IMG_1683_sm.JPG --read ${IMAGE_DIR}/IMG_1684_sm.JPG --write correlated.png luma_kernel_pair --kernel_size 12 --kernel_scale 1.000 --kernel_x 888 --kernel_y 274 ${KERNEL_FLAGS}
	${IMAGE_ANALYZE} --read ${IMAGE_DIR}/IMG_1683_sm.JPG --read ${IMAGE_DIR}/IMG_1684_sm.JPG --write correlated${ANGLE}.png luma_kernel_pair --kernel_size 40 --kernel_scale 1.000 --kernel_x 825 --kernel_y 774 ${KERNEL_FLAGS}
	# ${IMAGE_ANALYZE} --read ${IMAGE_DIR}/IMG_1683_sm.JPG --read ${IMAGE_DIR}/IMG_1683_sm.JPG --write correlated.png luma_kernel_pair --kernel_size 40 --kernel_scale 1.000 --kernel_x 499 --kernel_y 636 ${KERNEL_FLAGS}
	# ${IMAGE_ANALYZE} --read ${IMAGE_DIR}/1683_at_825_774.jpg --read ${IMAGE_DIR}/1683_at_825_774.jpg --write correlated.png luma_kernel_pair --kernel_size 40 --kernel_scale 1.000 --kernel_x 60 --kernel_y 160 ${KERNEL_FLAGS}

.PHONY:corr90
corr90:
	${MAKE} correlate ANGLE=0
	${MAKE} correlate ANGLE=05
	${MAKE} correlate ANGLE=10
	${MAKE} correlate ANGLE=15
	${MAKE} correlate ANGLE=20
	${MAKE} correlate ANGLE=25
	${MAKE} correlate ANGLE=30
	${MAKE} correlate ANGLE=35
	${MAKE} correlate ANGLE=40
	${MAKE} correlate ANGLE=45
	${MAKE} correlate ANGLE=50
	${MAKE} correlate ANGLE=55
	${MAKE} correlate ANGLE=60
	${MAKE} correlate ANGLE=65
	${MAKE} correlate ANGLE=70
	${MAKE} correlate ANGLE=75
	${MAKE} correlate ANGLE=80
	${MAKE} correlate ANGLE=85
	${MAKE} correlate ANGLE=90
	${MAKE} correlate ANGLE=330
	${MAKE} correlate ANGLE=335
	${MAKE} correlate ANGLE=340
	${MAKE} correlate ANGLE=345
	${MAKE} correlate ANGLE=350
	${MAKE} correlate ANGLE=355

${IMG_CAL}: build

.PHONY: build
build:
	cargo build --release

.PHONY: test
test:
	cargo test --release

.PHONY: locate
locate: located_${IMAGE}.json

located_${IMAGE}.json: ${IMG_CAL} pms_nac_${IMAGE}_training.json camera_nac_${IMAGE}.json ${NPS_JSON}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC}  --pms pms_nac_${IMAGE}_training.json --camera camera_nac_${IMAGE}.json --read ${IMAGE_DIR}/${IMAGE}.JPG --write ${IMAGE}.png > $@

.PHONY: create_rays
create_rays: ${IMAGE}_rays.json

.PHONY: ${IMAGE}_rays.json
${IMAGE}_rays.json: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_RAYS_FROM_CAMERA} --pms pms_nac_${IMAGE}.json --camera located_${IMAGE}.json > $@

.PHONY: combine_rays
combine_rays: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_COMB_CAMERA} 4v3a6040_rays.json 4v3a6041_rays.json 4v3a6042_rays.json

.PHONY: rays_from_model
rays_from_model: ${IMAGE}_rays_from_model.json

.PHONY: ${IMAGE}_rays_from_model.json
${IMAGE}_rays_from_model.json: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_RAYS_FROM_MODEL} --pms pms_nac_${IMAGE}.json --camera located_${IMAGE}.json > $@

.PHONY: locate_all_training
locate_all_training:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6040.json --pms pms_nac_4v3a6040_training.json > located_4v3a6040.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6041.json --pms pms_nac_4v3a6041_training.json > located_4v3a6041.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6042.json --pms pms_nac_4v3a6042_training.json > located_4v3a6042.json

.PHONY: locate_all_untraining
locate_all_untraining:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6040.json --pms pms_nac_4v3a6040.json > located_4v3a6040.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6041.json --pms pms_nac_4v3a6041.json > located_4v3a6041.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6042.json --pms pms_nac_4v3a6042.json > located_4v3a6042.json

.PHONY: orient_all
orient_all:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_ORIENT} --camera located_4v3a6040.json --pms pms_nac_4v3a6040_training.json > oriented_4v3a6040.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_ORIENT} --camera located_4v3a6041.json --pms pms_nac_4v3a6041_training.json > oriented_4v3a6041.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_ORIENT} --camera located_4v3a6042.json --pms pms_nac_4v3a6042_training.json > oriented_4v3a6042.json
	cp oriented_4v3a6040.json located_4v3a6040.json 
	cp oriented_4v3a6041.json located_4v3a6041.json 
	cp oriented_4v3a6042.json located_4v3a6042.json 

.PHONY: orient_all_untraining
orient_all_untraining:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_ORIENT} --camera located_4v3a6040.json --pms pms_nac_4v3a6040.json > oriented_4v3a6040.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_ORIENT} --camera located_4v3a6041.json --pms pms_nac_4v3a6041.json > oriented_4v3a6041.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_ORIENT} --camera located_4v3a6042.json --pms pms_nac_4v3a6042.json > oriented_4v3a6042.json
	cp oriented_4v3a6040.json located_4v3a6040.json 
	cp oriented_4v3a6041.json located_4v3a6041.json 
	cp oriented_4v3a6042.json located_4v3a6042.json 

.PHONY: images_all
images_all:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_IMG} --camera oriented_4v3a6040.json --pms pms_nac_4v3a6040.json --read ${IMAGE_DIR}/4v3a6040.JPG --write 40.png
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_IMG} --camera oriented_4v3a6041.json --pms pms_nac_4v3a6041.json --read ${IMAGE_DIR}/4v3a6041.JPG --write 41.png
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_IMG} --camera oriented_4v3a6042.json --pms pms_nac_4v3a6042.json --read ${IMAGE_DIR}/4v3a6042.JPG --write 42.png

.PHONY: wibble
wibble: ${IMG_CAL}
	# ${IMG_CAL} ${CAMERA_DB} ${NPS} adjust_model --np "1 tl game" camera_nac_4v3a6040.json pms_nac_4v3a6040_training.json camera_nac_4v3a6041.json pms_nac_4v3a6041_training.json camera_nac_4v3a6042.json pms_nac_4v3a6042_training.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} adjust_model --np "& mid-tip game" camera_nac_4v3a6040.json pms_nac_4v3a6040_training.json camera_nac_4v3a6041.json pms_nac_4v3a6041_training.json camera_nac_4v3a6042.json pms_nac_4v3a6042_training.json

.PHONY: get_model_points
get_model_points: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} get_model_points located_4v3a6040.json pms_nac_4v3a6040.json located_4v3a6041.json pms_nac_4v3a6041.json located_4v3a6042.json pms_nac_4v3a6042.json


# This simply finds non-background (top left pixel) regions and their centres-of-gravity on an image
find_regions: ${IMG_CAL} 
	${IMAGE_ANALYZE} --read ${IMAGE_DIR}/4v3a6028_with_corners.png find_regions

# This finds the grid points from non-black pixels in image_with_corners.png, determines the centre, and outputs a mapping file
grid_points: ${IMG_CAL} 
	${IMAGE_ANALYZE} --read ${IMAGE_DIR}/${IMAGE_GRID}_with_corners.png --write ${OUTPUT_DIR}corners.png find_grid_points > ${OUTPUT_DIR}grid_${IMAGE_GRID}.json

find_regions_image: ${IMG_CAL} 
	${IMAGE_ANALYZE} --read ${IMAGE_DIR}/${IMAGE}_pms.png find_regions

get_point_mappings: ${IMG_CAL} 
	${IMG_CAL} ${CAMERA_DB} ${NPS_BASE} get_point_mappings --read ${IMAGE_DIR}/${IMAGE}_pms.png > ${OUTPUT_DIR}pms_base_${IMAGE}.json
	${IMG_CAL} ${CAMERA_DB} ${NPS_ALL} get_point_mappings --read ${IMAGE_DIR}/${IMAGE}_pms.png > ${OUTPUT_DIR}pms_all_${IMAGE}.json

# Locate given the base set of named points
#
# Generate the located_${IMAGE}.json
locate_gpm_base:
	${IMG_CAL} ${CAMERA_DB} ${NPS_BASE} ${ICAL_LOC}    --camera ${DATA_DIR}camera_nac_${IMAGE}.json --pms ${OUTPUT_DIR}pms_base_${IMAGE}.json > ${OUTPUT_DIR}located_${IMAGE}.json
	${IMG_CAL} ${CAMERA_DB} ${NPS_BASE} ${ICAL_ORIENT} --camera ${OUTPUT_DIR}located_${IMAGE}.json  --pms ${OUTPUT_DIR}pms_base_${IMAGE}.json > ${OUTPUT_DIR}oriented_${IMAGE}.json

locate_gpm_all:
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_LOC}    --camera ${DATA_DIR}camera_nac_${IMAGE}.json --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json > ${OUTPUT_DIR}located_${IMAGE}.json
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_ORIENT} --camera ${OUTPUT_DIR}located_${IMAGE}.json  --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json > ${OUTPUT_DIR}oriented_${IMAGE}.json

create_rays_gpm:
	${IMG_CAL} ${CAMERA_DB} ${NPS_ALL} ${ICAL_RAYS_FROM_CAMERA} --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json --camera ${OUTPUT_DIR}oriented_${IMAGE}.json > ${OUTPUT_DIR}rays_fc_${IMAGE}.json

combine_rays_gpm:
	${IMG_CAL} ${CAMERA_DB} ${NPS_ALL} ${ICAL_COMB_CAMERA} ${OUTPUT_DIR}rays_fc_4v3a6040.json ${OUTPUT_DIR}rays_fc_4v3a6041.json ${OUTPUT_DIR}rays_fc_4v3a6042.json

# LOC_OR_ORIENT = located
LOC_OR_ORIENT = oriented

get_model_points_gpm:
	@echo "********************************************************************************"
	@echo "Getting the model points given all the images PMS and located/oriented cameras"
	@echo "********************************************************************************"
	${IMG_CAL} ${CAMERA_DB} ${NPS_ALL} get_model_points ${OUTPUT_DIR}${LOC_OR_ORIENT}_4v3a6040.json ${OUTPUT_DIR}pms_all_4v3a6040.json ${OUTPUT_DIR}${LOC_OR_ORIENT}_4v3a6041.json ${OUTPUT_DIR}pms_all_4v3a6041.json ${OUTPUT_DIR}${LOC_OR_ORIENT}_4v3a6042.json ${OUTPUT_DIR}pms_all_4v3a6042.json > ${OUTPUT_DIR}nps_gpm.json

image_gpm:
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_IMG} --camera ${OUTPUT_DIR}oriented_${IMAGE}.json --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json --read ${IMAGE_DIR}/${IMAGE}.JPG --write ${OUTPUT_DIR}gpm_${IMAGE}.png

create_model_rays_gpm:
	@echo "********************************************************************************"
	@echo "Creating rays from model based on camera ${IMAGE} "
	@echo "********************************************************************************"
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_RAYS_FROM_MODEL} --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json --camera ${OUTPUT_DIR}${LOC_OR_ORIENT}_${IMAGE}.json > ${OUTPUT_DIR}rays_fm_${IMAGE}.json

combine_model_rays_gpm:
	@echo "********************************************************************************"
	@echo "Trying to determine better location + orientation for cameras ${IMAGE} using rays from models"
	@echo "********************************************************************************"
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_COMB_MODEL} --camera ${OUTPUT_DIR}oriented_${IMAGE}.json   --rays ${OUTPUT_DIR}rays_fm_${IMAGE}.json > ${OUTPUT_DIR}relocated_${IMAGE}.json
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_ORIENT}     --camera ${OUTPUT_DIR}relocated_${IMAGE}.json  --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json > ${OUTPUT_DIR}reoriented_${IMAGE}.json

# Orient using all
#
# Generate the oriented2_${IMAGE}.json
reorient_gpm:
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} reorient --camera ${OUTPUT_DIR}relocated_${IMAGE}.json --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json >  ${OUTPUT_DIR}reoriented_${IMAGE}.json

image2_gpm:
	${IMG_CAL} ${CAMERA_DB} ${NPS_GPM} ${ICAL_IMG} --camera ${OUTPUT_DIR}oriented_${IMAGE}.json --pms ${OUTPUT_DIR}pms_all_${IMAGE}.json --read ${IMAGE_DIR}/${IMAGE}.JPG --write ${OUTPUT_DIR}gpm2_${IMAGE}.png

cp_reorient_gpm:
	cp ${OUTPUT_DIR}reoriented_${IMAGE}.json ${OUTPUT_DIR}oriented_${IMAGE}.json

start_gpm: ${IMG_CAL}
	${QMAKE} IMAGE=4v3a6040 get_point_mappings
	${QMAKE} IMAGE=4v3a6041 get_point_mappings
	${QMAKE} IMAGE=4v3a6042 get_point_mappings
	${QMAKE} IMAGE=4v3a6040 locate_gpm_base
	${QMAKE} IMAGE=4v3a6041 locate_gpm_base
	${QMAKE} IMAGE=4v3a6042 locate_gpm_base

do_gpm: ${IMG_CAL}
	${QMAKE} get_model_points_gpm
	${QMAKE} IMAGE=4v3a6040 image_gpm
	${QMAKE} IMAGE=4v3a6041 image_gpm
	${QMAKE} IMAGE=4v3a6042 image_gpm

	${QMAKE} IMAGE=4v3a6040 create_model_rays_gpm
	${QMAKE} IMAGE=4v3a6041 create_model_rays_gpm
	${QMAKE} IMAGE=4v3a6042 create_model_rays_gpm
	${QMAKE} IMAGE=4v3a6040 combine_model_rays_gpm
	${QMAKE} IMAGE=4v3a6041 combine_model_rays_gpm
	${QMAKE} IMAGE=4v3a6042 combine_model_rays_gpm

	${QMAKE} IMAGE=4v3a6040 locate_gpm_all
	${QMAKE} IMAGE=4v3a6041 locate_gpm_all
	${QMAKE} IMAGE=4v3a6042 locate_gpm_all

	${QMAKE} IMAGE=4v3a6040 image2_gpm
	${QMAKE} IMAGE=4v3a6041 image2_gpm
	${QMAKE} IMAGE=4v3a6042 image2_gpm

all_gpm:
	${QMAKE} start_gpm
	${QMAKE} do_gpm
	${QMAKE} iter_gpm
	${QMAKE} all_image2_gpm

iter_gpm: ${IMG_CAL}
	${QMAKE} get_model_points_gpm
	${QMAKE} IMAGE=4v3a6040 create_model_rays_gpm
	${QMAKE} IMAGE=4v3a6041 create_model_rays_gpm
	${QMAKE} IMAGE=4v3a6042 create_model_rays_gpm
	${QMAKE} IMAGE=4v3a6040 combine_model_rays_gpm
	${QMAKE} IMAGE=4v3a6041 combine_model_rays_gpm
	${QMAKE} IMAGE=4v3a6042 combine_model_rays_gpm
	${QMAKE} IMAGE=4v3a6040 cp_reorient_gpm
	${QMAKE} IMAGE=4v3a6041 cp_reorient_gpm
	${QMAKE} IMAGE=4v3a6042 cp_reorient_gpm

all_image2_gpm:
	${QMAKE} IMAGE=4v3a6040 image2_gpm
	${QMAKE} IMAGE=4v3a6041 image2_gpm
	${QMAKE} IMAGE=4v3a6042 image2_gpm

iter_gpm5: ${IMG_CAL}
	${QMAKE} iter_gpm
	${QMAKE} iter_gpm
	${QMAKE} iter_gpm
	${QMAKE} iter_gpm
	${QMAKE} iter_gpm

PROJECT = --project nac/nac_all_proj.json
PATCH_NPS =  "1 tl game" "10011" "10009"
PATCH_NPS =  "0 bl game" "1 tl game" "10003" "10000"
PATCH_NPS =  "M middle 1" "10003" "10011"
PATCH_NPS =  "0 bl game" "10000" "10001"
PATCH_NPS =  "10005" "0cm ruler" "1cm ruler" "2cm ruler" "3cm ruler" "10010"
patch:
	${IMG_CAL} ${PROJECT} image_patch --cip 0 --read ../../Images/4V3A6041.JPG --write p0.png ${PATCH_NPS}
	${IMG_CAL} ${PROJECT} image_patch --cip 1 --read ../../Images/4V3A6040.JPG --write p1.png ${PATCH_NPS}
	${IMG_CAL} ${PROJECT} image_patch --cip 2 --read ../../Images/4V3A6042.JPG --write p2.png ${PATCH_NPS}
