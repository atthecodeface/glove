IMG_CAL = ./target/release/image_calibrate
ICAL_LOC = locate_v2
ICAL_IMG = image
ICAL_RAYS_FROM_MODEL = create_rays_from_model
ICAL_RAYS_FROM_CAMERA = create_rays_from_camera
ICAL_COMB_MODEL = combine_rays_from_model
ICAL_COMB_CAMERA = combine_rays_from_camera
LOCATE_CAMERA = ./target/release/locate_camera
LOCATE2 = ./target/release/locate_v2
CREATE_RAYS = ./target/release/create_rays
COMBINE_RAYS = ./target/release/combine_rays

CAMERA_DB = --db camera_db.json
NPS_JSON = nps_noughts_and_crosses.json
NPS = --nps ${NPS_JSON}
IMAGE_DIR = /Users/gavinjstark/Git/Images
LOCATE_STEPS = --steps 31
LOCATE_SIZES = 100 50 10 5 1

IMAGE = 4v3a6042

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
	# ${LOCATE_CAMERA} ${CAMERA_DB} ${NPS} --total ${LOCATE_STEPS} ${LOCATE_SIZES} --pms pms_nac_${IMAGE}.json --camera camera_nac_${IMAGE}.json --read ${IMAGE_DIR}/${IMAGE}.JPG --write a.png > $@

.PHONY: create_rays
create_rays: ${IMAGE}_rays.json

.PHONY: ${IMAGE}_rays.json
${IMAGE}_rays.json: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_RAYS_FROM_CAMERA} --pms pms_nac_${IMAGE}.json --camera located_${IMAGE}.json > $@

.PHONY: combine_rays
combine_rays: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_COMB_MODEL} 4v3a6040_rays.json 4v3a6041_rays.json
	#4v3a6042_rays.json

.PHONY: rays_from_model
rays_from_model: ${IMAGE}_rays_from_model.json

.PHONY: ${IMAGE}_rays_from_model.json
${IMAGE}_rays_from_model.json: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_RAYS_FROM_MODEL} --pms pms_nac_${IMAGE}.json --camera located_${IMAGE}.json > $@

.PHONY: blah
blah:
	# $(MAKE) build
	$(MAKE) IMAGE=4v3a6040 locate
	$(MAKE) IMAGE=4v3a6041 locate
	$(MAKE) IMAGE=4v3a6042 locate
	$(MAKE) IMAGE=4v3a6040 create_rays
	$(MAKE) IMAGE=4v3a6041 create_rays
	$(MAKE) IMAGE=4v3a6042 create_rays
	$(MAKE) IMAGE=4v3a6040 rays_from_model
	$(MAKE) IMAGE=4v3a6041 rays_from_model
	$(MAKE) IMAGE=4v3a6042 rays_from_model
	$(MAKE) combine_rays

.PHONY: do_both
do_both:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6040.json --pms pms_nac_4v3a6040_training.json --read ${IMAGE_DIR}/4v3a6040.JPG --write 40.png > located_4v3a6040.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6041.json --pms pms_nac_4v3a6041_training.json --read ${IMAGE_DIR}/4v3a6041.JPG --write 41.png > located_4v3a6041.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_LOC} --camera camera_nac_4v3a6042.json --pms pms_nac_4v3a6042_training.json --read ${IMAGE_DIR}/4v3a6042.JPG --write 42.png > located_4v3a6042.json

.PHONY: images_all
images_all:
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_IMG} --camera located_4v3a6040.json --pms pms_nac_4v3a6040.json --read ${IMAGE_DIR}/4v3a6040.JPG --write 40.png
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_IMG} --camera located_4v3a6041.json --pms pms_nac_4v3a6041.json --read ${IMAGE_DIR}/4v3a6041.JPG --write 41.png
	${IMG_CAL} ${CAMERA_DB} ${NPS} ${ICAL_IMG} --camera located_4v3a6042.json --pms pms_nac_4v3a6042.json --read ${IMAGE_DIR}/4v3a6042.JPG --write 42.png

.PHONY: wibble
wibble: ${IMG_CAL}
	# ${IMG_CAL} ${CAMERA_DB} ${NPS} adjust_model --np "1 tl game" camera_nac_4v3a6040.json pms_nac_4v3a6040_training.json camera_nac_4v3a6041.json pms_nac_4v3a6041_training.json camera_nac_4v3a6042.json pms_nac_4v3a6042_training.json
	${IMG_CAL} ${CAMERA_DB} ${NPS} adjust_model --np "& mid-tip game" camera_nac_4v3a6040.json pms_nac_4v3a6040_training.json camera_nac_4v3a6041.json pms_nac_4v3a6041_training.json camera_nac_4v3a6042.json pms_nac_4v3a6042_training.json

.PHONY: get_model_points
get_model_points: ${IMG_CAL}
	${IMG_CAL} ${CAMERA_DB} ${NPS} get_model_points located_4v3a6040.json pms_nac_4v3a6040.json located_4v3a6041.json pms_nac_4v3a6041.json located_4v3a6042.json pms_nac_4v3a6042.json
