IMG_CAL = ./target/release/image_calibrate
ICAL_LOC = locate --steps=100
ICAL_ORIENT = orient
ICAL_IMG = image --pms_color "\#ffff00" --model_color "\#00ffff"
ICAL_RAYS_FROM_MODEL = create_rays_from_model
ICAL_RAYS_FROM_CAMERA = create_rays_from_camera
ICAL_COMB_MODEL = combine_rays_from_model
ICAL_COMB_CAMERA = combine_rays_from_camera

CAMERA_CALIBRATE = ./target/release/camera_calibrate
IMAGE_ANALYZE = ./target/release/image_analyze

IMAGE_DIR = /Users/gavinjstark/Git/Images
DATA_DIR = nac/
OUTPUT_DIR = nac_output/

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


.PHONY: x
x:
	${CAMERA_CALIBRATE} ${CAMERA_DB} image_grid --camera ${DATA_DIR}camera_nac_canon.json --read ${IMAGE_DIR}/4v3a6028.JPG --write 28.png
.PHONY: x

y:
	${CAMERA_CALIBRATE} ${CAMERA_DB} calibrate --camera ${DATA_DIR}camera_calibrate_6028.json --read ${IMAGE_DIR}/4v3a6028.JPG --write 28.png

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
	$(MAKE) IMAGE=4v3a6040 get_point_mappings
	$(MAKE) IMAGE=4v3a6041 get_point_mappings
	$(MAKE) IMAGE=4v3a6042 get_point_mappings
	$(MAKE) IMAGE=4v3a6040 locate_gpm_base
	$(MAKE) IMAGE=4v3a6041 locate_gpm_base
	$(MAKE) IMAGE=4v3a6042 locate_gpm_base

do_gpm: ${IMG_CAL}
	$(MAKE) get_model_points_gpm
	$(MAKE) IMAGE=4v3a6040 image_gpm
	$(MAKE) IMAGE=4v3a6041 image_gpm
	$(MAKE) IMAGE=4v3a6042 image_gpm

	$(MAKE) IMAGE=4v3a6040 create_model_rays_gpm
	$(MAKE) IMAGE=4v3a6041 create_model_rays_gpm
	$(MAKE) IMAGE=4v3a6042 create_model_rays_gpm
	$(MAKE) IMAGE=4v3a6040 combine_model_rays_gpm
	$(MAKE) IMAGE=4v3a6041 combine_model_rays_gpm
	$(MAKE) IMAGE=4v3a6042 combine_model_rays_gpm

	$(MAKE) IMAGE=4v3a6040 locate_gpm_all
	$(MAKE) IMAGE=4v3a6041 locate_gpm_all
	$(MAKE) IMAGE=4v3a6042 locate_gpm_all

	$(MAKE) IMAGE=4v3a6040 image2_gpm
	$(MAKE) IMAGE=4v3a6041 image2_gpm
	$(MAKE) IMAGE=4v3a6042 image2_gpm

all_gpm:
	$(MAKE) start_gpm
	$(MAKE) do_gpm
	$(MAKE) iter_gpm
	$(MAKE) all_image2_gpm

iter_gpm: ${IMG_CAL}
	$(MAKE) get_model_points_gpm
	$(MAKE) IMAGE=4v3a6040 create_model_rays_gpm
	$(MAKE) IMAGE=4v3a6041 create_model_rays_gpm
	$(MAKE) IMAGE=4v3a6042 create_model_rays_gpm
	$(MAKE) IMAGE=4v3a6040 combine_model_rays_gpm
	$(MAKE) IMAGE=4v3a6041 combine_model_rays_gpm
	$(MAKE) IMAGE=4v3a6042 combine_model_rays_gpm
	$(MAKE) IMAGE=4v3a6040 cp_reorient_gpm
	$(MAKE) IMAGE=4v3a6041 cp_reorient_gpm
	$(MAKE) IMAGE=4v3a6042 cp_reorient_gpm

all_image2_gpm:
	$(MAKE) IMAGE=4v3a6040 image2_gpm
	$(MAKE) IMAGE=4v3a6041 image2_gpm
	$(MAKE) IMAGE=4v3a6042 image2_gpm

iter_gpm5: ${IMG_CAL}
	$(MAKE) iter_gpm
	$(MAKE) iter_gpm
	$(MAKE) iter_gpm
	$(MAKE) iter_gpm
	$(MAKE) iter_gpm
